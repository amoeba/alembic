use winresource::WindowsResource;

fn main() {
    let mut res = WindowsResource::new();
    let icon_path = "assets/alembic.ico";

    res.set_icon(icon_path);
    res.compile().expect("Failed to build.");
}
