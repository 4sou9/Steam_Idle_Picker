using System;
using System.Linq;
using Application = System.Windows.Application;
using ResourceDictionary = System.Windows.ResourceDictionary;

namespace SteamIdlePicker.Services;

public static class LanguageService
{
    public static event EventHandler? LanguageChanged;

    public static void Apply(string language)
    {
        var dicts = Application.Current.Resources.MergedDictionaries;
        var existing = dicts.FirstOrDefault(d =>
            d.Source?.OriginalString.Contains("Strings.") == true);
        if (existing != null) dicts.Remove(existing);

        var uri = language == "en"
            ? "/SteamIdlePicker;component/Resources/Strings.en.xaml"
            : "/SteamIdlePicker;component/Resources/Strings.ja.xaml";

        dicts.Add(new ResourceDictionary { Source = new Uri(uri, UriKind.Relative) });
        LanguageChanged?.Invoke(null, EventArgs.Empty);
    }

    public static string Get(string key) =>
        Application.Current.Resources[key] as string ?? key;
}
