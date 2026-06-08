using System;
using System.Runtime.InteropServices;

namespace SteamIdlePicker.SteamClient.Interfaces;

[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct ISteamClient018
{
    public IntPtr CreateSteamPipe;
    public IntPtr ReleaseSteamPipe;
    public IntPtr ConnectToGlobalUser;
    public IntPtr CreateLocalUser;
    public IntPtr ReleaseUser;
    public IntPtr GetISteamUser;
    public IntPtr GetISteamGameServer;
    public IntPtr SetLocalIPBinding;
    public IntPtr GetISteamFriends;
    public IntPtr GetISteamUtils;
    public IntPtr GetISteamMatchmaking;
    public IntPtr GetISteamMatchmakingServers;
    public IntPtr GetISteamGenericInterface;
    public IntPtr GetISteamUserStats;
    public IntPtr GetISteamGameServerStats;
    public IntPtr GetISteamApps;
}
