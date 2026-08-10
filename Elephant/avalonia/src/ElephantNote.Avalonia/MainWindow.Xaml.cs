using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using ElephantNote.Avalonia.Muya;

namespace ElephantNote.Avalonia;

public sealed partial class MainWindow
{
    private void InitializeComponent() => AvaloniaXamlLoader.Load(this);

    private AvaloniaMuyaWebView MuyaWebView =>
        this.FindControl<AvaloniaMuyaWebView>("MuyaWebView")
        ?? throw new InvalidOperationException("The Muya WebView control is missing from MainWindow.axaml.");

}
