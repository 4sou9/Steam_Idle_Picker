using System;
using System.Runtime.InteropServices;

namespace SteamIdlePicker.SteamClient.Interfaces;

[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct ISteamApps008
{
    public IntPtr IsSubscribed;
    public IntPtr IsLowViolence;
    public IntPtr IsCybercafe;
    public IntPtr IsVACBanned;
    public IntPtr GetCurrentGameLanguage;
    public IntPtr GetAvailableGameLanguages;
    public IntPtr IsSubscribedApp;
}
