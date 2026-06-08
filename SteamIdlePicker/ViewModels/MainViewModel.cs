using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Threading.Tasks;
using System.Windows.Input;
using System.Windows.Threading;
using SteamIdlePicker.Models;
using SteamIdlePicker.Services;

namespace SteamIdlePicker.ViewModels;

public class MainViewModel : INotifyPropertyChanged
{
    private readonly SettingsService _settingsService;
    private readonly GameCacheService _gameCacheService;
    private readonly IdleManager _idleManager;
    private readonly DispatcherTimer _timer;

    private string _searchText = string.Empty;
    private bool _isIdling;
    private bool _isRefreshing;
    private string _elapsedTime = "00:00:00";
    private string _lastFetched = "—";
    private string _statusMessage = string.Empty;
    private DateTime _idleStartTime;

    public const int MaxSelection = 32;

    public ObservableCollection<GameItemViewModel> AllGames { get; } = [];
    public ObservableCollection<GameItemViewModel> FilteredGames { get; } = [];

    public AppSettings Settings { get; private set; }

    public string SearchText
    {
        get => _searchText;
        set { _searchText = value; OnPropertyChanged(); ApplyFilter(); }
    }

    public bool IsIdling
    {
        get => _isIdling;
        private set { _isIdling = value; OnPropertyChanged(); OnPropertyChanged(nameof(ToggleButtonLabel)); }
    }

    public bool IsRefreshing
    {
        get => _isRefreshing;
        private set { _isRefreshing = value; OnPropertyChanged(); }
    }

    public string ElapsedTime
    {
        get => _elapsedTime;
        private set { _elapsedTime = value; OnPropertyChanged(); }
    }

    public string LastFetched
    {
        get => _lastFetched;
        private set { _lastFetched = value; OnPropertyChanged(); }
    }

    public string StatusMessage
    {
        get => _statusMessage;
        private set { _statusMessage = value; OnPropertyChanged(); }
    }

    public int SelectedCount => AllGames.Count(g => g.IsSelected);
    public int IdlingCount => AllGames.Count(g => g.IsIdling);
    public string ToggleButtonLabel => IsIdling
        ? LanguageService.Get("Str.IdleStop")
        : LanguageService.Get("Str.IdleStart");

    public ICommand ToggleIdleCommand { get; }
    public ICommand StopSingleCommand { get; }
    public ICommand ClearSelectionCommand { get; }
    public ICommand RefreshLibraryCommand { get; }
    public ICommand ToggleLanguageCommand { get; }

    public MainViewModel()
    {
        _settingsService = new SettingsService();
        _gameCacheService = new GameCacheService();
        _idleManager = new IdleManager();
        Settings = _settingsService.Load();

        ToggleIdleCommand = new RelayCommand(
            _ => { if (IsIdling) StopAll(); else StartIdle(); },
            _ => IsIdling || SelectedCount > 0);

        StopSingleCommand = new RelayCommand(p => StopSingle((int)p!));
        ClearSelectionCommand = new RelayCommand(_ => ClearSelection());
        RefreshLibraryCommand = new RelayCommand(
            _ => _ = RefreshLibraryAsync(),
            _ => !IsRefreshing);
        ToggleLanguageCommand = new RelayCommand(_ => ToggleLanguage());

        LanguageService.LanguageChanged += (_, _) =>
            OnPropertyChanged(nameof(ToggleButtonLabel));

        _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
        _timer.Tick += (_, _) => UpdateElapsed();

        LoadCache();
    }

    private void ToggleLanguage()
    {
        Settings.Language = Settings.Language == "ja" ? "en" : "ja";
        LanguageService.Apply(Settings.Language);
        _settingsService.Save(Settings);

        // Refresh status message if it's one of the fixed strings
        if (StatusMessage == LanguageService.Get("Str.NoCache") ||
            StatusMessage == string.Empty) return;
        if (!AllGames.Any())
            StatusMessage = LanguageService.Get("Str.NoCache");
    }

    private void LoadCache()
    {
        var cache = _gameCacheService.LoadCache();
        if (cache == null)
        {
            StatusMessage = LanguageService.Get("Str.NoCache");
            return;
        }
        PopulateGames(cache, Settings.SelectedGames.ToHashSet());
        LastFetched = cache.FetchedAt.ToString("yyyy-MM-dd HH:mm");
    }

    private void PopulateGames(GameCache cache, System.Collections.Generic.HashSet<int> selectedIds)
    {
        AllGames.Clear();
        foreach (var game in cache.Games)
        {
            var vm = new GameItemViewModel(game);
            if (selectedIds.Contains(game.AppId))
                vm.IsSelected = true;
            vm.PropertyChanged += OnGamePropertyChanged;
            AllGames.Add(vm);
        }
        StatusMessage = string.Empty;
        ApplyFilter();
        NotifyCountsChanged();
    }

    private void ApplyFilter()
    {
        FilteredGames.Clear();
        var search = _searchText.Trim();
        foreach (var vm in AllGames)
        {
            if (string.IsNullOrEmpty(search) ||
                vm.Name.Contains(search, StringComparison.OrdinalIgnoreCase))
                FilteredGames.Add(vm);
        }
    }

    private void OnGamePropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName != nameof(GameItemViewModel.IsSelected)) return;
        var vm = (GameItemViewModel)sender!;

        if (vm.IsSelected && SelectedCount > MaxSelection)
        {
            vm.IsSelected = false;
            return;
        }

        if (IsIdling)
        {
            if (vm.IsSelected) { _idleManager.StartIdle(vm.AppId); vm.IsIdling = true; }
            else { _idleManager.StopIdle(vm.AppId); vm.IsIdling = false; }
        }

        SaveSelectedGames();
        NotifyCountsChanged();
    }

    private void StartIdle()
    {
        foreach (var vm in AllGames.Where(g => g.IsSelected))
        {
            _idleManager.StartIdle(vm.AppId);
            vm.IsIdling = true;
        }
        IsIdling = true;
        _idleStartTime = DateTime.Now;
        _timer.Start();
        NotifyCountsChanged();
    }

    private void StopAll()
    {
        _idleManager.StopAll();
        foreach (var vm in AllGames) vm.IsIdling = false;
        IsIdling = false;
        _timer.Stop();
        ElapsedTime = "00:00:00";
        NotifyCountsChanged();
    }

    private void StopSingle(int appId)
    {
        _idleManager.StopIdle(appId);
        var vm = AllGames.FirstOrDefault(g => g.AppId == appId);
        if (vm != null) { vm.IsIdling = false; vm.IsSelected = false; }
        if (!AllGames.Any(g => g.IsIdling)) { IsIdling = false; _timer.Stop(); ElapsedTime = "00:00:00"; }
        NotifyCountsChanged();
    }

    private void ClearSelection()
    {
        foreach (var vm in AllGames) vm.IsSelected = false;
        SaveSelectedGames();
        NotifyCountsChanged();
    }

    public async Task RefreshLibraryAsync()
    {
        IsRefreshing = true;
        StatusMessage = LanguageService.Get("Str.LoadingLibrary");
        try
        {
            var selectedIds = AllGames.Where(g => g.IsSelected).Select(g => g.AppId).ToHashSet();
            var result = await _gameCacheService.FetchLocalLibraryAsync();
            PopulateGames(result.Cache, selectedIds);
            LastFetched = result.Cache.FetchedAt.ToString("yyyy-MM-dd HH:mm");
            StatusMessage = result.Connected
                ? string.Format(LanguageService.Get("Str.StatusConnected"),
                    result.Cache.Games.Count, result.InstalledCount, result.ResolvedCount)
                : string.Format(LanguageService.Get("Str.StatusOffline"),
                    result.InstalledCount);
        }
        catch (Exception ex)
        {
            StatusMessage = LanguageService.Get("Str.LoadError") + ex.Message;
        }
        finally
        {
            IsRefreshing = false;
        }
    }

    private void UpdateElapsed()
        => ElapsedTime = (DateTime.Now - _idleStartTime).ToString(@"hh\:mm\:ss");

    private void SaveSelectedGames()
    {
        Settings.SelectedGames = AllGames.Where(g => g.IsSelected).Select(g => g.AppId).ToList();
        _settingsService.Save(Settings);
    }

    public void Cleanup() => _idleManager.StopAll();

    private void NotifyCountsChanged()
    {
        OnPropertyChanged(nameof(SelectedCount));
        OnPropertyChanged(nameof(IdlingCount));
    }

    public event PropertyChangedEventHandler? PropertyChanged;
    private void OnPropertyChanged([CallerMemberName] string? name = null)
        => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
}
