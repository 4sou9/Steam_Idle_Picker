using System;
using System.Runtime.InteropServices;
using SteamIdlePicker.SteamClient.Interfaces;

namespace SteamIdlePicker.SteamClient.Wrappers;

public class SteamClient018 : NativeWrapper<ISteamClient018>
{
    [UnmanagedFunctionPointer(CallingConvention.ThisCall)]
    private delegate int NativeCreateSteamPipe(IntPtr self);

    public int CreateSteamPipe()
        => Call<int, NativeCreateSteamPipe>(Functions.CreateSteamPipe, ObjectAddress);

    [UnmanagedFunctionPointer(CallingConvention.ThisCall)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool NativeReleaseSteamPipe(IntPtr self, int pipe);

    public bool ReleaseSteamPipe(int pipe)
        => Call<bool, NativeReleaseSteamPipe>(Functions.ReleaseSteamPipe, ObjectAddress, pipe);

    [UnmanagedFunctionPointer(CallingConvention.ThisCall)]
    private delegate int NativeConnectToGlobalUser(IntPtr self, int pipe);

    public int ConnectToGlobalUser(int pipe)
        => Call<int, NativeConnectToGlobalUser>(Functions.ConnectToGlobalUser, ObjectAddress, pipe);

    [UnmanagedFunctionPointer(CallingConvention.ThisCall)]
    private delegate void NativeReleaseUser(IntPtr self, int pipe, int user);

    public void ReleaseUser(int pipe, int user)
        => GetFunction<NativeReleaseUser>(Functions.ReleaseUser)(ObjectAddress, pipe, user);

    [UnmanagedFunctionPointer(CallingConvention.ThisCall)]
    private delegate IntPtr NativeGetISteamApps(IntPtr self, int user, int pipe, IntPtr version);

    private TClass? GetISteamApps<TClass>(int user, int pipe, string version)
        where TClass : INativeWrapper, new()
    {
        using var nativeVersion = NativeStrings.StringToStringHandle(version);
        var address = Call<IntPtr, NativeGetISteamApps>(
            Functions.GetISteamApps, ObjectAddress, user, pipe, nativeVersion.Handle);
        if (address == IntPtr.Zero) return default;
        var instance = new TClass();
        instance.SetupFunctions(address);
        return instance;
    }

    public SteamApps001? GetSteamApps001(int user, int pipe)
        => GetISteamApps<SteamApps001>(user, pipe, "STEAMAPPS_INTERFACE_VERSION001");

    public SteamApps008? GetSteamApps008(int user, int pipe)
        => GetISteamApps<SteamApps008>(user, pipe, "STEAMAPPS_INTERFACE_VERSION008");
}
