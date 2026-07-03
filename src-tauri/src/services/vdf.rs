// Minimal recursive VDF (Valve KeyValues) parser — mirrors VdfParser.cs.
// Only handles the subset needed to read localconfig.vdf: nested `"key" { ... }`
// sections and `"key" "value"` leaves, with `//` line comments.

use std::collections::HashMap;

pub struct VdfNode {
    pub children: HashMap<String, VdfNode>,
}

impl VdfNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&VdfNode> {
        self.children
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
    }
}

pub fn parse(content: &str) -> VdfNode {
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0usize;
    skip_whitespace(&chars, &mut pos);
    read_string(&chars, &mut pos); // root key
    skip_whitespace(&chars, &mut pos);
    parse_section(&chars, &mut pos)
}

fn parse_section(chars: &[char], pos: &mut usize) -> VdfNode {
    let mut node = VdfNode::new();
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
            let _value = read_string(chars, pos); // leaf value not needed by callers
            node.children.insert(key, VdfNode::new());
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
