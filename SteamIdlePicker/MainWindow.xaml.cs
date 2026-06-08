using System;
using System.ComponentModel;
using System.Windows;
using SteamIdlePicker.ViewModels;

namespace SteamIdlePicker;

public partial class MainWindow : Window
{
    public MainViewModel ViewModel { get; }

    public MainWindow()
    {
        InitializeComponent();
        ViewModel = new MainViewModel();
        DataContext = ViewModel;
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
