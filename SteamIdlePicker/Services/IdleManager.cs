using System.Diagnostics;
using System.IO;

namespace SteamIdlePicker.Services;

public class IdleManager
{
    private readonly Dictionary<int, Process> _processes = [];

    private static readonly string EngineDir = Path.Combine(
        AppDomain.CurrentDomain.BaseDirectory, "engine");

    private static readonly string IdlerExe = Path.Combine(EngineDir, "steam-idle.exe");

    public bool IsIdling(int appId) =>
        _processes.TryGetValue(appId, out var p) && !p.HasExited;

    public bool StartIdle(int appId)
    {
        if (IsIdling(appId)) return true;
        if (!File.Exists(IdlerExe)) return false;

        var process = Process.Start(new ProcessStartInfo(IdlerExe, appId.ToString())
        {
            WindowStyle = ProcessWindowStyle.Hidden,
            CreateNoWindow = true,
            UseShellExecute = false,
            WorkingDirectory = EngineDir
        });

        if (process == null) return false;

        _processes[appId] = process;
        return true;
    }

    public void StopIdle(int appId)
    {
        if (!_processes.TryGetValue(appId, out var p)) return;
        try { if (!p.HasExited) p.Kill(); } catch { }
        _processes.Remove(appId);
    }

    public void StopAll()
    {
        foreach (var (_, p) in _processes)
        {
            try { if (!p.HasExited) p.Kill(); } catch { }
        }
        _processes.Clear();
    }
}
