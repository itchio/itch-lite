use std::{ffi::OsStr, path::PathBuf, process::Command, time::Instant};

fn main() {
    let task = std::env::args().skip(1).next();
    match task.as_ref().map(String::as_str) {
        Some(task) => match task.as_ref() {
            "bind" => {
                bind();
            }
            _ => usage(),
        },
        _ => usage(),
    }
}

fn bind() {
    println!();

    let start = Instant::now();

    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("should be run from cargo")
        .into();
    let tether_dir = manifest_dir.parent().unwrap();

    let config_file = tether_dir.join("cbindgen.toml");
    let input_path = tether_dir.join("src").join("raw.rs");
    let output_path = tether_dir.join("native").join("tether.h");

    println!("{:>12} `{}`", "Generating", output_path.to_string_lossy());
    println!("{:>12} `{}`", "From", input_path.to_string_lossy());
    println!("{:>12} `{}`", "With config", config_file.to_string_lossy());

    let mut child = Command::new("cbindgen")
        .args(&[
            OsStr::new("--config"),
            config_file.as_os_str(),
            OsStr::new("--output"),
            output_path.as_os_str(),
            input_path.as_os_str(),
        ])
        .spawn()
        .expect("should be able to start cbindgen");
    child.wait().expect("cbindgen should complete successfully");

    {
        // "fix" cbindgen output to suit our needs
        let contents = std::fs::read_to_string(&output_path).unwrap();
        let contents = contents
            .replace("extern ", "")
            .replace(r#""C""#, r#"extern "C""#)
            .replace(" _tether;", " _tether_dummy;")
            .replace("typedef _tether", "typedef struct _tether");
        std::fs::write(&output_path, contents).unwrap();
    }

    println!();
    println!("Done generating in {:?}", start.elapsed());
}

fn usage() {
    println!("Usage: cargo xtask bind");
    std::process::exit(1);
}
