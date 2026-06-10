#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod runner;
mod bounce;

fn main() {
    let effect = bounce::Bounce::new();
    runner::run_main(effect, "bounce");
}
