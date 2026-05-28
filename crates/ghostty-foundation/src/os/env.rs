//! Environment variable list helpers ported from `src/os/env.zig`.

/// Path-list delimiter (`:` on Unix, `;` on Windows).
#[cfg(windows)]
const PATH_DELIMITER: char = ';';
#[cfg(not(windows))]
const PATH_DELIMITER: char = ':';

/// Append `value` to a path-list style variable (e.g. `PATH`).
///
/// If `current` is empty, returns `value` alone. Always allocates a new string.
pub fn append_env(current: &str, value: &str) -> String {
    if current.is_empty() {
        return value.to_owned();
    }
    append_env_always(current, value)
}

/// Append `value` even when `current` is empty (preserves an empty prefix for vars like `MANPATH`).
pub fn append_env_always(current: &str, value: &str) -> String {
    format!("{current}{PATH_DELIMITER}{value}")
}

/// Prepend `value` to a path-list style variable.
///
/// If `current` is empty, returns `value` alone. Always allocates a new string.
pub fn prepend_env(current: &str, value: &str) -> String {
    if current.is_empty() {
        return value.to_owned();
    }
    format!("{value}{PATH_DELIMITER}{current}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_env_empty() {
        assert_eq!(append_env("", "foo"), "foo");
    }

    #[test]
    fn append_env_existing() {
        let result = append_env("a:b", "foo");
        if cfg!(windows) {
            assert_eq!(result, "a:b;foo");
        } else {
            assert_eq!(result, "a:b:foo");
        }
    }

    #[test]
    fn prepend_env_empty() {
        assert_eq!(prepend_env("", "foo"), "foo");
    }

    #[test]
    fn prepend_env_existing() {
        let result = prepend_env("a:b", "foo");
        if cfg!(windows) {
            assert_eq!(result, "foo;a:b");
        } else {
            assert_eq!(result, "foo:a:b");
        }
    }
}
