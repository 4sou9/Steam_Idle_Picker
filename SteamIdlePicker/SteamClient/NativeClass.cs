using System;
using System.Runtime.InteropServices;

namespace SteamIdlePicker.SteamClient;

[StructLayout(LayoutKind.Sequential, Pack = 1, CharSet = CharSet.Ansi)]
internal struct NativeClass
{
    public IntPtr VirtualTable;
}
