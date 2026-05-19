use std::{env, fs, path::Path};

fn main() {
    watch_frontend_dist();
    tauri_build::build()
}

fn watch_frontend_dist() {
    let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") else {
        return;
    };
    let dist = Path::new(&manifest_dir).join("..").join("dist");
    println!("cargo:rerun-if-changed={}", dist.display());
    watch_files(&dist);
}

fn watch_files(path: &Path) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            watch_files(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
