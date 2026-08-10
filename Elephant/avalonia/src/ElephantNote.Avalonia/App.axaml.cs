using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;

namespace ElephantNote.Avalonia;

public sealed class App : Application
{
    private static bool IsBackgroundLaunch =>
        string.Equals(Environment.GetEnvironmentVariable("ELEPHANTNOTE_BACKGROUND"), "1", StringComparison.Ordinal);

    public override void Initialize() => AvaloniaXamlLoader.Load(this);

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            if (IsBackgroundLaunch)
                desktop.ShutdownMode = ShutdownMode.OnExplicitShutdown;
            desktop.MainWindow = new MainWindow();
        }

        base.OnFrameworkInitializationCompleted();
    }
}
