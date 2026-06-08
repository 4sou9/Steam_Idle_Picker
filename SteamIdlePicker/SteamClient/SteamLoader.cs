using System;
using System.IO;
using System.Runtime.InteropServices;
using Microsoft.Win32;
using SteamIdlePicker.SteamClient.Wrappers;

namespace SteamIdlePicker.SteamClient;

internal static class SteamLoader
{
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    private static extern IntPtr LoadLibraryEx(string path, IntPtr file, uint flags);

    [DllImport("kernel32.dll", SetLastError = true, BestFitMapping = false, ThrowOnUnmappableChar = true)]
    private static extern IntPtr GetProcAddress(IntPtr module, string name);

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    private static extern bool SetDllDirectory(string path);

    private const uint LoadWithAlteredSearchPath = 8;

    [UnmanagedFunctionPointer(CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    private delegate IntPtr CreateInterfaceDelegate(string version, IntPtr returnCode);

    private static IntPtr _handle = IntPtr.Zero;
    private static CreateInterfaceDelegate? _createInterface;

    public static string? GetInstallPath()
    {
        return (string?)Registry.GetValue(
                   @"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath", null)
               ?? (string?)Registry.GetValue(
                   @"HKEY_LOCAL_MACHINE\SOFTWARE\Valve\Steam", "InstallPath", null);
    }

    public static bool Load()
    {
        if (_handle != IntPtr.Zero) return true;

        var path = GetInstallPath();
        if (path == null) return false;

        SetDllDirectory(path + ";" + Path.Combine(path, "bin"));

        // 64ビットプロセスでは steamclient64.dll を使用する
        var dllName = Environment.Is64BitProcess ? "steamclient64.dll" : "steamclient.dll";
        var dllPath = Path.Combine(path, dllName);
        _handle = LoadLibraryEx(dllPath, IntPtr.Zero, LoadWithAlteredSearchPath);
        if (_handle == IntPtr.Zero) return false;

        var ptr = GetProcAddress(_handle, "CreateInterface");
        if (ptr == IntPtr.Zero) return false;

        _createInterface = Marshal.GetDelegateForFunctionPointer<CreateInterfaceDelegate>(ptr);
        return true;
    }

    public static TClass? CreateInterface<TClass>(string version)
        where TClass : INativeWrapper, new()
    {
        if (_createInterface == null) return default;
        var address = _createInterface(version, IntPtr.Zero);
        if (address == IntPtr.Zero) return default;
        var instance = new TClass();
        instance.SetupFunctions(address);
        return instance;
    }
}
