const path = require("path");
const { pathToFileURL, fileURLToPath } = require("url");

const glob = require("glob");
const isGlob = require("is-glob");

const PROTOCOL = "glob:";

/**
 * Sass modern-API importer that expands glob patterns in @import statements.
 * Replaces node-sass-glob-importer with a modern API compatible implementation.
 */
module.exports = {
  canonicalize(url, context) {
    if (!isGlob(url)) return null;

    const dir = context.containingUrl
      ? path.dirname(fileURLToPath(context.containingUrl))
      : process.cwd();

    const id = [url, dir, context.fromImport ? "import" : "use"].join("|");
    return new URL(`${PROTOCOL}${Buffer.from(id).toString("base64")}`);
  },

  load(canonicalUrl) {
    if (canonicalUrl.protocol !== PROTOCOL) return null;

    const [pattern, dir, type] = Buffer.from(
      canonicalUrl.pathname,
      "base64"
    )
      .toString()
      .split("|");

    const files = glob.sync(path.resolve(dir, pattern));
    const keyword = type === "import" ? "@import" : "@use";
    const contents = files
      .map((f) => `${keyword} "${pathToFileURL(f).href}";`)
      .join("\n");

    return { contents, syntax: "scss" };
  },
};
