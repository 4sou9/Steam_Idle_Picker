using System;
using System.Runtime.InteropServices;
using SteamIdlePicker.SteamClient.Interfaces;

namespace SteamIdlePicker.SteamClient.Wrappers;

public class SteamApps001 : NativeWrapper<ISteamApps001>
{
    [UnmanagedFunctionPointer(CallingConvention.ThisCall)]
    private delegate int NativeGetAppData(IntPtr self, uint appId, IntPtr key, IntPtr value, int valueLength);

    public string? GetAppData(uint appId, string key)
    {
        using var nativeKey = NativeStrings.StringToStringHandle(key);
        const int valueLength = 1024;
        var valuePtr = Marshal.AllocHGlobal(valueLength);
        try
        {
            int result = Call<int, NativeGetAppData>(
                Functions.GetAppData, ObjectAddress, appId, nativeKey.Handle, valuePtr, valueLength);
            return result == 0 ? null : NativeStrings.PointerToString(valuePtr, valueLength);
        }
        finally
        {
            Marshal.FreeHGlobal(valuePtr);
        }
    }
}
