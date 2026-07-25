#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    win_task_manager_lib::run()
}
