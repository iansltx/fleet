//! Calendar types matching Go's `server/fleet/calendar.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::policy::PolicyCalendarData;

// ─── Constants ───────────────────────────────────────────────────────────────

pub const CALENDAR_BODY_STATIC_HEADER: &str =
    "reserved this time to make some changes to your work computer";
pub const CALENDAR_EVENT_CONFLICT_TEXT: &str =
    "because there was no remaining availability ";
pub const CALENDAR_DEFAULT_DESCRIPTION: &str =
    "needs to make sure your device meets the organization's requirements.";
pub const CALENDAR_DEFAULT_RESOLUTION: &str = "\
\u{2022} Click the <a href=\"https://fleetdm.com/better\" rel=\"noreferrer\" target=\"_blank\" >Fleet</a> icon in your computer&apos;s menu and<br />&nbsp;&nbsp;&nbsp;select <b>My device</b>
\u{2022} Navigate to the <b>Policies</b> tab
\u{2022} Follow instructions to resolve any policies marked \"Fail\"
\u{2022} Click <b>Refetch</b>";

// ─── CalendarCreateEventOpts ─────────────────────────────────────────────────

/// CalendarCreateEventOpts holds options for creating a calendar event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CalendarCreateEventOpts {
    pub event_uuid: String,
    pub channel_id: String,
    pub resource_id: String,
}

// ─── CalendarGetAndUpdateEventOpts ───────────────────────────────────────────

/// CalendarGetAndUpdateEventOpts holds options for getting and updating a calendar event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CalendarGetAndUpdateEventOpts {
    pub update_timezone: bool,
}

// ─── CalendarWebhookPayload ──────────────────────────────────────────────────

/// CalendarWebhookPayload is the payload sent to a calendar webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarWebhookPayload {
    pub timestamp: DateTime<Utc>,
    pub host_id: u32,
    pub host_display_name: String,
    pub host_serial_number: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failing_policies: Vec<PolicyCalendarData>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error: String,
}
