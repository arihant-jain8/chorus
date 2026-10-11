use chorus::app::Chorus;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn set_volume(index: usize, volume: f32, state: tauri::State<'_, Chorus>) {
    state.set_sink_volume(index, volume);
}

#[tauri::command]
fn set_source_volume(volume: f32, state: tauri::State<'_, Chorus>) {
    state.set_source_volume(volume);
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let chorus = Chorus::launch(); // 1) state the engine, get handle

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(chorus) // hand the handle to Tauri
        .invoke_handler(tauri::generate_handler![greet, set_volume, set_source_volume])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
