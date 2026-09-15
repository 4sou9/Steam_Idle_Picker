// Minimal recursive VDF (Valve KeyValues) parser — mirrors VdfParser.cs.
// Handles the subset used by Steam's text files (localconfig.vdf, libraryfolders.vdf,
// appmanifest_*.acf): nested `"key" { ... }` sections and `"key" "value"` leaves, with
// `//` line comments and backslash escapes.

use std::collections::HashMap;

#[derive(Default)]
pub struct VdfNode {
    pub children: HashMap<String, VdfNode>,
    /// Set for `"key" "value"` leaves, `None` for sections.
    pub value: Option<String>,
}

impl VdfNode {
    /// Child lookup; VDF keys are case-insensitive.
    pub fn get(&self, key: &str) -> Option<&VdfNode> {
        self.children
            .get(key)
            .or_else(|| self.children.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v))
    }

    /// Value of the leaf child `key`.
    pub fn value_of(&self, key: &str) -> Option<&str> {
        self.get(key)?.value.as_deref()
    }
}

/// Parses a document and returns the root key's section (the root key itself, e.g.
/// `"AppState"`, is skipped).
pub fn parse(content: &str) -> VdfNode {
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0usize;
    skip_whitespace(&chars, &mut pos);
    read_string(&chars, &mut pos); // root key
    skip_whitespace(&chars, &mut pos);
    parse_section(&chars, &mut pos)
}

fn parse_section(chars: &[char], pos: &mut usize) -> VdfNode {
    let mut node = VdfNode::default();
    if *pos < chars.len() && chars[*pos] == '{' {
        *pos += 1;
    }

    loop {
        skip_whitespace(chars, pos);
        if *pos >= chars.len() || chars[*pos] == '}' {
            if *pos < chars.len() {
                *pos += 1;
            }
            break;
        }
        if chars[*pos] != '"' {
            break;
        }

        let key = read_string(chars, pos);
        skip_whitespace(chars, pos);

        if *pos < chars.len() && chars[*pos] == '{' {
            node.children.insert(key, parse_section(chars, pos));
        } else if *pos < chars.len() && chars[*pos] == '"' {
            let value = read_string(chars, pos);
            node.children.insert(
                key,
                VdfNode {
                    value: Some(value),
                    ..VdfNode::default()
                },
            );
        } else {
            break;
        }
    }

    node
}

fn read_string(chars: &[char], pos: &mut usize) -> String {
    if *pos >= chars.len() || chars[*pos] != '"' {
        return String::new();
    }
    *pos += 1;
    let mut s = String::new();
    while *pos < chars.len() && chars[*pos] != '"' {
        if chars[*pos] == '\\' && *pos + 1 < chars.len() {
            *pos += 1;
            s.push(match chars[*pos] {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                c => c,
            });
        } else {
            s.push(chars[*pos]);
        }
        *pos += 1;
    }
    if *pos < chars.len() {
        *pos += 1;
    }
    s
}

fn skip_whitespace(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() {
        if chars[*pos].is_whitespace() {
            *pos += 1;
            continue;
        }
        if *pos + 1 < chars.len() && chars[*pos] == '/' && chars[*pos + 1] == '/' {
            while *pos < chars.len() && chars[*pos] != '\n' {
                *pos += 1;
            }
            continue;
        }
        break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_sections_and_values() {
        let root = parse(
            r#"
            // comment
            "Root"
            {
                "Section"
                {
                    "Key"    "Value"
                }
                "Leaf"   "x"
            }
            "#,
        );
        assert_eq!(root.get("section").and_then(|s| s.value_of("key")), Some("Value"));
        assert_eq!(root.value_of("Leaf"), Some("x"));
        assert!(root.get("Section").unwrap().value.is_none());
    }

    #[test]
    fn unescapes_strings() {
        let root = parse(r#""Root" { "name" "Say \"hi\"" "path" "C:\\Games\\Steam" }"#);
        assert_eq!(root.value_of("name"), Some(r#"Say "hi""#));
        assert_eq!(root.value_of("path"), Some(r"C:\Games\Steam"));
    }

    #[test]
    fn tolerates_truncated_input() {
        let root = parse(r#""Root" { "a" { "b" "1""#);
        assert_eq!(root.get("a").and_then(|a| a.value_of("b")), Some("1"));
    }
}
