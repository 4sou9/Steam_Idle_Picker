using System;
using System.Collections.Generic;
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

public enum SortMode { None, IdleStatus, Name, AppId }

public class MainViewModel : INotifyPropertyChanged
{
    private readonly SettingsService _settingsService;
    private readonly GameCacheService _gameCacheService;
    private readonly IdleManager _idleManager;
    private readonly DispatcherTimer _timer;

    private string _searchText = string.Empty;
    private bool _isIdling;
    private bool _isRefreshing;
    private string _statusMessage = string.Empty;
    private SortMode _currentSortMode = SortMode.None;
    private bool _sortAscending = true;

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

    public string StatusMessage
    {
        get => _statusMessage;
        private set
        {
            _statusMessage = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(HasStatusMessage));
        }
    }

    public bool HasStatusMessage => !string.IsNullOrEmpty(_statusMessage);

    public int SelectedCount => AllGames.Count(g => g.IsSelected);
    public int IdlingCount => AllGames.Count(g => g.IsIdling);
    public string IdlingDisplay => $"{IdlingCount}/{MaxSelection}";

    public string ToggleButtonLabel => IsIdling
        ? LanguageService.Get("Str.IdleStop")
        : LanguageService.Get("Str.IdleStart");

    // Sort labels — include ↑/↓ when that sort is active
    public string StatusSortLabel => GetSortLabel(SortMode.IdleStatus, "Str.SortStatus");
    public string NameSortLabel   => GetSortLabel(SortMode.Name,       "Str.SortName");
    public string IdSortLabel     => GetSortLabel(SortMode.AppId,      "Str.SortId");
    public string StatusSortArrow => _currentSortMode == SortMode.IdleStatus
        ? (_sortAscending ? " ↑" : " ↓") : string.Empty;

    public ICommand ToggleIdleCommand { get; }
    public ICommand StopSingleCommand { get; }
    public ICommand ClearSelectionCommand { get; }
    public ICommand RefreshLibraryCommand { get; }
    public ICommand SortByStatusCommand { get; }
    public ICommand SortByNameCommand { get; }
    public ICommand SortByIdCommand { get; }

    public MainViewModel()
    {
        _settingsService = new SettingsService();
        _gameCacheService = new GameCacheService();
        _idleManager = new IdleManager();
        Settings = _settingsService.Load();

        ToggleIdleCommand = new RelayCommand(
            _ => { if (IsIdling) StopAll(); else StartIdle(); },
            _ => IsIdling || SelectedCount > 0);

        StopSingleCommand    = new RelayCommand(p => StopSingle((int)p!));
        ClearSelectionCommand = new RelayCommand(_ => ClearSelection());
        RefreshLibraryCommand = new RelayCommand(
            _ => _ = RefreshLibraryAsync(),
            _ => !IsRefreshing);

        SortByStatusCommand = new RelayCommand(_ => ToggleSort(SortMode.IdleStatus));
        SortByNameCommand   = new RelayCommand(_ => ToggleSort(SortMode.Name));
        SortByIdCommand     = new RelayCommand(_ => ToggleSort(SortMode.AppId));

        LanguageService.LanguageChanged += (_, _) =>
        {
            OnPropertyChanged(nameof(ToggleButtonLabel));
            OnPropertyChanged(nameof(StatusSortLabel));
            OnPropertyChanged(nameof(NameSortLabel));
            OnPropertyChanged(nameof(IdSortLabel));
        };

        _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
        _timer.Tick += (_, _) => NotifyCountsChanged();

        LoadCache();
    }

    // ── Sort ──────────────────────────────────────────────────────────────

    private string GetSortLabel(SortMode mode, string key)
    {
        var baseName = LanguageService.Get(key);
        if (_currentSortMode != mode) return baseName;
        return _sortAscending ? $"{baseName} ↑" : $"{baseName} ↓";
    }

    private void ToggleSort(SortMode mode)
    {
        if (_currentSortMode == mode)
            _sortAscending = !_sortAscending;
        else
        {
            _currentSortMode = mode;
            _sortAscending = true;
        }
        NotifySortLabels();
        ApplyFilter();
    }

    private void NotifySortLabels()
    {
        OnPropertyChanged(nameof(StatusSortLabel));
        OnPropertyChanged(nameof(StatusSortArrow));
        OnPropertyChanged(nameof(NameSortLabel));
        OnPropertyChanged(nameof(IdSortLabel));
    }

    // ── Data ──────────────────────────────────────────────────────────────

    private void LoadCache()
    {
        var cache = _gameCacheService.LoadCache();
        if (cache == null)
        {
            StatusMessage = LanguageService.Get("Str.NoCache");
            return;
        }
        PopulateGames(cache, Settings.SelectedGames.ToHashSet());
    }

    private void PopulateGames(GameCache cache, HashSet<int> selectedIds)
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
        var search = _searchText.Trim();
        IEnumerable<GameItemViewModel> result = AllGames;

        if (!string.IsNullOrEmpty(search))
            result = result.Where(g => g.Name.Contains(search, StringComparison.OrdinalIgnoreCase));

        result = _currentSortMode switch
        {
            SortMode.Name => _sortAscending
                ? result.OrderBy(g => g.Name, StringComparer.OrdinalIgnoreCase)
                : result.OrderByDescending(g => g.Name, StringComparer.OrdinalIgnoreCase),
            SortMode.AppId => _sortAscending
                ? result.OrderBy(g => g.AppId)
                : result.OrderByDescending(g => g.AppId),
            SortMode.IdleStatus => _sortAscending
                ? result.OrderByDescending(g => g.IsIdling)  // running first
                : result.OrderBy(g => g.IsIdling),            // not-running first
            _ => result
        };

        FilteredGames.Clear();
        foreach (var vm in result)
            FilteredGames.Add(vm);
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
        _timer.Start();
        NotifyCountsChanged();
    }

    private void StopAll()
    {
        _idleManager.StopAll();
        foreach (var vm in AllGames) vm.IsIdling = false;
        IsIdling = false;
        _timer.Stop();
        NotifyCountsChanged();
    }

    private void StopSingle(int appId)
    {
        _idleManager.StopIdle(appId);
        var vm = AllGames.FirstOrDefault(g => g.AppId == appId);
        if (vm != null) { vm.IsIdling = false; vm.IsSelected = false; }
        if (!AllGames.Any(g => g.IsIdling)) { IsIdling = false; _timer.Stop(); }
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
        OnPropertyChanged(nameof(IdlingDisplay));

        // Re-sort list when idle status changes (e.g. game starts/stops)
        if (_currentSortMode == SortMode.IdleStatus)
            ApplyFilter();
    }

    public event PropertyChangedEventHandler? PropertyChanged;
    private void OnPropertyChanged([CallerMemberName] string? name = null)
        => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
}
