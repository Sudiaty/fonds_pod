fn main() {
    // Set default APP_VERSION if not provided
    if std::env::var("APP_VERSION").is_err() {
        println!("cargo:rustc-env=APP_VERSION=dev");
    }

    println!("Compiling Slint...");
    slint_build::compile_with_config(
        "./ui/app-window.slint",
        slint_build::CompilerConfiguration::new().with_bundled_translations("./ui/locale"),
    ).expect("Slint build failed");
    println!("Slint compiled successfully");

    // Embed Windows resources (icon for tray)
    #[cfg(windows)]
    {
        embed_resource::compile("resources.rc", embed_resource::NONE);
    }
}
