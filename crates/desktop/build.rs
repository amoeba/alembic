use std::{env, fs, path::Path};
use winresource::WindowsResource;

fn main() {
    let mut res = WindowsResource::new();
    let icon_path = "assets/alembic.ico";
    res.set_icon(icon_path);
    res.compile().expect("Failed to build.");

    // Copy loog.png
    let out_dir = env::var("OUT_DIR").unwrap();

    // OUT_DIR is the build output directory for the desktop crate which is
    // inside the target directory. So we need to traverse up a few directories
    // to find the right place to copy to
    let final_out_dir = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let final_copy_target = final_out_dir.join("logo.png");

    // Paths are reference from the desktop crate root, not workspace
    fs::copy("assets\\logo.png", final_copy_target)
        .expect("Failed to copy logo.png to build directory");
}
