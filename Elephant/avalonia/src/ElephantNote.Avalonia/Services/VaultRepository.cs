using System.Diagnostics;
using System.Text.Json;
using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Services;

public sealed class VaultRepository
{
    private const int PreviewLimit = 16 * 1024;
    private const int SearchLimit = 50;
    private readonly string _configDirectory;
    private readonly string _configPath;
    private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true, PropertyNameCaseInsensitive = true };

    public VaultRepository()
    {
        _configDirectory = ResolveConfigDirectory();
        _configPath = System.IO.Path.Combine(_configDirectory, "tauri-vaults.json");
        AppLog.Configure(_configDirectory);
    }

    public string ConfigDirectory => _configDirectory;

    public async Task<VaultConfig> LoadConfigAsync()
    {
        var requestId = AppLog.Start("vault.config.load", $"configFile={_configPath}");
        var timer = Stopwatch.StartNew();
        try
        {
            if (!File.Exists(_configPath))
            {
                var empty = new VaultConfig();
                AppLog.Complete("vault.config.load", requestId, timer, "vaultCount=0");
                return empty;
            }
            await using var stream = File.OpenRead(_configPath);
            var config = await JsonSerializer.DeserializeAsync<VaultConfig>(stream, JsonOptions) ?? new VaultConfig();
            AppLog.Complete("vault.config.load", requestId, timer, $"vaultCount={config.Vaults.Count}");
            return config;
        }
        catch (Exception error)
        {
            AppLog.Fail("vault.config.load", requestId, timer, error);
            throw new InvalidOperationException($"Unable to load Elephant vault configuration: {error.Message}", error);
        }
    }

    public async Task SaveConfigAsync(VaultConfig config)
    {
        var requestId = AppLog.Start("vault.config.save", $"configFile={_configPath}");
        var timer = Stopwatch.StartNew();
        try
        {
            Directory.CreateDirectory(_configDirectory);
            await WriteAtomicallyAsync(_configPath, JsonSerializer.Serialize(config, JsonOptions));
            AppLog.Complete("vault.config.save", requestId, timer, $"vaultCount={config.Vaults.Count}");
        }
        catch (Exception error)
        {
            AppLog.Fail("vault.config.save", requestId, timer, error);
            throw;
        }
    }

    public async Task<VaultDescriptor?> GetActiveVaultAsync()
    {
        var config = await LoadConfigAsync();
        return config.Vaults.FirstOrDefault(v => v.Id == config.ActiveVaultId && Directory.Exists(v.Path));
    }

    public async Task<VaultDescriptor> SelectVaultAsync(string path)
    {
        var fullPath = System.IO.Path.GetFullPath(path);
        if (!Directory.Exists(fullPath)) throw new DirectoryNotFoundException($"Vault directory does not exist: {fullPath}");
        var config = await LoadConfigAsync();
        var normalized = fullPath.Replace(System.IO.Path.DirectorySeparatorChar, '/');
        var vault = config.Vaults.FirstOrDefault(item => string.Equals(item.Path, normalized, StringComparison.OrdinalIgnoreCase));
        if (vault is null)
        {
            var name = new DirectoryInfo(fullPath).Name;
            var id = SlugId(name);
            var suffix = 2;
            while (config.Vaults.Any(item => item.Id == id)) id = $"{SlugId(name)}-{suffix++}";
            vault = new VaultDescriptor(id, name, normalized, LastOpenedAt: DateTimeOffset.UtcNow.ToUnixTimeSeconds().ToString());
            config.Vaults.Add(vault);
        }
        config.ActiveVaultId = vault.Id;
        await SaveConfigAsync(config);
        return vault;
    }

    public async Task<IReadOnlyList<VaultEntry>> ListEntriesAsync(VaultDescriptor vault, string relativePath = "")
    {
        var directory = EnsureInside(vault.Path, relativePath);
        if (!Directory.Exists(directory)) throw new DirectoryNotFoundException(directory);
        var entries = new List<VaultEntry>();
        foreach (var info in new DirectoryInfo(directory).EnumerateFileSystemInfos())
        {
            if (IsIgnored(info)) continue;
            if (info is DirectoryInfo childDirectory)
            {
                entries.Add(new VaultEntry(Relative(vault.Path, childDirectory.FullName), childDirectory.Name, "Folder", true, childDirectory.LastWriteTimeUtc));
                continue;
            }
            if (info is FileInfo file && file.Extension.Equals(".md", StringComparison.OrdinalIgnoreCase))
            {
                var preview = ReadPreview(file.FullName, file.Name[..^file.Extension.Length]);
                entries.Add(new VaultEntry(Relative(vault.Path, file.FullName), preview.Title, preview.Excerpt, false, file.LastWriteTimeUtc));
            }
        }
        return await Task.FromResult(entries.OrderByDescending(item => item.IsDirectory).ThenBy(item => item.Title, StringComparer.OrdinalIgnoreCase).ToArray());
    }

    public async Task<string> ReadNoteAsync(VaultDescriptor vault, string relativePath)
    {
        var path = EnsureInside(vault.Path, relativePath);
        if (!path.EndsWith(".md", StringComparison.OrdinalIgnoreCase)) throw new InvalidDataException("Only Markdown notes can be opened.");
        return await File.ReadAllTextAsync(path);
    }

    public async Task SaveNoteAsync(VaultDescriptor vault, string relativePath, string content)
    {
        var path = EnsureInside(vault.Path, relativePath);
        if (!path.EndsWith(".md", StringComparison.OrdinalIgnoreCase)) throw new InvalidDataException("Only Markdown notes can be saved.");
        Directory.CreateDirectory(System.IO.Path.GetDirectoryName(path)!);
        await WriteAtomicallyAsync(path, content);
    }

    public async Task<VaultEntry> CreateNoteAsync(VaultDescriptor vault, string relativeDirectory = "")
    {
        var directory = EnsureInside(vault.Path, relativeDirectory);
        Directory.CreateDirectory(directory);
        var path = UniquePath(System.IO.Path.Combine(directory, "Untitled.md"));
        await WriteAtomicallyAsync(path, string.Empty);
        return new VaultEntry(Relative(vault.Path, path), "", "", false, DateTime.UtcNow);
    }

    public async Task<VaultEntry> CreateFolderAsync(VaultDescriptor vault, string relativeDirectory = "")
    {
        var parent = EnsureInside(vault.Path, System.IO.Path.GetDirectoryName(relativeDirectory) ?? "");
        var requested = System.IO.Path.GetFileName(relativeDirectory.TrimEnd('/', '\\'));
        if (string.IsNullOrWhiteSpace(requested)) requested = "New Folder";
        var path = UniquePath(System.IO.Path.Combine(parent, requested));
        Directory.CreateDirectory(path);
        return await Task.FromResult(new VaultEntry(Relative(vault.Path, path), requested, "Folder", true, DateTime.UtcNow));
    }

    public async Task<IReadOnlyList<SearchResult>> SearchAsync(VaultDescriptor vault, string query)
    {
        var normalized = query.Trim();
        if (normalized.Length == 0) return [];
        var results = new List<SearchResult>();
        foreach (var file in EnumerateMarkdown(vault.Path))
        {
            var content = await File.ReadAllTextAsync(file);
            var path = Relative(vault.Path, file);
            var pathHit = path.Contains(normalized, StringComparison.OrdinalIgnoreCase);
            var contentHit = content.Contains(normalized, StringComparison.OrdinalIgnoreCase);
            if (!pathHit && !contentHit) continue;
            results.Add(new SearchResult(path, System.IO.Path.GetFileNameWithoutExtension(file), Excerpt(content), pathHit ? 2 : 1));
            if (results.Count >= SearchLimit) break;
        }
        return results.OrderByDescending(item => item.Score).ThenBy(item => item.Title, StringComparer.OrdinalIgnoreCase).ToArray();
    }

    private static IEnumerable<string> EnumerateMarkdown(string root)
    {
        foreach (var file in Directory.EnumerateFiles(root, "*.md", SearchOption.AllDirectories))
        {
            var info = new FileInfo(file);
            if (!IsIgnored(info)) yield return file;
        }
    }

    private static (string Title, string Excerpt) ReadPreview(string path, string fallback)
    {
        var content = File.ReadAllText(path);
        var lines = content[..Math.Min(content.Length, PreviewLimit)].Split('\n', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        var headingLine = lines.FirstOrDefault(line => line.StartsWith("# ", StringComparison.Ordinal));
        var heading = headingLine is null ? null : headingLine[2..].Trim();
        var excerpt = string.Join(' ', lines.Where(line => !line.StartsWith("# ", StringComparison.Ordinal)).Take(3)).Trim();
        return (string.IsNullOrWhiteSpace(heading) ? fallback : heading, excerpt.Length > 240 ? excerpt[..240] + "…" : excerpt);
    }

    private static string Excerpt(string content)
    {
        var excerpt = string.Join(' ', content.Split('\n', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Take(3));
        return excerpt.Length > 600 ? excerpt[..600] + "…" : excerpt;
    }

    private static bool IsIgnored(FileSystemInfo info) => info.Name.StartsWith(".", StringComparison.Ordinal) || info.Name.EndsWith("~", StringComparison.Ordinal) || info.Name.EndsWith(".tmp", StringComparison.OrdinalIgnoreCase) || info.Attributes.HasFlag(FileAttributes.ReparsePoint);

    private static string EnsureInside(string root, string relative)
    {
        var fullRoot = System.IO.Path.GetFullPath(root).TrimEnd(System.IO.Path.DirectorySeparatorChar) + System.IO.Path.DirectorySeparatorChar;
        var cleanRelative = relative.Replace('\\', System.IO.Path.DirectorySeparatorChar).Replace('/', System.IO.Path.DirectorySeparatorChar);
        var fullPath = System.IO.Path.GetFullPath(System.IO.Path.Combine(fullRoot, cleanRelative));
        if (!fullPath.StartsWith(fullRoot, StringComparison.OrdinalIgnoreCase)) throw new UnauthorizedAccessException("The requested path leaves the active vault.");
        RejectReparsePoints(System.IO.Path.GetFullPath(root), fullPath);
        return fullPath;
    }

    private static void RejectReparsePoints(string root, string path)
    {
        var current = path;
        while (current.Length >= root.Length)
        {
            if ((File.Exists(current) || Directory.Exists(current)) && File.GetAttributes(current).HasFlag(FileAttributes.ReparsePoint))
            {
                throw new UnauthorizedAccessException("The requested path crosses a symbolic link or reparse point.");
            }
            if (string.Equals(current, root, StringComparison.OrdinalIgnoreCase)) break;
            current = Directory.GetParent(current)?.FullName ?? root;
        }
    }

    private static async Task WriteAtomicallyAsync(string path, string content)
    {
        var temporary = $"{path}.{Guid.NewGuid():N}.tmp";
        try
        {
            await File.WriteAllTextAsync(temporary, content);
            File.Move(temporary, path, overwrite: true);
        }
        finally
        {
            if (File.Exists(temporary)) File.Delete(temporary);
        }
    }

    private static string Relative(string root, string path) => System.IO.Path.GetRelativePath(root, path).Replace('\\', '/');

    private static string UniquePath(string path)
    {
        if (!File.Exists(path) && !Directory.Exists(path)) return path;
        var directory = System.IO.Path.GetDirectoryName(path)!;
        var stem = System.IO.Path.GetFileNameWithoutExtension(path);
        var extension = System.IO.Path.GetExtension(path);
        for (var index = 2; index < 1000; index++)
        {
            var candidate = System.IO.Path.Combine(directory, $"{stem} {index}{extension}");
            if (!File.Exists(candidate) && !Directory.Exists(candidate)) return candidate;
        }
        throw new IOException("Unable to allocate a unique vault entry name.");
    }

    private static string SlugId(string value)
    {
        var slug = string.Concat(value.Trim().ToLowerInvariant().Select(ch => char.IsLetterOrDigit(ch) ? ch : '-')).Trim('-');
        return string.IsNullOrWhiteSpace(slug) ? "vault" : slug;
    }

    private static string ResolveConfigDirectory()
    {
        var overrideDirectory = Environment.GetEnvironmentVariable("ELEPHANTNOTE_CONFIG_DIR");
        if (!string.IsNullOrWhiteSpace(overrideDirectory)) return System.IO.Path.GetFullPath(overrideDirectory);
        var baseDirectory = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        if (string.IsNullOrWhiteSpace(baseDirectory)) baseDirectory = System.IO.Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".config");
        var candidates = new[] { System.IO.Path.Combine(baseDirectory, "com.elephantnote.app"), System.IO.Path.Combine(baseDirectory, "Elephant") };
        return candidates.FirstOrDefault(directory => File.Exists(System.IO.Path.Combine(directory, "tauri-vaults.json"))) ?? candidates[0];
    }
}
