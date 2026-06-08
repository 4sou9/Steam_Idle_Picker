using System.Collections.Generic;
using System.Text;

namespace SteamIdlePicker.Services;

internal class VdfNode
{
    public Dictionary<string, VdfNode> Children { get; } = new(System.StringComparer.OrdinalIgnoreCase);
    public string? Value { get; init; }
}

internal static class VdfParser
{
    public static VdfNode Parse(string content)
    {
        int pos = 0;
        SkipWhitespace(content, ref pos);
        ReadString(content, ref pos); // root key
        SkipWhitespace(content, ref pos);
        return ParseSection(content, ref pos);
    }

    private static VdfNode ParseSection(string content, ref int pos)
    {
        var node = new VdfNode();
        if (pos < content.Length && content[pos] == '{') pos++;

        while (pos < content.Length)
        {
            SkipWhitespace(content, ref pos);
            if (pos >= content.Length || content[pos] == '}') { if (pos < content.Length) pos++; break; }
            if (content[pos] != '"') break;

            var key = ReadString(content, ref pos);
            SkipWhitespace(content, ref pos);

            if (pos < content.Length && content[pos] == '{')
                node.Children[key] = ParseSection(content, ref pos);
            else if (pos < content.Length && content[pos] == '"')
                node.Children[key] = new VdfNode { Value = ReadString(content, ref pos) };
        }

        return node;
    }

    private static string ReadString(string content, ref int pos)
    {
        if (pos >= content.Length || content[pos] != '"') return "";
        pos++;
        var sb = new StringBuilder();
        while (pos < content.Length && content[pos] != '"')
        {
            if (content[pos] == '\\' && pos + 1 < content.Length)
            {
                pos++;
                sb.Append(content[pos] switch { 'n' => '\n', 't' => '\t', 'r' => '\r', _ => content[pos] });
            }
            else
            {
                sb.Append(content[pos]);
            }
            pos++;
        }
        if (pos < content.Length) pos++;
        return sb.ToString();
    }

    private static void SkipWhitespace(string content, ref int pos)
    {
        while (pos < content.Length)
        {
            if (char.IsWhiteSpace(content[pos])) { pos++; continue; }
            if (pos + 1 < content.Length && content[pos] == '/' && content[pos + 1] == '/')
            {
                while (pos < content.Length && content[pos] != '\n') pos++;
                continue;
            }
            break;
        }
    }
}
