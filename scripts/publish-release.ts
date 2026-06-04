/**
 * Publishes a GitHub Release with all installer assets attached at creation.
 *
 * This is compatible with GitHub immutable releases because assets are passed
 * directly to `gh release create`, which uploads them before publication.
 */
import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";

interface PackageJson {
  readonly version: string;
}

const root = process.cwd();
const artifactDir = resolve(root, process.argv[2] ?? "dist");
const { version } = JSON.parse(
  readFileSync(join(root, "package.json"), "utf8")
) as PackageJson;
const tag = `v${version}`;

const run = (command: string, args: readonly string[]): string =>
  execFileSync(command, [...args], {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();

const runInherit = (command: string, args: readonly string[]): void => {
  execFileSync(command, [...args], { cwd: root, stdio: "inherit" });
};

const releaseExists = (): boolean => {
  try {
    run("gh", ["release", "view", tag]);
    return true;
  } catch {
    return false;
  }
};

const collectInstallerAssets = (directory: string): string[] => {
  if (!existsSync(directory)) {
    throw new Error(`Artifact directory does not exist: ${directory}`);
  }

  const assets: string[] = [];
  const entries = readdirSync(directory);

  for (const entry of entries) {
    const entryPath = join(directory, entry);
    const stats = statSync(entryPath);

    if (stats.isDirectory()) {
      assets.push(...collectInstallerAssets(entryPath));
      continue;
    }

    if (entryPath.endsWith(".exe") || entryPath.endsWith(".msi")) {
      assets.push(entryPath);
    }
  }

  return assets.sort();
};

const createReleaseNotes = (): string => {
  const changelog = readFileSync(join(root, "CHANGELOG.md"), "utf8");
  const escapedVersion = version.replace(/\./g, "\\.");
  const sectionPattern = new RegExp(
    `## ${escapedVersion}[^\n]*\n([\\s\\S]*?)(?=\n## |$)`
  );
  const sectionMatch = changelog.match(sectionPattern);
  const notes = sectionMatch
    ? sectionMatch[1].trim()
    : "See CHANGELOG.md for details.";
  const tempDir = mkdtempSync(join(tmpdir(), "usage-bar-release-"));
  const notesPath = join(tempDir, "release-notes.md");

  writeFileSync(notesPath, notes);

  return notesPath;
};

if (releaseExists()) {
  process.stdout.write(`Release ${tag} already exists - skipping.\n`);
  process.exit(0);
}

const assets = collectInstallerAssets(artifactDir);

if (assets.length === 0) {
  throw new Error(`No installer assets found in ${artifactDir}`);
}

const notesPath = createReleaseNotes();

try {
  runInherit("gh", [
    "release",
    "create",
    tag,
    ...assets,
    "--title",
    tag,
    "--notes-file",
    notesPath,
    "--verify-tag",
  ]);
  process.stdout.write(`Published GitHub release ${tag}.\n`);
} finally {
  rmSync(dirname(notesPath), { recursive: true, force: true });
}
