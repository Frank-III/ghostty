//! Default word boundaries and whitespace characters used for selection logic.
//!
//! Port of `src/terminal/selection_codepoints.zig`.

/// Default boundary characters for word selection: ` \t'"│`|:;,()[]{}<>$`
pub const DEFAULT_WORD_BOUNDARIES: [char; 20] = [
    '\0',
    ' ',
    '\t',
    '\'',
    '"',
    '\u{2502}',
    '`',
    '|',
    ':',
    ';',
    ',',
    '(',
    ')',
    '[',
    ']',
    '{',
    '}',
    '<',
    '>',
    '$',
];

/// Default whitespace characters trimmed from line selections.
pub const DEFAULT_LINE_WHITESPACE: [char; 3] = ['\0', ' ', '\t'];
