fn main() {
    // Embed the application icon in the Windows executable.
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/app/icon.ico")
            .compile()
            .expect("failed to embed Windows resources");
    }
}
