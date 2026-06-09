using System;
using System.Drawing;
using System.IO;
using System.Windows;
using System.Windows.Forms;
using Microsoft.Win32;
using SteamIdlePicker.Services;
using Application = System.Windows.Application;

namespace SteamIdlePicker;

public partial class App : Application
{
    public static bool IsDarkMode { get; private set; }

    private NotifyIcon? _notifyIcon;
    private ToolStripMenuItem? _showItem;
    private ToolStripMenuItem? _quitItem;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        IsDarkMode = DetectDarkMode();
        var lang = DetectLanguage();

        Resources.MergedDictionaries.Add(new ResourceDictionary
        {
            Source = new Uri($"/SteamIdlePicker;component/Resources/Theme.{(IsDarkMode ? "Dark" : "Light")}.xaml",
                             UriKind.Relative)
        });

        LanguageService.Apply(lang);
        LanguageService.LanguageChanged += OnLanguageChanged;

        _showItem = new ToolStripMenuItem(LanguageService.Get("Str.TrayShow"), null, (_, _) => ShowMainWindow());
        _quitItem = new ToolStripMenuItem(LanguageService.Get("Str.TrayQuit"), null, (_, _) => ExitApp());

        var menu = new ContextMenuStrip();
        menu.Items.Add(_showItem);
        menu.Items.Add(new ToolStripSeparator());
        menu.Items.Add(_quitItem);

        var iconPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Resources", "app.ico");
        var icon = File.Exists(iconPath) ? new System.Drawing.Icon(iconPath) : SystemIcons.Application;

        _notifyIcon = new NotifyIcon
        {
            Icon = icon,
            Visible = true,
            Text = "Steam Idle Picker",
            ContextMenuStrip = menu
        };
        _notifyIcon.DoubleClick += (_, _) => ShowMainWindow();

        var mainWindow = new MainWindow();
        MainWindow = mainWindow;
        mainWindow.Show();
    }

    private static bool DetectDarkMode()
    {
        try
        {
            var v = Registry.GetValue(
                @"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "AppsUseLightTheme", 1);
            return v is int i && i == 0;
        }
        catch { return true; }
    }

    private static string DetectLanguage()
    {
        var lang = System.Globalization.CultureInfo.CurrentUICulture.TwoLetterISOLanguageName;
        return lang == "ja" ? "ja" : "en";
    }

    private void OnLanguageChanged(object? sender, EventArgs e)
    {
        if (_showItem != null) _showItem.Text = LanguageService.Get("Str.TrayShow");
        if (_quitItem != null) _quitItem.Text = LanguageService.Get("Str.TrayQuit");
    }

    private void ShowMainWindow()
    {
        MainWindow.Show();
        MainWindow.WindowState = WindowState.Normal;
        MainWindow.Activate();
    }

    private void ExitApp()
    {
        if (MainWindow is MainWindow mw)
            mw.ViewModel.Cleanup();
        _notifyIcon?.Dispose();
        Shutdown();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        _notifyIcon?.Dispose();
        base.OnExit(e);
    }
}
