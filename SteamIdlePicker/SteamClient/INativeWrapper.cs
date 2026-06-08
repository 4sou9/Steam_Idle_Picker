using System;

namespace SteamIdlePicker.SteamClient;

public interface INativeWrapper
{
    void SetupFunctions(IntPtr objectAddress);
}
