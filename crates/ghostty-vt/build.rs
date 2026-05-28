fn main() {
    println!("cargo::rustc-check-cfg=cfg(ghostty_vt_terminal_owned)");
    println!("cargo:rustc-env=GHOSTTY_VT_SIMD=1");
    println!("cargo:rustc-env=GHOSTTY_VT_KITTY_GRAPHICS=1");
    println!("cargo:rustc-env=GHOSTTY_VT_TMUX_CONTROL_MODE=1");
    println!("cargo:rustc-env=GHOSTTY_VT_OPTIMIZE=debug");
    println!("cargo:rustc-env=GHOSTTY_VT_VERSION_STRING=0.1.0-dev");
    println!("cargo:rustc-env=GHOSTTY_VT_VERSION_MAJOR=0");
    println!("cargo:rustc-env=GHOSTTY_VT_VERSION_MINOR=1");
    println!("cargo:rustc-env=GHOSTTY_VT_VERSION_PATCH=0");
    println!("cargo:rustc-env=GHOSTTY_VT_VERSION_PRE=");
    println!("cargo:rustc-env=GHOSTTY_VT_VERSION_BUILD=");
}
