using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Core.Tests.Domain;

public sealed class MarkdownDocumentTests
{
    [Fact]
    public void ParsesMetadataAndRoundTripsFrontmatterExactly()
    {
        const string markdown = "---\r\ntitle: \"Research\"\r\ntype: article\r\ntags: [\"llm\", \"needs, review\"]\r\ncustom: keep-me\r\n---\r\n\r\n# Heading\r\nBody\r\n";
        var document = MarkdownDocument.Parse(markdown);

        Assert.Equal("Research", document.Title);
        Assert.Equal("article", document.Type);
        Assert.Equal(new[] { "llm", "needs, review" }, document.Tags.ToArray());
        Assert.Equal("keep-me", document.Frontmatter!.Get("custom"));
        Assert.Equal(markdown, document.ToMarkdown());
        Assert.Equal(markdown, document.WithBody(document.Body).ToMarkdown());
    }

    [Fact]
    public void BodyEditsDoNotDropFrontmatterOrChangeItsLineEndings()
    {
        const string markdown = "---\r\ntitle: Note\r\nunknown: preserved\r\n---\r\n\r\nOriginal";
        var document = MarkdownDocument.Parse(markdown).WithBody("\r\nChanged");

        Assert.StartsWith("---\r\ntitle: Note\r\nunknown: preserved\r\n---\r\n", document.ToMarkdown());
        Assert.EndsWith("\r\nChanged", document.ToMarkdown());
    }

    [Fact]
    public void ReadsBlockTagsWithoutChangingTheirRawFrontmatter()
    {
        const string markdown = "---\ntags:\n  - \"#ideas\"\n  - \"needs, review\"\n---\n\nBody";
        var document = MarkdownDocument.Parse(markdown);

        Assert.Equal(new[] { "ideas", "needs, review" }, document.Tags.ToArray());
        Assert.Equal(markdown, document.ToMarkdown());
    }
}
