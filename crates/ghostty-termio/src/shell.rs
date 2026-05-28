//! Shell command string helpers (`src/os/shell.zig`).

/// Builds a space-separated shell command string (Zig `ShellCommandBuilder`).
#[derive(Debug, Default)]
pub struct ShellCommandBuilder {
    parts: Vec<String>,
}

impl ShellCommandBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an argument, inserting a space before non-first segments.
    pub fn append_arg(&mut self, arg: &str) {
        if arg.is_empty() {
            return;
        }
        self.parts.push(arg.to_owned());
    }

    pub fn as_str(&self) -> String {
        self.parts.join(" ")
    }

    /// Owned null-terminated command (caller may use with C APIs).
    pub fn into_null_terminated(self) -> Vec<u8> {
        let mut buf = self.as_str().into_bytes();
        buf.push(0);
        buf
    }
}

/// Escape bytes that shells treat specially (`ShellEscapeWriter` subset).
pub fn shell_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        let escaped = match byte {
            b'\\' | b'"' | b'\'' | b'$' | b'`' | b'*' | b'?' | b' ' | b'|' | b'(' | b')' => {
                Some((b'\\', byte))
            }
            _ => None,
        };
        if let Some((slash, ch)) = escaped {
            out.push(slash as char);
            out.push(ch as char);
        } else {
            out.push(byte as char);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_command() {
        let cmd = ShellCommandBuilder::new();
        assert_eq!(cmd.as_str(), "");
    }

    #[test]
    fn single_and_multiple_args() {
        let mut cmd = ShellCommandBuilder::new();
        cmd.append_arg("bash");
        assert_eq!(cmd.as_str(), "bash");

        cmd.append_arg("--posix");
        cmd.append_arg("-l");
        assert_eq!(cmd.as_str(), "bash --posix -l");
    }

    #[test]
    fn skips_empty_arg() {
        let mut cmd = ShellCommandBuilder::new();
        cmd.append_arg("bash");
        cmd.append_arg("");
        assert_eq!(cmd.as_str(), "bash");
    }

    #[test]
    fn null_terminated() {
        let mut cmd = ShellCommandBuilder::new();
        cmd.append_arg("bash");
        cmd.append_arg("--posix");
        let buf = cmd.into_null_terminated();
        assert_eq!(&buf[..buf.len() - 1], b"bash --posix");
        assert_eq!(buf.last(), Some(&0));
    }

    #[test]
    fn shell_escape_samples() {
        assert_eq!(shell_escape("abc"), "abc");
        assert_eq!(shell_escape("a c"), "a\\ c");
        assert_eq!(shell_escape("a?c"), "a\\?c");
        assert_eq!(shell_escape("a\\c"), "a\\\\c");
        assert_eq!(shell_escape("a|c"), "a\\|c");
        assert_eq!(shell_escape("a\"c"), "a\\\"c");
        assert_eq!(shell_escape("a(1)"), "a\\(1\\)");
    }
}
