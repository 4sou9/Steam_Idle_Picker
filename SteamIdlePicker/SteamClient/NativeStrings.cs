using System;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace SteamIdlePicker.SteamClient;

internal class NativeStrings
{
    public sealed class StringHandle : SafeHandleZeroOrMinusOneIsInvalid
    {
        internal StringHandle(IntPtr preexistingHandle, bool ownsHandle)
            : base(ownsHandle)
        {
            SetHandle(preexistingHandle);
        }

        public IntPtr Handle => handle;

        protected override bool ReleaseHandle()
        {
            if (handle != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(handle);
                handle = IntPtr.Zero;
                return true;
            }
            return false;
        }
    }

    public static unsafe StringHandle StringToStringHandle(string value)
    {
        if (value == null)
            return new StringHandle(IntPtr.Zero, true);

        var bytes = Encoding.UTF8.GetBytes(value);
        var p = Marshal.AllocHGlobal(bytes.Length + 1);
        Marshal.Copy(bytes, 0, p, bytes.Length);
        ((byte*)p)[bytes.Length] = 0;
        return new StringHandle(p, true);
    }

    public static unsafe string? PointerToString(IntPtr nativeData, int length)
    {
        var bytes = (sbyte*)nativeData.ToPointer();
        if (bytes == null) return null;

        int running = 0;
        var b = bytes;
        if (length == 0 || *b == 0) return string.Empty;

        while ((*b++) != 0 && running < length)
            running++;

        return new string(bytes, 0, running, Encoding.UTF8);
    }
}
