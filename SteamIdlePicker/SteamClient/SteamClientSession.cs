using System;
using SteamIdlePicker.SteamClient.Wrappers;

namespace SteamIdlePicker.SteamClient;

public class SteamClientSession : IDisposable
{
    private SteamClient018? _client;
    private int _pipe;
    private int _user;

    public SteamApps001? Apps001 { get; private set; }
    public SteamApps008? Apps008 { get; private set; }

    public bool Initialize()
    {
        if (!SteamLoader.Load()) return false;

        _client = SteamLoader.CreateInterface<SteamClient018>("SteamClient018");
        if (_client == null) return false;

        _pipe = _client.CreateSteamPipe();
        if (_pipe == 0) return false;

        _user = _client.ConnectToGlobalUser(_pipe);
        if (_user == 0) return false;

        Apps001 = _client.GetSteamApps001(_user, _pipe);
        Apps008 = _client.GetSteamApps008(_user, _pipe);

        return Apps001 != null;
    }

    public void Dispose()
    {
        if (_client == null) return;
        if (_user > 0) { _client.ReleaseUser(_pipe, _user); _user = 0; }
        if (_pipe > 0) { _client.ReleaseSteamPipe(_pipe); _pipe = 0; }
    }
}
