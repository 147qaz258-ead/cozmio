import { existsSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();
const outDir = path.join(root, "out");
const npmCmd = process.platform === "win32" ? "npm.cmd" : "npm";
const npxCmd = process.platform === "win32" ? "npx.cmd" : "npx";

const config = {
  siteUrl: "https://cozmio.net",
  projectName: "conto-landing",
  branch: "main",
};

const args = new Set(process.argv.slice(2));
const skipLint = args.has("--skip-lint");
const skipBuild = args.has("--skip-build");
const buildOnly = args.has("--build-only");

function run(command, commandArgs, extraEnv = {}) {
  const env = {
    ...process.env,
    NEXT_PUBLIC_SITE_URL: config.siteUrl,
    ...extraEnv,
  };

  const result =
    process.platform === "win32"
      ? spawnSync(
          [command, ...commandArgs]
            .map((part) => (/[ \t"]/.test(part) ? `"${part.replace(/"/g, '\\"')}"` : part))
            .join(" "),
          {
            cwd: root,
            stdio: "inherit",
            shell: true,
            env,
          },
        )
      : spawnSync(command, commandArgs, {
          cwd: root,
          stdio: "inherit",
          env,
        });

  if (result.error) {
    console.error(result.error);
    process.exit(1);
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

console.log(`\nPulseclaw site release`);
console.log(`- site url: ${config.siteUrl}`);
console.log(`- pages project: ${config.projectName}`);
console.log(`- branch: ${config.branch}\n`);

if (!skipLint) {
  run(npmCmd, ["run", "lint"]);
} else {
  console.log("Skipping lint.\n");
}

if (!skipBuild) {
  run(npmCmd, ["run", "build"]);
} else {
  console.log("Skipping build.\n");
}

if (!existsSync(outDir)) {
  console.error(`Static export directory not found: ${outDir}`);
  process.exit(1);
}

if (buildOnly) {
  console.log("Build completed. Skipping deploy because --build-only was provided.");
  process.exit(0);
}

run(npxCmd, [
  "wrangler",
  "pages",
  "deploy",
  "out",
  "--project-name",
  config.projectName,
  "--branch",
  config.branch,
]);
