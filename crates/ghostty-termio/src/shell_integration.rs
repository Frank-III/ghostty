//! Automatic shell integration env injection (`src/termio/shell_integration.zig` subset).

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::Path;

use ghostty_config::ShellIntegration;

use crate::command::CommandSpec;

/// Shell kinds supported for automatic integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegratedShell {
    Bash,
    Zsh,
    Fish,
}

/// Context for applying shell integration to a spawn spec.
#[derive(Debug, Clone, Copy)]
pub struct ShellIntegrationContext<'a> {
    pub mode: ShellIntegration,
    pub resources_dir: Option<&'a Path>,
}

impl<'a> ShellIntegrationContext<'a> {
    pub fn new(mode: ShellIntegration, resources_dir: Option<&'a Path>) -> Self {
        Self {
            mode,
            resources_dir,
        }
    }
}

/// Detect shell from argv0 / program path (mirrors Zig `detectShell` on the executable name).
pub fn detect_shell(path: &OsStr, args: &[OsString]) -> Option<IntegratedShell> {
    let arg0 = args.first().map(OsStr::new).unwrap_or(path);
    let exe = Path::new(arg0)
        .file_name()
        .or_else(|| Path::new(path).file_name())?
        .to_string_lossy();
    let exe = exe.as_ref();

    if exe == "bash" {
        #[cfg(target_os = "macos")]
        if arg0 == OsStr::new("/bin/bash") {
            return None;
        }
        return Some(IntegratedShell::Bash);
    }
    if exe == "zsh" {
        return Some(IntegratedShell::Zsh);
    }
    if exe == "fish" {
        return Some(IntegratedShell::Fish);
    }
    None
}

fn forced_shell(mode: ShellIntegration) -> Option<IntegratedShell> {
    match mode {
        ShellIntegration::None | ShellIntegration::Detect => None,
        ShellIntegration::Bash => Some(IntegratedShell::Bash),
        ShellIntegration::Zsh => Some(IntegratedShell::Zsh),
        ShellIntegration::Fish => Some(IntegratedShell::Fish),
    }
}

fn resolve_shell(
    mode: ShellIntegration,
    path: &OsStr,
    args: &[OsString],
) -> Option<IntegratedShell> {
    if mode == ShellIntegration::None {
        return None;
    }
    if let Some(shell) = forced_shell(mode) {
        return Some(shell);
    }
    detect_shell(path, args)
}

fn env_map(spec: &mut CommandSpec) -> &mut BTreeMap<OsString, OsString> {
    spec.env.get_or_insert_with(BTreeMap::new)
}

fn setup_xdg_data_dirs(env: &mut BTreeMap<OsString, OsString>, resources_dir: &Path) -> bool {
    let integ_path = resources_dir.join("shell-integration");
    if !integ_path.is_dir() {
        return false;
    }
    env.insert(
        OsString::from("GHOSTTY_SHELL_INTEGRATION_XDG_DIR"),
        integ_path.as_os_str().to_os_string(),
    );
    let default = "/usr/local/share:/usr/share";
    let existing = env
        .get(OsStr::new("XDG_DATA_DIRS"))
        .and_then(|v| v.to_str())
        .unwrap_or(default);
    let merged = format!("{}:{}", integ_path.display(), existing);
    env.insert(OsString::from("XDG_DATA_DIRS"), OsString::from(merged));
    true
}

fn apply_zsh(env: &mut BTreeMap<OsString, OsString>, resources_dir: &Path) -> bool {
    let zdot = resources_dir.join("shell-integration/zsh");
    if !zdot.is_dir() {
        return false;
    }
    if let Some(old) = env.get(OsStr::new("ZDOTDIR")).cloned() {
        env.insert(OsString::from("GHOSTTY_ZSH_ZDOTDIR"), old);
    }
    env.insert(OsString::from("ZDOTDIR"), zdot.as_os_str().to_os_string());
    env.insert(
        OsString::from("GHOSTTY_SHELL_FEATURES"),
        OsString::from("cursor:blink,path,sudo,title"),
    );
    true
}

fn apply_bash(env: &mut BTreeMap<OsString, OsString>, resources_dir: &Path) -> bool {
    let script = resources_dir.join("shell-integration/bash/ghostty.bash");
    if !script.is_file() {
        env.insert(OsString::from("GHOSTTY_BASH_INJECT"), OsString::from("1"));
        return resources_dir.join("shell-integration").is_dir();
    }
    if let Some(old) = env.get(OsStr::new("ENV")).cloned() {
        env.insert(OsString::from("GHOSTTY_BASH_ENV"), old);
    }
    env.insert(OsString::from("ENV"), script.as_os_str().to_os_string());
    env.insert(OsString::from("GHOSTTY_BASH_INJECT"), OsString::from("1"));
    true
}

/// Apply shell integration env (and preserve command) to a built [`CommandSpec`].
pub fn apply_shell_integration(spec: &mut CommandSpec, ctx: &ShellIntegrationContext<'_>) {
    let Some(shell) = resolve_shell(ctx.mode, &spec.path, &spec.args) else {
        return;
    };
    let Some(resources_dir) = effective_resources(ctx) else {
        return;
    };

    let env = env_map(spec);
    match shell {
        IntegratedShell::Zsh => {
            let _ = apply_zsh(env, &resources_dir);
        }
        IntegratedShell::Bash => {
            let _ = apply_bash(env, &resources_dir);
        }
        IntegratedShell::Fish => {
            let _ = setup_xdg_data_dirs(env, &resources_dir);
        }
    }
}

fn effective_resources(ctx: &ShellIntegrationContext<'_>) -> Option<std::path::PathBuf> {
    ctx.resources_dir
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::var_os("GHOSTTY_RESOURCES_DIR").map(std::path::PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{OsStr, OsString};

    fn spec(path: &str, args: &[&str]) -> CommandSpec {
        CommandSpec {
            path: OsString::from(path),
            args: args.iter().map(|s| OsString::from(*s)).collect(),
            env: Some(BTreeMap::new()),
            cwd: None,
        }
    }

    #[test]
    fn detect_zsh_and_sh() {
        assert_eq!(
            detect_shell(OsStr::new("zsh"), &[]),
            Some(IntegratedShell::Zsh)
        );
        assert_eq!(
            detect_shell(OsStr::new("/bin/sh"), &[OsString::from("sh")]),
            None
        );
    }

    #[test]
    fn forced_mode_overrides_detect() {
        let mut s = spec("/bin/sh", &["sh".into()]);
        let tmp = std::env::temp_dir().join(format!("ghostty-integ-{}", std::process::id()));
        let _ = std::fs::create_dir_all(tmp.join("shell-integration/zsh"));
        apply_shell_integration(
            &mut s,
            &ShellIntegrationContext::new(ShellIntegration::Zsh, Some(&tmp)),
        );
        assert!(s.env.as_ref().unwrap().contains_key(OsStr::new("ZDOTDIR")));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn xdg_prepend_for_fish() {
        let mut s = spec("/usr/bin/fish", &["fish".into()]);
        let tmp = std::env::temp_dir().join(format!("ghostty-fish-{}", std::process::id()));
        std::fs::create_dir_all(tmp.join("shell-integration")).unwrap();
        apply_shell_integration(
            &mut s,
            &ShellIntegrationContext::new(ShellIntegration::Detect, Some(&tmp)),
        );
        let env = s.env.as_ref().unwrap();
        let xdg = env
            .get(OsStr::new("XDG_DATA_DIRS"))
            .unwrap()
            .to_str()
            .unwrap();
        assert!(xdg.starts_with(&format!("{}/shell-integration", tmp.display())));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn none_skips_env() {
        let mut s = spec("/bin/zsh", &["zsh".into()]);
        let tmp = std::env::temp_dir().join(format!("ghostty-none-{}", std::process::id()));
        std::fs::create_dir_all(tmp.join("shell-integration/zsh")).unwrap();
        apply_shell_integration(
            &mut s,
            &ShellIntegrationContext::new(ShellIntegration::None, Some(&tmp)),
        );
        assert!(s.env.as_ref().unwrap().get(OsStr::new("ZDOTDIR")).is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
