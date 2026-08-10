using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.CompilerServices;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.Media;
using Avalonia.Styling;
using Avalonia.Threading;
using ElephantNote.Avalonia.Domain;
using ElephantNote.Avalonia.Muya;
using ElephantNote.Avalonia.Services;

namespace ElephantNote.Avalonia;

public sealed partial class MainWindow : Window, INotifyPropertyChanged
{
    private readonly VaultRepository _repository = new();
    private readonly PreferencesRepository _preferencesRepository;
    private readonly MuyaEditorHost _muyaHost;
    private VaultConfig _config = new();
    private VaultDescriptor? _activeVault;
    private string _currentPath = "";
    private string _openedNotePath = "";
    private string _noteContent = "";
    private string _noteTitle = "";
    private string _searchQuery = "";
    private string _activeSurface = "notes";
    private string _theme = "light";
    private bool _autoSave;
    private bool _muyaEditorAvailable;
    private NativeAcceptanceServer? _acceptanceServer;
    private string _statusText = "Ready";
    private string _errorText = "";
    private static bool IsBackgroundLaunch =>
        string.Equals(Environment.GetEnvironmentVariable("ELEPHANTNOTE_BACKGROUND"), "1", StringComparison.Ordinal);
    private static bool IsAccessibilityAutomation =>
        string.Equals(Environment.GetEnvironmentVariable("ELEPHANTNOTE_ACCESSIBILITY_AUTOMATION"), "1", StringComparison.Ordinal);

    public new event PropertyChangedEventHandler? PropertyChanged;

    public ObservableCollection<VaultEntry> Entries { get; } = [];
    public ObservableCollection<VaultEntry> SidebarEntries { get; } = [];
    public ObservableCollection<SearchResult> SearchResults { get; } = [];
    public IReadOnlyList<string> Themes { get; } = ["light", "dark"];

    public string WindowTitle => "Elephant";
    public string VaultName => _activeVault?.Name ?? "Elephant";
    public string OpenedNotePath => _openedNotePath;
    public string CurrentHeading => string.IsNullOrEmpty(_currentPath) ? "All notes" : _currentPath;
    public string CurrentSubheading => _activeVault is null ? "Choose a local Markdown vault." : _activeVault.Path;
    public string NoteTitle => _noteTitle;
    public string ActiveSurfaceTitle => _activeSurface switch
    {
        "dashboard" => "Dashboard",
        "wiki" => "Wiki",
        "graph" => "Graph",
        "models" => "AI models",
        "chat" => "Chat",
        _ => "Native workspace"
    };
    public string ActiveSurfaceMessage => $"The {_activeSurface} surface is not migrated to a native runtime yet. It remains available in the existing web/Tauri client until its real domain and persistence adapter are ported.";
    public bool IsEmptyVaultVisible => _activeVault is null;
    public bool IsWorkspaceVisible => _activeVault is not null && _activeSurface == "notes" && string.IsNullOrEmpty(_openedNotePath);
    public bool IsEditorVisible => _activeVault is not null && _activeSurface == "notes" && !string.IsNullOrEmpty(_openedNotePath);
    public bool IsSearchVisible => _activeVault is not null && _activeSurface == "search";
    public bool IsSettingsVisible => _activeSurface == "settings";
    public bool IsUnsupportedSurfaceVisible => _activeVault is not null && _activeSurface is not ("notes" or "search" or "settings");
    public bool IsMuyaEditorVisible => IsEditorVisible && _muyaEditorAvailable;
    public bool IsNativeEditorFallbackVisible => IsEditorVisible && !_muyaEditorAvailable;
    public bool HasError => !string.IsNullOrWhiteSpace(_errorText);
    public string StatusText { get => _statusText; private set => SetField(ref _statusText, value); }
    public string ErrorText { get => _errorText; private set { if (SetField(ref _errorText, value)) OnPropertyChanged(nameof(HasError)); } }

    public string NoteContent { get => _noteContent; set => SetField(ref _noteContent, value); }
    public string SearchQuery { get => _searchQuery; set => SetField(ref _searchQuery, value); }

    public new string Theme
    {
        get => _theme;
        set
        {
            var normalized = value is "dark" ? "dark" : "light";
            if (!SetField(ref _theme, normalized)) return;
            RequestedThemeVariant = normalized == "dark" ? ThemeVariant.Dark : ThemeVariant.Light;
            ApplyThemeResources();
            SavePreferences();
        }
    }

    public bool AutoSave
    {
        get => _autoSave;
        set
        {
            if (!SetField(ref _autoSave, value)) return;
            SavePreferences();
        }
    }

    public MainWindow()
    {
        InitializeComponent();
        if (IsBackgroundLaunch)
        {
            ShowActivated = false;
            if (IsAccessibilityAutomation)
                Position = new PixelPoint(-4000, -4000);
            else
                Opacity = 0;
        }
        DataContext = this;
        _preferencesRepository = new PreferencesRepository(_repository.ConfigDirectory);
        var muyaPath = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "Muya", "index.html");
        _muyaHost = new MuyaEditorHost(new MuyaEditorHostOptions { LocalMuyaResource = new Uri(muyaPath) });
        _muyaHost.ContentChanged += MuyaContentChanged;
        _muyaHost.SaveRequested += MuyaSaveRequested;
        _muyaHost.Error += MuyaError;
        var preferences = _preferencesRepository.Load();
        _theme = preferences.Theme;
        _autoSave = preferences.AutoSave;
        RequestedThemeVariant = _theme == "dark" ? ThemeVariant.Dark : ThemeVariant.Light;
        ApplyThemeResources();
        Opened += OnOpened;
        Closed += async (_, _) =>
        {
            if (_acceptanceServer is not null) await _acceptanceServer.DisposeAsync();
            await _muyaHost.DisposeAsync();
        };
    }

    private async void OnOpened(object? sender, EventArgs e)
    {
        await RunActionAsync("app.load", LoadAsync);
        var socketPath = Environment.GetEnvironmentVariable("ELEPHANTNOTE_ACCEPTANCE_SOCKET");
        if (string.IsNullOrWhiteSpace(socketPath)) return;

        var requestId = AppLog.Start("acceptance.server", $"socket={socketPath}");
        var timer = Stopwatch.StartNew();
        try
        {
            _acceptanceServer = NativeAcceptanceServer.Start(socketPath, HandleAcceptanceCommandAsync);
            AppLog.Complete("acceptance.server", requestId, timer, "ready=true");
        }
        catch (Exception error)
        {
            AppLog.Fail("acceptance.server", requestId, timer, error);
            ErrorText = $"Native acceptance channel could not start: {error.Message}";
        }
    }

    private Task<object?> HandleAcceptanceCommandAsync(JsonElement request)
    {
        var completion = new TaskCompletionSource<object?>(TaskCreationOptions.RunContinuationsAsynchronously);
        Dispatcher.UIThread.Post(async () =>
        {
            try { completion.SetResult(await ExecuteAcceptanceCommandAsync(request)); }
            catch (Exception error) { completion.SetException(error); }
        });
        return completion.Task;
    }

    private async Task<object?> ExecuteAcceptanceCommandAsync(JsonElement request)
    {
        var command = request.TryGetProperty("command", out var rawCommand)
            ? rawCommand.GetString()
            : null;
        var payload = request.TryGetProperty("payload", out var rawPayload) ? rawPayload : default;
        return command switch
        {
            "state" => AcceptanceSnapshot(),
            "open-note" => await AcceptanceOpenNoteAsync(RequirePayloadString(payload, "path")),
            "append-text" => await AcceptanceAppendTextAsync(RequirePayloadString(payload, "text")),
            "set-content" => await AcceptanceSetContentAsync(RequirePayloadString(payload, "content")),
            "save-note" => await AcceptanceSaveNoteAsync(),
            "back-to-notes" => await AcceptanceBackToNotesAsync(),
            "focus-editor" => new { focused = await _muyaHost.FocusAsync(), muyaReady = _muyaEditorAvailable },
            _ => throw new InvalidDataException($"Unknown native acceptance command '{command}'.")
        };
    }

    private async Task<object> AcceptanceOpenNoteAsync(string relativePath)
    {
        if (Path.IsPathRooted(relativePath) || !relativePath.EndsWith(".md", StringComparison.OrdinalIgnoreCase))
            throw new InvalidDataException("Acceptance notes must be relative Markdown paths.");
        var entry = new VaultEntry(
            relativePath.Replace('\\', '/'),
            Path.GetFileNameWithoutExtension(relativePath),
            "",
            false,
            DateTime.UtcNow);
        await OpenEntryAsync(entry);
        return AcceptanceSnapshot();
    }

    private async Task<object> AcceptanceAppendTextAsync(string text)
    {
        if (!_muyaEditorAvailable) throw new InvalidOperationException("Muya is not ready for acceptance editing.");
        await _muyaHost.SetMarkdownAsync(($"{_muyaHost.Content ?? NoteContent}{text}"));
        return AcceptanceSnapshot();
    }

    private async Task<object> AcceptanceSetContentAsync(string content)
    {
        if (!_muyaEditorAvailable) throw new InvalidOperationException("Muya is not ready for acceptance editing.");
        await _muyaHost.SetMarkdownAsync(content);
        return AcceptanceSnapshot();
    }

    private async Task<object> AcceptanceSaveNoteAsync()
    {
        await SaveCurrentNoteAsync();
        return AcceptanceSnapshot();
    }

    private async Task<object> AcceptanceBackToNotesAsync()
    {
        await ReturnToNotesAsync();
        return AcceptanceSnapshot();
    }

    private object AcceptanceSnapshot() => new
    {
        activeSurface = _activeSurface,
        openedNotePath = _openedNotePath,
        muyaReady = _muyaEditorAvailable,
        contentLength = (_muyaHost.Content ?? NoteContent).Length,
        status = StatusText,
        error = HasError ? ErrorText : null
    };

    private static string RequirePayloadString(JsonElement payload, string propertyName)
    {
        if (payload.ValueKind != JsonValueKind.Object ||
            !payload.TryGetProperty(propertyName, out var value) ||
            value.ValueKind != JsonValueKind.String)
            throw new InvalidDataException($"Acceptance payload requires string '{propertyName}'.");
        return value.GetString() ?? throw new InvalidDataException($"Acceptance payload '{propertyName}' is null.");
    }

    private async Task LoadAsync()
    {
        _config = await _repository.LoadConfigAsync();
        _activeVault = _config.Vaults.FirstOrDefault(item => item.Id == _config.ActiveVaultId && Directory.Exists(item.Path));
        await RefreshEntriesAsync();
        await InitializeMuyaAsync();
        NotifyWorkspace();
        StatusText = _activeVault is null ? "No vault selected" : $"Loaded {_activeVault.Name}";
    }

    private async Task InitializeMuyaAsync()
    {
        if (!MuyaResourceLocator.Exists(_muyaHost.LocalMuyaResource!))
        {
            _muyaEditorAvailable = false;
            return;
        }

        var requestId = AppLog.Start("muya.attach");
        var timer = Stopwatch.StartNew();
        try
        {
            var attached = await _muyaHost.AttachNativeWebViewAsync(MuyaWebView);
            _muyaEditorAvailable = attached && await _muyaHost.WaitUntilReadyAsync();
            AppLog.Complete("muya.attach", requestId, timer, $"available={_muyaEditorAvailable}");
            if (!_muyaEditorAvailable)
            {
                ErrorText = "The Muya bundle is not available; using the native Markdown fallback.";
            }
        }
        catch (Exception error)
        {
            _muyaEditorAvailable = false;
            AppLog.Fail("muya.attach", requestId, timer, error);
            ErrorText = $"Muya could not start: {error.Message}";
        }
    }

    private async Task RefreshEntriesAsync()
    {
        Entries.Clear();
        SidebarEntries.Clear();
        if (_activeVault is null) return;
        var items = await _repository.ListEntriesAsync(_activeVault, _currentPath);
        foreach (var item in items) Entries.Add(item);
        if (string.IsNullOrEmpty(_currentPath))
        {
            foreach (var item in items) SidebarEntries.Add(item);
        }
        StatusText = $"{items.Count} entries · {_activeVault.Name}";
    }

    private async void ChooseVaultClick(object? sender, RoutedEventArgs e)
    {
        await RunActionAsync("vault.choose", async () =>
        {
            var topLevel = TopLevel.GetTopLevel(this);
            if (topLevel?.StorageProvider is null) throw new InvalidOperationException("The native storage provider is unavailable.");
            var folders = await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions { AllowMultiple = false, Title = "Open Elephant vault" });
            var selected = folders.FirstOrDefault();
            if (selected?.Path is null) return;
            await SelectVaultAsync(selected.Path.LocalPath);
        });
    }

    private async void CreateVaultClick(object? sender, RoutedEventArgs e)
    {
        await RunActionAsync("vault.create", async () =>
        {
            var documents = Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments);
            if (string.IsNullOrWhiteSpace(documents)) documents = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            var path = System.IO.Path.Combine(documents, "ElephantVault");
            Directory.CreateDirectory(path);
            await SelectVaultAsync(path);
        });
    }

    private async Task SelectVaultAsync(string path)
    {
        _activeVault = await _repository.SelectVaultAsync(path);
        _config = await _repository.LoadConfigAsync();
        _currentPath = "";
        _openedNotePath = "";
        await RefreshEntriesAsync();
        SetSurface("notes");
    }

    private async void AllNotesClick(object? sender, RoutedEventArgs e)
    {
        await RunActionAsync("workspace.all-notes", async () =>
        {
            _currentPath = "";
            _openedNotePath = "";
            SetSurface("notes");
            await RefreshEntriesAsync();
        });
    }

    private async void SidebarSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        var entry = e.AddedItems.OfType<VaultEntry>().FirstOrDefault();
        if (entry is null) return;
        await OpenEntryAsync(entry);
    }

    private async void EntrySelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        var entry = e.AddedItems.OfType<VaultEntry>().FirstOrDefault();
        if (entry is null) return;
        await OpenEntryAsync(entry);
    }

    private async Task OpenEntryAsync(VaultEntry entry)
    {
        if (_activeVault is null) return;
        await RunActionAsync(entry.IsDirectory ? "folder.open" : "note.open", async () =>
        {
            if (entry.IsDirectory)
            {
                _currentPath = entry.Path;
                _openedNotePath = "";
                SetSurface("notes");
                await RefreshEntriesAsync();
                return;
            }
            _openedNotePath = entry.Path;
            _noteContent = await _repository.ReadNoteAsync(_activeVault, entry.Path);
            _noteTitle = entry.Title;
            await _muyaHost.OpenDocumentAsync(new MuyaOpenDocumentRequest(
                entry.Path,
                _noteContent,
                entry.Path,
                entry.Title));
            SetSurface("notes");
            OnPropertyChanged(nameof(NoteContent));
            OnPropertyChanged(nameof(NoteTitle));
            OnPropertyChanged(nameof(OpenedNotePath));
        });
    }

    private async void CreateNoteClick(object? sender, RoutedEventArgs e)
    {
        await RunActionAsync("note.create", async () =>
        {
            if (_activeVault is null) return;
            var entry = await _repository.CreateNoteAsync(_activeVault, _currentPath);
            await OpenEntryAsync(entry);
            await RefreshEntriesAsync();
        });
    }

    private async void CreateFolderClick(object? sender, RoutedEventArgs e)
    {
        await RunActionAsync("folder.create", async () =>
        {
            if (_activeVault is null) return;
            var relative = System.IO.Path.Combine(_currentPath, "New Folder").Replace('\\', '/');
            await _repository.CreateFolderAsync(_activeVault, relative);
            await RefreshEntriesAsync();
        });
    }

    private async void SaveNoteClick(object? sender, RoutedEventArgs e)
    {
        await RunActionAsync("note.save", SaveCurrentNoteAsync);
    }

    private async Task SaveCurrentNoteAsync()
    {
        if (_activeVault is null || string.IsNullOrWhiteSpace(_openedNotePath)) return;
        var content = _muyaEditorAvailable ? _muyaHost.Content ?? NoteContent : NoteContent;
        NoteContent = content;
        await _repository.SaveNoteAsync(_activeVault, _openedNotePath, content);
        StatusText = $"Saved {OpenedNotePath}";
        await RefreshEntriesAsync();
    }

    private async void MuyaSaveRequested(object? sender, MuyaSaveRequestedEventArgs args)
    {
        if (args.Request.DocumentId != _openedNotePath) return;
        await RunActionAsync("note.save", async () =>
        {
            if (_activeVault is null) return;
            NoteContent = args.Request.Content;
            await _repository.SaveNoteAsync(_activeVault, args.Request.DocumentId, args.Request.Content);
            StatusText = $"Saved {args.Request.DocumentId}";
            await RefreshEntriesAsync();
        });
    }

    private async void BackToNotesClick(object? sender, RoutedEventArgs e)
    {
        _openedNotePath = "";
        SetSurface("notes");
        await RefreshEntriesAsync();
    }

    private async void SearchClick(object? sender, RoutedEventArgs e) => await ExecuteSearchAsync();

    private async void SearchKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key == Key.Enter) await ExecuteSearchAsync();
    }

    private async Task ExecuteSearchAsync()
    {
        await RunActionAsync("search.query", async () =>
        {
            if (_activeVault is null) return;
            SetSurface("search");
            SearchResults.Clear();
            foreach (var result in await _repository.SearchAsync(_activeVault, SearchQuery)) SearchResults.Add(result);
            StatusText = $"{SearchResults.Count} results";
        });
    }

    private void SettingsClick(object? sender, RoutedEventArgs e) => SetSurface("settings");

    private void RailClick(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: string surface }) return;
        if (surface == "notes")
        {
            _ = ReturnToNotesAsync();
            return;
        }
        if (surface == "settings")
        {
            SetSurface("settings");
            return;
        }
        _openedNotePath = "";
        SetSurface(surface);
    }

    private async Task ReturnToNotesAsync()
    {
        _currentPath = "";
        _openedNotePath = "";
        SetSurface("notes");
        await RefreshEntriesAsync();
    }

    private void SetSurface(string surface)
    {
        _activeSurface = surface;
        NotifyWorkspace();
    }

    private void NotifyWorkspace()
    {
        OnPropertyChanged(nameof(VaultName));
        OnPropertyChanged(nameof(CurrentHeading));
        OnPropertyChanged(nameof(CurrentSubheading));
        OnPropertyChanged(nameof(IsEmptyVaultVisible));
        OnPropertyChanged(nameof(IsWorkspaceVisible));
        OnPropertyChanged(nameof(IsEditorVisible));
        OnPropertyChanged(nameof(IsSearchVisible));
        OnPropertyChanged(nameof(IsSettingsVisible));
        OnPropertyChanged(nameof(IsUnsupportedSurfaceVisible));
        OnPropertyChanged(nameof(IsMuyaEditorVisible));
        OnPropertyChanged(nameof(IsNativeEditorFallbackVisible));
        OnPropertyChanged(nameof(ActiveSurfaceTitle));
        OnPropertyChanged(nameof(ActiveSurfaceMessage));
    }

    private async Task RunActionAsync(string action, Func<Task> work)
    {
        var requestId = AppLog.Start(action);
        var timer = Stopwatch.StartNew();
        ErrorText = "";
        try
        {
            await work();
            AppLog.Complete(action, requestId, timer);
        }
        catch (Exception error)
        {
            AppLog.Fail(action, requestId, timer, error);
            ErrorText = error.Message;
            StatusText = $"{action} failed";
        }
    }

    private void SavePreferences()
    {
        if (_preferencesRepository is null) return;
        try { _preferencesRepository.Save(new AvaloniaPreferences { Theme = Theme, AutoSave = AutoSave }); }
        catch (Exception error) { ErrorText = error.Message; }
    }

    private void ApplyThemeResources()
    {
        if (Application.Current is not { } application) return;
        var dark = _theme == "dark";
        var palette = dark
            ? new Dictionary<string, string>
            {
                ["AppBackgroundBrush"] = "#0F141D",
                ["AppSurfaceBrush"] = "#141A24",
                ["AppSidebarBrush"] = "#101722",
                ["AppSoftBrush"] = "#1B2432",
                ["AppSelectionBrush"] = "#263F63",
                ["AppBorderBrush"] = "#283244",
                ["AppTextBrush"] = "#EEF3FB",
                ["AppMutedBrush"] = "#98A3B6",
                ["AppPrimaryBrush"] = "#5EA1FF",
                ["AppDangerBrush"] = "#FF6B7A"
            }
            : new Dictionary<string, string>
            {
                ["AppBackgroundBrush"] = "#F7F9FC",
                ["AppSurfaceBrush"] = "#FFFFFF",
                ["AppSidebarBrush"] = "#EDF2F7",
                ["AppSoftBrush"] = "#E9EFF7",
                ["AppSelectionBrush"] = "#DCE9FF",
                ["AppBorderBrush"] = "#C5CFDD",
                ["AppTextBrush"] = "#101828",
                ["AppMutedBrush"] = "#667085",
                ["AppPrimaryBrush"] = "#2563EB",
                ["AppDangerBrush"] = "#DC2626"
            };
        foreach (var (key, color) in palette) application.Resources[key] = new SolidColorBrush(Color.Parse(color));
    }

    private void MuyaContentChanged(object? sender, MuyaContentChangedEventArgs args)
    {
        if (args.DocumentId != _openedNotePath) return;
        NoteContent = args.Content;
        if (args.IsUserEdit)
        {
            var rustSuffix = args.RustRevision is null ? "" : $" · Rust {args.RustRevision}";
            StatusText = $"Unsaved changes{rustSuffix}";
        }
    }

    private void MuyaError(object? sender, MuyaEditorErrorEventArgs args)
    {
        ErrorText = $"Muya: {args.Error.Message}";
        var requestId = AppLog.Start("muya.runtime", $"code={args.Error.Code}");
        var timer = Stopwatch.StartNew();
        AppLog.Fail("muya.runtime", requestId, timer, new InvalidOperationException(args.Error.Message));
    }

    private bool SetField<T>(ref T field, T value, [CallerMemberName] string? propertyName = null)
    {
        if (EqualityComparer<T>.Default.Equals(field, value)) return false;
        field = value;
        OnPropertyChanged(propertyName);
        return true;
    }

    private void OnPropertyChanged([CallerMemberName] string? propertyName = null) => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
}
