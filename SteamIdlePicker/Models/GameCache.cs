namespace SteamIdlePicker.Models;

public class GameCache
{
    public DateTime FetchedAt { get; set; }
    public List<SteamGame> Games { get; set; } = [];
}
