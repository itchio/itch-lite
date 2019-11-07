use std::path::PathBuf;
use std::process::Command;
use std::{env, io};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    let rust_path = env::current_dir()?;
    let native_path = rust_path.join("native");

    env::set_current_dir(&native_path)?;

    // Make sure the platform is supported.

    if !cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos",
    )) {
        panic!("unsupported platform");
    }

    // Link any platform-specific dependencies.

    if cfg!(target_os = "linux") {
        pkg_config::Config::new()
            .atleast_version("3.14")
            .probe("gtk+-3.0")?;
        pkg_config::Config::new()
            .atleast_version("2.22")
            .probe("webkit2gtk-4.0")?;
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=dylib=ole32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=windowsapp");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    }

    // Build and link to the library.

    if cfg!(target_os = "linux") {
        let mut cc = cc::Build::new();

        for flag in sh("pkg-config --cflags gtk+-3.0 webkit2gtk-4.0")?.split_whitespace() {
            cc.flag(flag);
        }

        cc.file("gtk.c").compile("tether");
    } else if cfg!(target_os = "windows") {
        cc::Build::new()
            .flag("/EHsc")
            .flag("/std:c++17")
            .file("winapi.cpp")
            .compile("tether");
    } else if cfg!(target_os = "macos") {
        cc::Build::new()
            .flag("-ObjC")
            .flag("-fobjc-arc")
            .file("cocoa.m")
            .compile("tether");
    }

    // N.B: bindings used to be generated with bindgen,
    // now they're kept in sync manually, see `src/raw.rs`

    Ok(())
}

fn sh(script: &str) -> io::Result<String> {
    Command::new("sh")
        .args(&["-c", script])
        .output()
        .and_then(|o| {
            if o.status.success() {
                Ok(String::from_utf8_lossy(&o.stdout).into())
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "script failed"))
            }
        })
}
