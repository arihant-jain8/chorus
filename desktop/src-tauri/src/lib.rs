use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn set_volume(index: usize, volume: f32, state: tauri::State<'_, Mutex<Sender<Command>>>) {
    let _ = state.lock().unwrap().send(Command::SetVolume { index, volume });
}

enum Command {
    SetVolume { index: usize, volume: f32 },
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = channel::<Command>();

    std::thread::spawn(move || {
        for cmd in rx {
            match cmd {
                Command::SetVolume { index, volume } => println!("{} {}", index, volume),
            }
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(tx))
        .invoke_handler(tauri::generate_handler![greet, set_volume])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
