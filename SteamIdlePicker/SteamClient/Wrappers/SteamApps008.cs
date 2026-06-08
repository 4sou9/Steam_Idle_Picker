using System;
using System.Runtime.InteropServices;
using SteamIdlePicker.SteamClient.Interfaces;

namespace SteamIdlePicker.SteamClient.Wrappers;

public class SteamApps008 : NativeWrapper<ISteamApps008>
{
    [UnmanagedFunctionPointer(CallingConvention.ThisCall)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool NativeIsSubscribedApp(IntPtr self, uint appId);

    public bool IsSubscribedApp(uint appId)
        => Call<bool, NativeIsSubscribedApp>(Functions.IsSubscribedApp, ObjectAddress, appId);
}
