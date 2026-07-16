use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=.env");

    let env_path = Path::new(".env");
    if env_path.exists() {
        if let Ok(contents) = fs::read_to_string(env_path) {
            for line in contents.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    if !key.is_empty() && !key.starts_with('#') {
                        println!("cargo:rustc-env={}={}", key, value);
                    }
                }
            }
        }
    }
}
