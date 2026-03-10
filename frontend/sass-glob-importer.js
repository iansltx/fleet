const path = require("path");
const { pathToFileURL, fileURLToPath } = require("url");

const glob = require("glob");
const isGlob = require("is-glob");

// Use a file: URL pointing to a virtual path so sass-loader's source map
// handling doesn't choke on a custom scheme.
const VIRTUAL_DIR = "file:///virtual/sass-glob/";

/**
 * Sass modern-API importer that expands glob patterns in @use/@import
 * statements. Replaces node-sass-glob-importer with a modern API
 * compatible implementation.
 */
module.exports = {
  canonicalize(url, context) {
    if (!isGlob(url)) return null;

    const dir = context.containingUrl
      ? path.dirname(fileURLToPath(context.containingUrl))
      : process.cwd();

    const id = [url, dir, context.fromImport ? "import" : "use"].join("|");
    const encoded = Buffer.from(id).toString("base64");
    return new URL(`${VIRTUAL_DIR}${encoded}.scss`);
  },

  load(canonicalUrl) {
    const href = canonicalUrl.href;
    if (!href.startsWith(VIRTUAL_DIR)) return null;

    const encoded = href.slice(VIRTUAL_DIR.length).replace(/\.scss$/, "");
    const [pattern, dir, type] = Buffer.from(encoded, "base64")
      .toString()
      .split("|");

    const files = glob.sync(path.resolve(dir, pattern));
    const useImport = type === "import";
    const contents = files
      .map((f) => {
        const url = pathToFileURL(f).href;
        return useImport ? `@import "${url}";` : `@use "${url}" as *;`;
      })
      .join("\n");

    return { contents, syntax: "scss" };
  },
};
