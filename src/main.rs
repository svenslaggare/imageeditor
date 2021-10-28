mod program;
mod helpers;
mod command_buffer;
mod rendering;
mod ui;
mod editor;
mod glfw_app;
mod gtk_app;

fn main() {
    // glfw_app::main();
    gtk_app::main();
}