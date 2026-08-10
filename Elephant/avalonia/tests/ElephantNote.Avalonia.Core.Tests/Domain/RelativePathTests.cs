using System.Text.Json;
using ElephantNote.Avalonia.Domain;

namespace ElephantNote.Avalonia.Core.Tests.Domain;

public sealed class RelativePathTests
{
    [Theory]
    [InlineData("/vault/note.md")]
    [InlineData("\\vault\\note.md")]
    [InlineData("C:\\vault\\note.md")]
    [InlineData("Notes/../note.md")]
    [InlineData("../note.md")]
    public void RejectsAbsoluteAndParentPaths(string value) =>
        Assert.Throws<ArgumentException>(() => RelativePath.Parse(value));

    [Fact]
    public void NormalizesSeparatorsWithoutRewritingParentSegments()
    {
        var path = RelativePath.Parse("./Notes\\Ideas//Note.md");

        Assert.Equal("Notes/Ideas/Note.md", path.Value);
        Assert.Equal("Notes/Ideas", path.Parent.Value);
        Assert.Equal("Note.md", path.FileName);
    }

    [Fact]
    public void RejectsUnsafePathsWhenDeserializingSidebarJson()
    {
        Assert.Throws<JsonException>(() => JsonSerializer.Deserialize<SidebarEntry>("{\"path\":\"../outside.md\"}"));
    }
}
