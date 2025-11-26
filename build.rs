fn main() {
    println!("Compiling Slint...");
    slint_build::compile_with_config(
        "./ui/app-window.slint",
        slint_build::CompilerConfiguration::new().with_bundled_translations("./ui/locale"),
    ).expect("Slint build failed");
    println!("Slint compiled successfully");
}
