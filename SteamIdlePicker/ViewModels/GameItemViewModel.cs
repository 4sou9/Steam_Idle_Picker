using System.ComponentModel;
using System.Runtime.CompilerServices;
using SteamIdlePicker.Models;

namespace SteamIdlePicker.ViewModels;

public class GameItemViewModel : INotifyPropertyChanged
{
    private bool _isSelected;
    private bool _isIdling;

    public int AppId { get; }
    public string Name { get; }

    public bool IsSelected
    {
        get => _isSelected;
        set { _isSelected = value; OnPropertyChanged(); }
    }

    public bool IsIdling
    {
        get => _isIdling;
        set { _isIdling = value; OnPropertyChanged(); }
    }

    public GameItemViewModel(SteamGame game)
    {
        AppId = game.AppId;
        Name = game.Name;
    }

    public event PropertyChangedEventHandler? PropertyChanged;

    private void OnPropertyChanged([CallerMemberName] string? name = null)
        => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
}
