# Steam Idle Picker

A Windows desktop app that keeps selected Steam games in a "running" state simultaneously.

![Screenshot](01.png)

## Requirements

- Windows 10/11 (x64)
- Steam (must be running to idle; the game list is best refreshed while Steam is running)

## Usage

1. Launch `Steam Idle Picker.exe`
2. Click the refresh button in the header to load your game list
3. Check the games you want to idle (up to 32) — checked games stay pinned to the top of the list, followed by favorites
4. Click the play button — Steam will show them as "Playing"
5. Click the same button (now a stop icon) to stop all idling
6. Uncheck a game to stop idling it individually

The status footer shows how many games are idling out of the 32 max. Problems appear on its right in yellow — for example when Steam is not running, when a game cannot be idled (e.g. it is not owned), or when idling stopped because Steam exited.

## Finding games

- **Search**: type part of a game name; the ✕ button clears the search
- **Favorites**: hover a row and click the ☆ to add it to favorites (★). Favorites are listed right after the checked games
- **Filter**: the drop-down next to the search box shows All, Favorites or Running games
- **Sort**: click "Name" or "ID" in the list header; click again to reverse
- **Right-click a row**: add/remove favorite, copy the AppID, open the Steam store page, or open the game in your Steam library

## Notes

- Language (English / Japanese) and theme (Dark / Light) are detected automatically from Windows settings
- The title bar is custom-drawn (no native Windows chrome) — drag it to move the window, and use the minimize/maximize/close buttons on the right
- Idling stops together with the app, even if the app is killed or crashes, and leftover idle processes from an earlier session are ended at startup
- If Steam cannot be reached while refreshing, the previous game list is kept instead of shrinking to installed games only
- Settings and the game list cache are stored in `%APPDATA%\com.steamidlepicker.app` (moved automatically from `%APPDATA%\SteamIdlePicker` used up to 2.1.1). The uninstaller's "delete app data" option removes it

## Development

Built with [Tauri 2](https://tauri.app) (Rust) + React/TypeScript.

```bash
npm install
npm run tauri dev    # run in dev mode
npm run tauri build  # produce a release build + NSIS installer
```

The app icon is drawn by `scripts/make-icon.py` (Pillow). To change it, edit the script, then run `python scripts/make-icon.py` and `npx tauri icon app-icon.png`, and delete the generated files that `tauri.conf.json` does not reference (`android/`, `ios/`, `icon.icns`, `Square*Logo.png`, ...).
