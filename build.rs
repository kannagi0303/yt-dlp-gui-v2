#[cfg(target_os = "windows")]
fn main() {
    let mut resource = winres::WindowsResource::new();
    resource.set_icon("assets/logo.ico");
    resource
        .compile()
        .expect("failed to embed Windows application icon");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
