using System;
using System.Threading;
using System.Windows;
using Microsoft.Win32;
using SteamIdlePicker.Services;
using Application = System.Windows.Application;

namespace SteamIdlePicker;

public partial class App : Application
{
    public static bool IsDarkMode { get; private set; }

    private static Mutex? _mutex;

    protected override void OnStartup(StartupEventArgs e)
    {
        bool createdNew;
        try
        {
            _mutex = new Mutex(true, "SteamIdlePicker_SingleInstance", out createdNew);
        }
        catch (AbandonedMutexException ex)
        {
            _mutex = ex.Mutex;
            createdNew = true;
        }

        if (!createdNew)
        {
            _mutex?.Dispose();
            _mutex = null;
            Shutdown();
            return;
        }

        base.OnStartup(e);

        IsDarkMode = DetectDarkMode();
        var lang = DetectLanguage();

        Resources.MergedDictionaries.Add(new ResourceDictionary
        {
            Source = new Uri($"/SteamIdlePicker;component/Resources/Theme.{(IsDarkMode ? "Dark" : "Light")}.xaml",
                             UriKind.Relative)
        });

        LanguageService.Apply(lang);

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

    protected override void OnExit(ExitEventArgs e)
    {
        _mutex?.ReleaseMutex();
        _mutex?.Dispose();
        base.OnExit(e);
    }
}
