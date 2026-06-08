using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using SteamIdlePicker.Models;

namespace SteamIdlePicker.Services;

public record FetchResult(GameCache Cache, int InstalledCount, int ResolvedCount, bool Connected);

public class GameCacheService
{
    private static readonly string AppDataDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
        "SteamIdlePicker");

    private static readonly string CachePath = Path.Combine(AppDataDir, "games_cache.json");

    public GameCache? LoadCache()
    {
        if (!File.Exists(CachePath)) return null;
        try { return JsonSerializer.Deserialize<GameCache>(File.ReadAllText(CachePath)); }
        catch { return null; }
    }

    public Task<FetchResult> FetchLocalLibraryAsync()
    {
        return Task.Run(() =>
        {
            var games = new Dictionary<int, SteamGame>();

            foreach (var game in SteamLibraryService.GetInstalledGames())
                games[game.AppId] = game;

            int installedCount = games.Count;
            int resolvedCount = 0;
            bool connected = false;

            using (var steamLib = new SteamLibraryService())
            {
                connected = steamLib.Initialize();

                foreach (var appId in SteamLibraryService.GetLocalConfigAppIds())
                {
                    var id = (int)appId;
                    if (games.ContainsKey(id)) continue;
                    if (!connected) continue;

                    var name = steamLib.GetGameName(appId);
                    if (string.IsNullOrWhiteSpace(name)) continue;

                    games[id] = new SteamGame { AppId = id, Name = name };
                    resolvedCount++;
                }
            }

            var list = new List<SteamGame>(games.Values);
            list.Sort((a, b) => string.Compare(a.Name, b.Name, StringComparison.OrdinalIgnoreCase));

            var cache = new GameCache { FetchedAt = DateTime.Now, Games = list };
            Directory.CreateDirectory(AppDataDir);
            File.WriteAllText(CachePath, JsonSerializer.Serialize(cache,
                new JsonSerializerOptions { WriteIndented = true }));

            return new FetchResult(cache, installedCount, resolvedCount, connected);
        });
    }
}
