using System;
using System.Drawing;
using System.IO;
using System.Windows;
using System.Windows.Forms;
using SteamIdlePicker.Services;
using Application = System.Windows.Application;

namespace SteamIdlePicker;

public partial class App : Application
{
    private NotifyIcon? _notifyIcon;
    private ToolStripMenuItem? _showItem;
    private ToolStripMenuItem? _quitItem;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        var settings = new SettingsService().Load();
        LanguageService.Apply(settings.Language);
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

    private void OnLanguageChanged(object? sender, System.EventArgs e)
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
