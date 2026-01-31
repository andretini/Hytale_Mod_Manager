fn main() {
    #[cfg(windows)]
    {
        use winresource::WindowsResource;

        let mut res = WindowsResource::new();
        // This path must be relative to Cargo.toml
        res.set_icon("icon.ico");

        if let Err(e) = res.compile() {
            eprintln!("Error compiling icon: {}", e);
            std::process::exit(1);
        }
    }
}