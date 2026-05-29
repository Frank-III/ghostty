//! Config file line parsing (`src/cli/args.zig` `LineIterator`).

use crate::error::SourceLocation;

const WHITESPACE: &[char] = &[' ', '\t', '\r'];

/// One logical config entry as `key` and optional `value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLine {
    pub key: String,
    pub value: Option<String>,
    pub line: usize,
}

/// Iterate non-comment `key = value` lines from a config file body.
pub struct LineIter<'a> {
    content: &'a str,
    line: usize,
    rest: &'a str,
}

impl<'a> LineIter<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            line: 0,
            rest: content,
        }
    }

    pub fn source_path(&self) -> &'a str {
        self.content
    }
}

impl<'a> Iterator for LineIter<'a> {
    type Item = ConfigLine;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.rest.is_empty() {
                return None;
            }
            let (raw, remainder) = match self.rest.find('\n') {
                Some(idx) => {
                    let (line, rest) = self.rest.split_at(idx);
                    (line, &rest[1..])
                }
                None => {
                    let line = self.rest;
                    self.rest = "";
                    (line, "")
                }
            };
            self.rest = remainder;
            self.line += 1;

            let entry = raw.trim_matches(WHITESPACE);
            if entry.is_empty() || entry.starts_with('#') {
                continue;
            }

            let (key, value) = if let Some(idx) = entry.find('=') {
                let key = entry[..idx].trim_matches(WHITESPACE);
                let mut value = entry[idx + 1..].trim_matches(WHITESPACE);
                if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
                    value = &value[1..value.len() - 1];
                }
                (
                    key.to_string(),
                    if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    },
                )
            } else {
                (entry.to_string(), None)
            };

            return Some(ConfigLine {
                key,
                value,
                line: self.line,
            });
        }
    }
}

impl ConfigLine {
    pub fn location(&self, path: &str) -> SourceLocation {
        SourceLocation {
            path: path.to_string(),
            line: self.line,
        }
    }
}

/// Strip a leading UTF-8 BOM if present.
pub fn strip_utf8_bom(content: &str) -> &str {
    content.strip_prefix('\u{feff}').unwrap_or(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(s: &str) -> Vec<(String, Option<String>)> {
        LineIter::new(s).map(|l| (l.key, l.value)).collect()
    }

    #[test]
    fn basic_lines() {
        let lines = collect("A\nB=42\nC\n\n# comment\nD\n  E\nF=  \"value \"\n");
        assert_eq!(
            lines,
            vec![
                ("A".into(), None),
                ("B".into(), Some("42".into())),
                ("C".into(), None),
                ("D".into(), None),
                ("E".into(), None),
                ("F".into(), Some("value ".into())),
            ]
        );
    }

    #[test]
    fn spaces_around_equals() {
        let lines = collect("A = B\n");
        assert_eq!(lines, vec![("A".into(), Some("B".into()))]);
    }

    #[test]
    fn bom_stripped() {
        assert_eq!(strip_utf8_bom("\u{feff}key = 1"), "key = 1");
    }
}
