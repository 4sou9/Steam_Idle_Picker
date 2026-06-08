# Steam Idle Picker

A Windows desktop app that keeps selected Steam games in a "running" state simultaneously.

---

## Features

- Displays your Steam game library (installed games + played games from local config)
- Select up to **32 games** to idle at the same time
- Real-time elapsed time display while idling
- Search/filter games by name
- Minimizes to system tray — idling continues in the background
- Restores previously selected games on startup
- **Japanese / English** UI toggle

## Requirements

- Windows 10/11
- [.NET 8 Desktop Runtime](https://dotnet.microsoft.com/download/dotnet/8)
- Steam (must be running to idle games)

## Usage

1. Launch `SteamIdlePicker.exe`
2. Click **Load Library** (ライブラリ読み込み) to fetch your game list
3. Check the games you want to idle (up to 32)
4. Click **▶ Start Idling** — Steam will show them as "Playing"
5. Click **■ Stop Idling** or individual **×** buttons to stop

### Window behavior

| Action | Result |
|--------|--------|
| Close button (×) | Kills all idle processes and exits immediately |
| Minimize (−) | Hides to system tray, idling continues |
| Tray icon double-click | Restores the window |
| Tray menu → Quit | Kills all idle processes and exits |

## How it works

The main app (`SteamIdlePicker.exe`) spawns a separate child process (`engine/steam-idle.exe`) for each selected game. Each child process calls `SteamAPI.Init()` with the target AppID, causing Steam to register that game as "Playing". Killing the child process stops the idle.

This is the same approach used by [idle_master_extended](https://github.com/JonasNilson/idle_master_extended).

## Game library source

Games are loaded from two local sources — no Steam Web API key required:

1. **Installed games** — read from `appmanifest_*.acf` files in your Steam library folders
2. **Played games** — read from `userdata/<steamid>/config/localconfig.vdf`

Games that are owned but have never been installed or launched may not appear.

## Files

```
SteamIdlePicker.exe          — Main application
SteamIdlePicker.dll          — Managed assembly
SteamIdlePicker.deps.json    — Runtime dependencies
SteamIdlePicker.runtimeconfig.json
engine/
  steam-idle.exe             — Child process for idling
  steam-idle.dll
  steam_api64.dll            — Steam API (Valve)
  Steamworks.NET.dll         — Steamworks.NET wrapper
```

## Settings

Stored in `%AppData%\SteamIdlePicker\`:

| File | Contents |
|------|----------|
| `settings.json` | Language preference, last selected games |
| `games_cache.json` | Cached game list from last library load |

## Building from source

```
dotnet build --configuration Release
```

Output: `SteamIdlePicker/bin/Release/net8.0-windows/`
