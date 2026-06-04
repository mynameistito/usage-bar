/**
 * Prepares a release by ensuring the version tag exists.
 *
 * This intentionally does not create the GitHub Release. Immutable releases
 * require all assets to be available before publishing, so release creation
 * happens after the installer build jobs complete.
 */
import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { join } from "node:path";

interface PackageJson {
  readonly version: string;
}

const root = process.cwd();
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

const writeOutput = (name: string, value: string): void => {
  const outputPath = process.env.GITHUB_OUTPUT;

  if (!outputPath) {
    process.stdout.write(`${name}=${value}\n`);
    return;
  }

  appendFileSync(outputPath, `${name}=${value}\n`);
};

const releaseExists = (): boolean => {
  try {
    run("gh", ["release", "view", tag]);
    return true;
  } catch {
    return false;
  }
};

const tagExists = (): boolean => {
  try {
    run("git", ["rev-parse", "--verify", `refs/tags/${tag}`]);
    return true;
  } catch {
    return false;
  }
};

writeOutput("tag", tag);

if (releaseExists()) {
  process.stdout.write(`Release ${tag} already exists - skipping.\n`);
  writeOutput("should_publish", "false");
  process.exit(0);
}

runInherit("git", ["fetch", "--tags"]);

if (tagExists()) {
  process.stdout.write(`Tag ${tag} already exists - reusing it.\n`);
} else {
  runInherit("git", ["tag", tag]);
  runInherit("git", ["push", "origin", tag]);
  process.stdout.write(`Created and pushed tag ${tag}.\n`);
}

writeOutput("should_publish", "true");
