fn main() {
    println!("Compiling Slint...");
    slint_build::compile("./ui/app-window.slint").expect("Slint build failed");
    println!("Slint compiled successfully");
}
