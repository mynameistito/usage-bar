/**
 * Creates a git tag and GitHub release for the current version.
 *
 * Called via `bun run release` by the CI release workflow after the
 * "Version Packages" PR is merged (i.e. when there are no pending changesets).
 *
 * This script assumes the caller has verified the tag doesn't exist.
 * Exits 1 if the tag exists as a defensive check for unexpected scenarios.
 *
 * Requires:
 *   - GH_TOKEN env var (set automatically in GitHub Actions)
 *   - git configured with push access to origin
 */
import { execSync } from "node:child_process";
import { readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();

const { version } = JSON.parse(
  readFileSync(join(root, "package.json"), "utf8")
) as { version: string };

const tag = `v${version}`;

// ── Defensive guard ──────────────────────────────────────────────────────────
// The CI workflow checks for an existing tag before calling this script, so
// this guard should not trigger in normal CI flow. It exists as a safety net
// when the script is invoked outside of CI (e.g. locally).
try {
  execSync(`git rev-parse --verify refs/tags/${tag}`, { stdio: "pipe" });
  console.log(`Tag ${tag} already exists — nothing to release.`);
  process.exit(1);
} catch {
  // Tag does not exist yet — proceed with the release.
}

// ── Tag ──────────────────────────────────────────────────────────────────────
execSync(`git tag ${tag}`, { stdio: "inherit" });
execSync(`git push origin ${tag}`, { stdio: "inherit" });
console.log(`Created and pushed tag ${tag}.`);

// ── Release notes ────────────────────────────────────────────────────────────
// Extract the section for this version from CHANGELOG.md.
const changelog = readFileSync(join(root, "CHANGELOG.md"), "utf8");

// Match "## <version> …\n<body>" up to the next "## " heading or end of file.
const escapedVersion = version.replace(/\./g, "\\.");
const sectionPattern = new RegExp(
  `## ${escapedVersion}[^\n]*\n([\\s\\S]*?)(?=\n## |$)`
);
const sectionMatch = changelog.match(sectionPattern);
const notes = sectionMatch
  ? sectionMatch[1].trim()
  : "See CHANGELOG.md for details.";

// Write to a temp file so we don't have to worry about shell escaping.
const notesPath = join(root, ".changeset", "_release-notes.md");
writeFileSync(notesPath, notes);

// ── GitHub release ───────────────────────────────────────────────────────────
execSync(
  `gh release create "${tag}" --title "${tag}" --notes-file "${notesPath}"`,
  { stdio: "inherit" }
);
console.log(`Published GitHub release ${tag}.`);
unlinkSync(notesPath);
