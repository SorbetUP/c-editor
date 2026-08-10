namespace ElephantNote.Avalonia.Domain;

public sealed class MarkdownDocument
{
    private MarkdownDocument(string source, string body, MarkdownFrontmatter? frontmatter)
    {
        Source = source;
        Body = body;
        Frontmatter = frontmatter;
    }

    public string Source { get; }
    public string Body { get; }
    public MarkdownFrontmatter? Frontmatter { get; }
    public string Title => Frontmatter?.Title ?? FindHeading(Body) ?? "";
    public string Type => Frontmatter?.Type ?? "note";
    public IReadOnlyList<string> Tags => Frontmatter?.Tags ?? [];

    public static MarkdownDocument Parse(string markdown)
    {
        ArgumentNullException.ThrowIfNull(markdown);
        if (!TryReadFrontmatter(markdown, out var prefixEnd, out var raw))
        {
            return new MarkdownDocument(markdown, markdown, null);
        }

        var prefix = markdown[..prefixEnd];
        return new MarkdownDocument(markdown, markdown[prefixEnd..], new MarkdownFrontmatter(raw, prefix));
    }

    /// <summary>Returns a new document while retaining the original frontmatter bytes.</summary>
    public MarkdownDocument WithBody(string body)
    {
        ArgumentNullException.ThrowIfNull(body);
        var source = Frontmatter is null ? body : Frontmatter.Prefix + body;
        return new MarkdownDocument(source, body, Frontmatter);
    }

    public string ToMarkdown() => Source;

    private static bool TryReadFrontmatter(string markdown, out int prefixEnd, out string raw)
    {
        prefixEnd = 0;
        raw = "";
        var start = markdown.StartsWith('\uFEFF') ? 1 : 0;
        if (!TryReadLine(markdown, start, out var firstLineEnd, out var firstLine) || firstLine.TrimEnd('\r') != "---") return false;

        var contentStart = firstLineEnd;
        var cursor = contentStart;
        while (TryReadLine(markdown, cursor, out var lineEnd, out var line))
        {
            if (line.TrimEnd('\r') == "---")
            {
                raw = markdown[contentStart..cursor];
                prefixEnd = lineEnd;
                return true;
            }
            cursor = lineEnd;
        }
        return false;
    }

    private static bool TryReadLine(string text, int start, out int next, out string line)
    {
        if (start > text.Length)
        {
            next = start;
            line = "";
            return false;
        }
        var newline = text.IndexOf('\n', start);
        if (newline < 0)
        {
            next = text.Length;
            line = text[start..];
            return true;
        }
        next = newline + 1;
        line = text[start..newline];
        return true;
    }

    private static string? FindHeading(string body) =>
        body.Split('\n').Select(line => line.TrimEnd('\r')).FirstOrDefault(line => line.StartsWith("# "))?[2..].Trim();
}
