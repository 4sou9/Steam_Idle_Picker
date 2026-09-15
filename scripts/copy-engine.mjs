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

// steam-idle links against the SDK bundled with steamworks-sys, so ship the DLL from
// that same SDK. A DLL from another SDK version may not match the generated bindings.
// Mirrors steamworks-sys's build.rs, including its STEAM_SDK_LOCATION override.
function findSteamSdk() {
  if (process.env.STEAM_SDK_LOCATION) return process.env.STEAM_SDK_LOCATION;
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--format-version", "1"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    })
  );
  const sys = metadata.packages.find((p) => p.name === "steamworks-sys");
  if (!sys) throw new Error("steamworks-sys not found in cargo metadata");
  return join(dirname(sys.manifest_path), "lib", "steam");
}

const idlerExe = join(root, "target", "release", "steam-idle.exe");
const steamApiDll = join(findSteamSdk(), "redistributable_bin", "win64", "steam_api64.dll");

if (!existsSync(idlerExe)) {
  throw new Error(`steam-idle.exe not found at ${idlerExe}`);
}
if (!existsSync(steamApiDll)) {
  throw new Error(`steam_api64.dll not found at ${steamApiDll}`);
}

copyFileSync(idlerExe, join(engineDir, "steam-idle.exe"));
copyFileSync(steamApiDll, join(engineDir, "steam_api64.dll"));

console.log(`[copy-engine] staged engine files into ${engineDir}`);
