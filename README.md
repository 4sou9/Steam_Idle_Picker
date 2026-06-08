# Steam Idle Picker

A Windows desktop app that keeps selected Steam games in a "running" state simultaneously.

## Requirements

- Windows 10/11 (x64)
- [.NET 8 Desktop Runtime](https://dotnet.microsoft.com/en-us/download/dotnet/8.0)
- Steam (must be running)

## Usage

1. Launch `SteamIdlePicker.exe`
2. Click **Load Library** to fetch your game list
3. Check the games you want to idle (up to 32)
4. Click **▶ Start Idling** — Steam will show them as "Playing"
5. Click **■ Stop Idling** to stop

### Tray behavior

| Action | Result |
|--------|--------|
| Close (×) | Kills all idle processes and exits |
| Minimize (−) | Hides to tray, idling continues |
| Tray double-click | Restores window |

## How it works

Spawns a hidden child process (`engine/steam-idle.exe`) per game that calls `SteamAPI.Init()` with the target AppID, causing Steam to register it as "Playing".

## Game library

Loaded from local files — no API key required:

1. **Installed games** — `appmanifest_*.acf` in your Steam library
2. **Played games** — `userdata/<steamid>/config/localconfig.vdf`

Games never installed or launched may not appear.

## Building from source

```
dotnet build --configuration Release
```
