using System;
using System.Collections.Generic;
using System.IO;
using System.Text.RegularExpressions;
using SteamIdlePicker.Models;
using SteamIdlePicker.SteamClient;

namespace SteamIdlePicker.Services;

public class SteamLibraryService : IDisposable
{
    private readonly SteamClientSession _session = new();
    private bool _initialized;

    /// <summary>steamclient64.dll への接続を試みる。Steam未起動でも他の機能は使える。</summary>
    public bool Initialize()
    {
        _initialized = _session.Initialize();
        return _initialized;
    }

    public bool IsConnected => _initialized;

    /// <summary>steamclient.dll 経由でゲーム名を取得する。接続失敗時は null を返す。</summary>
    public string? GetGameName(uint appId)
    {
        if (!_initialized) return null;
        return _session.Apps001?.GetAppData(appId, "name");
    }

    /// <summary>
    /// ACFファイルからインストール済みゲームを取得する（名前付き）。
    /// steamclient.dll 不要で常に動作する。
    /// </summary>
    public static IEnumerable<SteamGame> GetInstalledGames()
    {
        var steamPath = SteamLoader.GetInstallPath();
        if (steamPath == null) yield break;

        var libraryPaths = new List<string> { Path.Combine(steamPath, "steamapps") };

        var foldersVdf = Path.Combine(steamPath, "steamapps", "libraryfolders.vdf");
        if (File.Exists(foldersVdf))
        {
            var content = File.ReadAllText(foldersVdf);
            foreach (Match m in Regex.Matches(content, @"""path""\s+""([^""]+)"""))
            {
                var dir = Path.Combine(m.Groups[1].Value.Replace("\\\\", "\\"), "steamapps");
                if (Directory.Exists(dir)) libraryPaths.Add(dir);
            }
        }

        foreach (var libPath in libraryPaths)
        {
            if (!Directory.Exists(libPath)) continue;
            foreach (var acf in Directory.GetFiles(libPath, "appmanifest_*.acf"))
            {
                string acfContent;
                try { acfContent = File.ReadAllText(acf); }
                catch { continue; }

                var idMatch = Regex.Match(acfContent, @"""appid""\s+""(\d+)""");
                var nameMatch = Regex.Match(acfContent, @"""name""\s+""([^""]+)""");
                if (!idMatch.Success || !nameMatch.Success) continue;
                if (!uint.TryParse(idMatch.Groups[1].Value, out var id)) continue;

                var name = nameMatch.Groups[1].Value;
                if (string.IsNullOrWhiteSpace(name)) continue;

                yield return new SteamGame { AppId = (int)id, Name = name };
            }
        }
    }

    /// <summary>localconfig.vdf からプレイ済み全 AppID を取得する。</summary>
    public static IEnumerable<uint> GetLocalConfigAppIds()
    {
        var steamPath = SteamLoader.GetInstallPath();
        if (steamPath == null) yield break;

        var userDataPath = Path.Combine(steamPath, "userdata");
        if (!Directory.Exists(userDataPath)) yield break;

        foreach (var userDir in Directory.GetDirectories(userDataPath))
        {
            var configPath = Path.Combine(userDir, "config", "localconfig.vdf");
            if (!File.Exists(configPath)) continue;

            foreach (var id in ParseLocalConfigAppIds(configPath))
                yield return id;
        }
    }

    private static IEnumerable<uint> ParseLocalConfigAppIds(string path)
    {
        string content;
        try { content = File.ReadAllText(path); }
        catch { yield break; }

        VdfNode root;
        try { root = VdfParser.Parse(content); }
        catch { yield break; }

        // Software > Valve > Steam > apps
        if (!root.Children.TryGetValue("Software", out var sw)) yield break;
        if (!sw.Children.TryGetValue("Valve", out var valve)) yield break;
        if (!valve.Children.TryGetValue("Steam", out var steam)) yield break;
        if (!steam.Children.TryGetValue("apps", out var apps)) yield break;

        foreach (var key in apps.Children.Keys)
            if (uint.TryParse(key, out var id))
                yield return id;
    }

    public void Dispose() => _session.Dispose();
}
