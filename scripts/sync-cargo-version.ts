/**
 * Syncs the version from package.json into src-tauri/Cargo.toml.
 * Run automatically via `bun run version`.
 *
 * Always run from the project root: `bun run sync-version`
 */
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// Matches the bare `version = "..."` line in the [package] section.
// Dependency lines always include `{`, so we skip those separately.
const PACKAGE_VERSION_RE = /^version\s*=\s*"[^"]+"/;

// Matches the `"version": "..."` line in tauri.conf.json.
const TAURI_CONF_VERSION_RE = /^\s*"version":\s*"[^"]+"/;
const TAURI_CONF_VERSION_REPLACE_RE = /"version":\s*"[^"]+"/;

const root = process.cwd();

const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as {
  version: string;
};
const { version } = pkg;

const cargoPath = join(root, "src-tauri", "Cargo.toml");
const lines = readFileSync(cargoPath, "utf8").split("\n");

let replaced = false;
const updated = lines.map((line) => {
  // Only replace the first bare `version = "..."` (the [package] field).
  // Dependency version fields look like `foo = { version = "..." }` so they
  // contain `{` on the same line — skip those.
  if (!replaced && PACKAGE_VERSION_RE.test(line) && !line.includes("{")) {
    replaced = true;
    return `version = "${version}"`;
  }
  return line;
});

if (!replaced) {
  console.error("No version field found in Cargo.toml — nothing updated.");
  process.exit(1);
}

writeFileSync(cargoPath, updated.join("\n"), "utf8");
console.log(`Synced Cargo.toml version → ${version}`);

const tauriConfPath = join(root, "src-tauri", "tauri.conf.json");
const tauriConfLines = readFileSync(tauriConfPath, "utf8").split("\n");
const updatedTauriConf = tauriConfLines.map((line) => {
  if (TAURI_CONF_VERSION_RE.test(line)) {
    return line.replace(
      TAURI_CONF_VERSION_REPLACE_RE,
      `"version": "${version}"`
    );
  }
  return line;
});
writeFileSync(tauriConfPath, updatedTauriConf.join("\n"), "utf8");
console.log(`Synced tauri.conf.json version → ${version}`);
