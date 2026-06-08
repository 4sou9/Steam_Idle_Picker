using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace SteamIdlePicker.SteamClient;

public abstract class NativeWrapper<TNativeFunctions> : INativeWrapper
{
    protected IntPtr ObjectAddress;
    protected TNativeFunctions Functions = default!;

    public void SetupFunctions(IntPtr objectAddress)
    {
        ObjectAddress = objectAddress;

        var iface = (NativeClass)Marshal.PtrToStructure(ObjectAddress, typeof(NativeClass))!;
        Functions = (TNativeFunctions)Marshal.PtrToStructure(iface.VirtualTable, typeof(TNativeFunctions))!;
    }

    private readonly Dictionary<IntPtr, Delegate> _cache = [];

    protected TDelegate GetFunction<TDelegate>(IntPtr pointer) where TDelegate : class
    {
        if (!_cache.TryGetValue(pointer, out var d))
        {
            d = Marshal.GetDelegateForFunctionPointer(pointer, typeof(TDelegate));
            _cache[pointer] = d;
        }
        return (TDelegate)(object)d;
    }

    protected TReturn Call<TReturn, TDelegate>(IntPtr pointer, params object[] args) where TDelegate : class
    {
        var d = _cache.TryGetValue(pointer, out var cached)
            ? cached
            : _cache[pointer] = Marshal.GetDelegateForFunctionPointer(pointer, typeof(TDelegate));
        return (TReturn)d.DynamicInvoke(args)!;
    }
}
