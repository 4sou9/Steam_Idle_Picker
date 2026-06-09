using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;
using SteamIdlePicker.ViewModels;

namespace SteamIdlePicker;

public partial class MainWindow : Window
{
    [DllImport("dwmapi.dll")]
    private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref int attrValue, int attrSize);

    public MainViewModel ViewModel { get; }

    public MainWindow()
    {
        InitializeComponent();
        ViewModel = new MainViewModel();
        DataContext = ViewModel;
    }

    protected override void OnSourceInitialized(EventArgs e)
    {
        base.OnSourceInitialized(e);
        if (App.IsDarkMode)
        {
            var hwnd = new WindowInteropHelper(this).Handle;
            int dark = 1;
            DwmSetWindowAttribute(hwnd, 20, ref dark, 4); // DWMWA_USE_IMMERSIVE_DARK_MODE
        }
    }

    protected override void OnStateChanged(EventArgs e)
    {
        if (WindowState == WindowState.Minimized)
            Hide();
        base.OnStateChanged(e);
    }

    protected override void OnClosing(CancelEventArgs e)
    {
        ViewModel.Cleanup();
        System.Windows.Application.Current.Shutdown();
        base.OnClosing(e);
    }
}
