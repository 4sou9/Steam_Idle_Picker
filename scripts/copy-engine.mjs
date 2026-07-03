// Builds the steam-idle helper binary and stages it (plus steam_api64.dll) into
// src-tauri/engine/ so tauri.conf.json's `bundle.resources` can pick it up.
// Run manually with `node scripts/copy-engine.mjs` or automatically via
// tauri.conf.json's `beforeBuildCommand`.
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const engineDir = join(root, "src-tauri", "engine");

console.log("[copy-engine] building steam-idle (release)...");
execFileSync("cargo", ["build", "--release", "-p", "steam-idle"], {
  cwd: root,
  stdio: "inherit",
});

mkdirSync(engineDir, { recursive: true });

const idlerExe = join(root, "target", "release", "steam-idle.exe");
const steamApiDll = join(root, "Dependencies", "steam_api64.dll");

if (!existsSync(idlerExe)) {
  throw new Error(`steam-idle.exe not found at ${idlerExe}`);
}
if (!existsSync(steamApiDll)) {
  throw new Error(`steam_api64.dll not found at ${steamApiDll}`);
}

copyFileSync(idlerExe, join(engineDir, "steam-idle.exe"));
copyFileSync(steamApiDll, join(engineDir, "steam_api64.dll"));

console.log(`[copy-engine] staged engine files into ${engineDir}`);
