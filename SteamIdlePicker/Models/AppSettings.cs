using System.Collections.Generic;

namespace SteamIdlePicker.Models;

public class AppSettings
{
    public string Language { get; set; } = "ja";
    public List<int> SelectedGames { get; set; } = [];
}
