using Steamworks;

if (args.Length == 0) return;

Environment.SetEnvironmentVariable("SteamAppId", args[0]);

if (!SteamAPI.Init()) return;

while (true)
{
    SteamAPI.RunCallbacks();
    Thread.Sleep(1000);
}
