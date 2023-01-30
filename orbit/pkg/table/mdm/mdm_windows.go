//go:build windows
// +build windows

package mdmbridge

import (
	"bytes"
	"context"
	"crypto/sha256"
	"errors"
	"fmt"
	"os"
	"strings"
	"sync"
	"unsafe"

	"github.com/fleetdm/fleet/v4/orbit/pkg/platform"
	"github.com/google/uuid"
	"github.com/hillu/go-ntdll"
	"github.com/osquery/osquery-go/plugin/table"
	"github.com/rs/zerolog/log"
	"golang.org/x/sys/windows"
	"golang.org/x/sys/windows/registry"
)

var (
	// Windows DLL and functions for runtime binding
	modmdmregistration = windows.NewLazySystemDLL("mdmregistration.dll")
	modmdmlocalmgmt    = windows.NewLazySystemDLL("mdmlocalmanagement.dll")

	procIsDeviceRegisteredWithManagement = modmdmregistration.NewProc("IsDeviceRegisteredWithManagement")
	procSendSyncMLcommand                = modmdmlocalmgmt.NewProc("ApplyLocalManagementSyncML")

	// Synchronization mutex
	mu sync.Mutex

	// SMBIOS UUID SHA256 hash generation executor
	uuidHashGeneration sync.Once

	// SHA256 hash of device SMBIOS UUID
	uuidHash []byte
)

const (
	maxBufSize            = 2048 // Max Unicode Length for UPN - https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-adls/63f5e067-d1b3-4e6e-9e53-a92953b6005b
	flagsRegistryLocation = `SYSTEM\CurrentControlSet\Services\embeddedmode\Parameters`
	flagsRegistryName     = `Flags`
	mdmMutexName          = "__MDM_LOCAL_MANAGEMENT_NAMED_MUTEX__"
)

// Columns is the schema of the table.
func Columns() []table.ColumnDefinition {
	return []table.ColumnDefinition{
		table.TextColumn("enrollment_status"),
		table.TextColumn("enrolled_user"),
		table.TextColumn("mdm_command_input"),
		table.TextColumn("mdm_command_output"),
	}
}

// Generate is called to return the results for the table at query time.
// Constraints for generating can be retrieved from the queryContext.
func Generate(ctx context.Context, queryContext table.QueryContext) ([]map[string]string, error) {
	// grabbing command input query if present
	var inputCmd string

	// checking if 'mdm_command_input' is in the where clause
	if constraintList, present := queryContext.Constraints["mdm_command_input"]; present {
		for _, constraint := range constraintList.Constraints {
			if constraint.Operator == table.OperatorEquals {
				inputCmd = constraint.Expression
				log.Debug().Msgf("mdm_bridge input command request:\n%s", inputCmd)
			}
		}
	}

	// getting MDM enrollment status
	isHostMDMenrolled, enrollmentURI, err := getEnrollmentInfo()
	if err != nil {
		return nil, fmt.Errorf("there was a problem getting enrollment info: %s ", err)
	}

	// updating device enrollment status
	deviceEnrollmentStatus := "device_enrolled"
	if isHostMDMenrolled == 0 {
		deviceEnrollmentStatus = "device_not_enrolled"
	}

	// Executing the input MDM command if it was present
	if len(inputCmd) > 0 {

		// performs the actual MDM cmd execution against the OS MDM stack
		outputCmd, err := executeMDMcommand(inputCmd)
		if err != nil {
			return nil, fmt.Errorf("mdm command execution: %s ", err)
		}

		log.Debug().Msgf("mdm_bridge output command response:\n%s", outputCmd)

		return []map[string]string{
			{
				"enrollment_status":  "enrolled_device",
				"enrolled_user":      enrollmentURI,
				"mdm_command_input":  inputCmd,
				"mdm_command_output": outputCmd,
			},
		}, nil
	}

	// returning table enrollment status + cmd output status if present
	return []map[string]string{
		{
			"enrollment_status": deviceEnrollmentStatus,
			"enrolled_user":     enrollmentURI,
			"mdm_command_input": "",
			"mdm_command_ouput": "",
		},
	}, nil
}

// builtin utf16tostring string expects a uint16 array but we have a pointer to a uint16
// so we need to cast it after converting it to an unsafe pointer
// this is a common pattern though the buffer size varies
// see https://golang.org/pkg/unsafe/#Pointer for more details
func localUTF16toString(ptr unsafe.Pointer) (string, error) {
	if ptr == nil {
		return "", errors.New("failed UTF16 conversion due to null pointer")
	}

	uint16ptrarr := (*[maxBufSize]uint16)(ptr)[:]
	return windows.UTF16ToString(uint16ptrarr), nil
}

// getEnrollmentInfo returns the MDM enrollment status by calling into OS API IsDeviceRegisteredWithManagement()
func getEnrollmentInfo() (uint32, string, error) {
	// variable to hold the MDM enrollment status
	var isDeviceRegisteredWithMDM uint32 = 0

	// heap-allocated buffer to hold the URI data
	buffUriData := make([]uint16, 0, maxBufSize)
	if buffUriData == nil {
		return 0, "", errors.New("failed to allocate memory for URI data")
	}

	// IsDeviceRegisteredWithManagement is going to return the MDM enrollment status
	// https://learn.microsoft.com/en-us/windows/win32/api/mdmregistration/nf-mdmregistration-isdeviceregisteredwithmanagement
	if returnCode, _, err := procIsDeviceRegisteredWithManagement.Call(uintptr(unsafe.Pointer(&isDeviceRegisteredWithMDM)), maxBufSize, uintptr(unsafe.Pointer(&buffUriData))); returnCode != uintptr(windows.ERROR_SUCCESS) {
		return 0, "", fmt.Errorf("there was an error calling IsDeviceRegisteredWithManagement(): %s (0x%X)", err, returnCode)
	}

	// Sanity check to ensure that we are returning a valid string
	uriData := ""
	if isDeviceRegisteredWithMDM > 0 {
		workUriData, err := localUTF16toString(unsafe.Pointer(&buffUriData))
		if err != nil {
			return 0, "", err
		}

		if len(workUriData) > 0 {
			uriData = workUriData
		}
	}

	return isDeviceRegisteredWithMDM, uriData, nil
}

// executeMDMcommand executes syncML MDM commands against the OS MDM stack and returns the status of the command execution
func executeMDMcommand(inputCMD string) (string, error) {
	// Synchronizing MDM command execution
	mu.Lock()
	defer mu.Unlock()

	// Enabling MDM command executions
	if err := enableCmdExecution(); err != nil {
		return "", err
	}

	// Close MDM management mutex if neeeded - this is a hack to enable multiple MDM management calls
	handle, err := windows.OpenMutex(windows.MUTEX_ALL_ACCESS, false, windows.StringToUTF16Ptr(mdmMutexName))
	if err == nil {
		windows.CloseHandle(handle) // closing handle just opened due to OpenMutex()

		// then closing previously used handle
		if err := closeManagementMutex(); err != nil {
			return "", err
		}
	}

	// converting input MDM cmd go string into UTF16 windows string
	inputCmdPtr, err := windows.UTF16FromString(inputCMD)
	if err != nil {
		return "", err
	}

	// MDM stack is ready to receive commands
	var outputStrBuffer *uint16
	if returnCode, _, err := procSendSyncMLcommand.Call(uintptr(unsafe.Pointer(&inputCmdPtr[0])), uintptr(unsafe.Pointer(&outputStrBuffer))); returnCode != uintptr(windows.ERROR_SUCCESS) {
		return "", fmt.Errorf("there was an error calling ApplyLocalManagementSyncML(): %s (0x%X)", err, returnCode)
	}

	// converting windows MDM UTF16 output string into go string
	outputCmd, err := localUTF16toString(unsafe.Pointer(outputStrBuffer))
	if err != nil {
		return "", err
	}

	// freeing OS allocated heap memory
	_, err = windows.LocalFree(windows.Handle(unsafe.Pointer(outputStrBuffer)))
	if err != nil {
		return "", err
	}

	// Disabling MDM command executions
	if err := disableCmdExecution(); err != nil {
		return "", err
	}

	if len(outputCmd) == 0 {
		return "", fmt.Errorf("the OS MDM stack returned an empty string")
	}

	return outputCmd, nil
}

// closeManagementMutex walks the system handles to find and close the MDM management mutexes on
// current process. This is a hack found after reverse engineering mdmlocalmanagement.dll.
func closeManagementMutex() error {
	const bufsize = 2048                     // buffer allocation for native windows syscalls
	currentProcessPID := uint32(os.Getpid()) // current process PID

	var handleOccurences uint32 = 0

	// querying first the list of handles on the kernel using NtQuerySystemInformation() syscall and SystemHandleInformation
	// https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation
	bufQuerySystemSyscall := make([]byte, 0, bufsize)
	var rlen uint32
	if st := ntdll.CallWithExpandingBuffer(func() ntdll.NtStatus {
		return ntdll.NtQuerySystemInformation(
			ntdll.SystemHandleInformation,
			&bufQuerySystemSyscall[0],
			uint32(len(bufQuerySystemSyscall)),
			&rlen,
		)
	}, &bufQuerySystemSyscall, &rlen); st.IsError() {
		return fmt.Errorf("NtQuerySystemInformation: %s, len=%d", st.Error(), rlen)
	}

	// Sanity check on returned buffer
	if bufQuerySystemSyscall == nil {
		return errors.New("invalid list of handles returned by NtQuerySystemInformation")
	}

	// Casting the returned buffer to the SystemHandleInformation type
	// https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/ex/sysinfo/handle.htm
	handlesList := (*ntdll.SystemHandleInformationT)(unsafe.Pointer(&bufQuerySystemSyscall[0]))

	// Iterating over the list of handlers
	for _, systemHandleEntry := range handlesList.GetHandles() {

		// only processing the current process handles
		if currentProcessPID != systemHandleEntry.OwnerPid {
			continue
		}

		// Calling NtQueryObject syscalls with ObjectTypeInformation to obtain object type information of a given handle. This requires static allocation.
		// https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryobject
		var handleObjectTypeBuf [bufsize]byte
		var outputLen uint32 = 0
		st := ntdll.NtQueryObject(ntdll.Handle(systemHandleEntry.HandleValue), ntdll.ObjectTypeInformation, &handleObjectTypeBuf[0], uint32(len(handleObjectTypeBuf)), &outputLen)
		if st != ntdll.STATUS_SUCCESS || outputLen == 0 {
			continue
		}

		// Casting the returned buffer to the OBJECT_TYPE_INFORMATION type
		// https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-__public_object_type_information
		oti := (*ntdll.ObjectTypeInformationT)(unsafe.Pointer(&handleObjectTypeBuf[0]))
		if oti.TypeName.String() == "Mutant" {

			// Calling NtQueryObject syscalls with ObjectNameInformation to obtain named object information of a given handle. This requires static allocation.
			// https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryobject
			var handleObjectNameBuf [bufsize]byte
			var outputLen uint32 = 0
			st := ntdll.NtQueryObject(ntdll.Handle(systemHandleEntry.HandleValue), ntdll.ObjectNameInformation, &handleObjectNameBuf[0], uint32(len(handleObjectNameBuf)), &outputLen)
			if st != ntdll.STATUS_SUCCESS || outputLen == 0 {
				continue
			}

			oni := (*ntdll.ObjectNameInformationT)(unsafe.Pointer(&handleObjectNameBuf[0]))

			if strings.Contains(oni.Name.String(), mdmMutexName) {
				windows.CloseHandle(windows.Handle(systemHandleEntry.HandleValue))
				handleOccurences++
			}
		}
	}

	if handleOccurences == 0 {
		return fmt.Errorf("target named mutex %s was not found", mdmMutexName)
	}

	return nil
}

// getUUIDhash returns the SHA256 hash of the host SMBIOS UUID
func getUUIDhash() ([]byte, error) {
	// Get UUID string first
	uuidStr, _, err := platform.GetSMBiosUUID()
	if err != nil {
		return nil, errors.New("there was a problem retrieving the UUID")
	}

	// Parse UUID string into uuid.UUID type
	uuidMachine, err := uuid.Parse(uuidStr)
	if err != nil {
		return nil, errors.New("there was a problem parsing the UUID")
	}

	// Grabbing the UUID bytes in the expected endianness
	var uuidBytes bytes.Buffer
	uuidBytes.WriteByte(uuidMachine[3])
	uuidBytes.WriteByte(uuidMachine[2])
	uuidBytes.WriteByte(uuidMachine[1])
	uuidBytes.WriteByte(uuidMachine[0])
	uuidBytes.WriteByte(uuidMachine[5])
	uuidBytes.WriteByte(uuidMachine[4])
	uuidBytes.WriteByte(uuidMachine[7])
	uuidBytes.WriteByte(uuidMachine[6])
	uuidBytes.WriteByte(uuidMachine[8])
	uuidBytes.WriteByte(uuidMachine[9])
	uuidBytes.WriteByte(uuidMachine[10])
	uuidBytes.WriteByte(uuidMachine[11])
	uuidBytes.WriteByte(uuidMachine[12])
	uuidBytes.WriteByte(uuidMachine[13])
	uuidBytes.WriteByte(uuidMachine[14])
	uuidBytes.WriteByte(uuidMachine[15])

	// Returning the SHA256 hash of the UUID bytes
	h := sha256.New()
	_, errhash := h.Write(uuidBytes.Bytes())
	if errhash != nil {
		return nil, errors.New("there was a problem generating the SHA256 hash")
	}
	return h.Sum(nil), nil
}

// enableCmdExecution initializes the registry flags required for OS MDM execution
func enableCmdExecution() error {
	// generating SHA256 hash of SMBIOS UUID
	// this is wrapped by sync.Once so it only executes once
	uuidHashGeneration.Do(func() {
		// generate SHA256 hash of UUID bytes
		workHash, err := getUUIDhash()
		if err != nil {
			log.Debug().Err(err).Msg("there was an issue generating the UUID hash")
		}

		// making the UUID hash to be globally accessible
		if len(workHash) > 0 {
			uuidHash = workHash
		}
	})

	// Sanity check on availability of UUID hash
	if len(uuidHash) == 0 {
		return errors.New("there was a problem with UUID SHA256 hash generation")
	}

	// UUID hash is already there, so we just need to write the registry to enable MDM commands
	// execution. Registry flag is set and unset on each command execution to isolate the MDM command
	// execution to this logic only
	k, err := registry.OpenKey(registry.LOCAL_MACHINE, flagsRegistryLocation, registry.QUERY_VALUE|registry.SET_VALUE)
	if err != nil {
		return err
	}

	if err := k.SetBinaryValue(flagsRegistryName, uuidHash); err != nil {
		return err
	}

	if err := k.Close(); err != nil {
		return err
	}

	return nil
}

// enableCmdExecution removes a special registry flag to disable MDM command execution
func disableCmdExecution() error {
	// Here we are just making sure to delete the Flags registry entry
	k, err := registry.OpenKey(registry.LOCAL_MACHINE, flagsRegistryLocation, registry.QUERY_VALUE|registry.SET_VALUE)
	if err != nil {
		return err
	}

	if err := k.DeleteValue(flagsRegistryName); err != nil {
		return err
	}

	if err := k.Close(); err != nil {
		return err
	}

	return nil
}
