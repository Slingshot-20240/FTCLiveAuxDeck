#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Manager, State};

struct AudioRequest {
    path: PathBuf,
    volume: f32,
}

struct AudioState {
    tx: Mutex<mpsc::Sender<AudioRequest>>,
}

#[tauri::command]
fn play_sound(app: AppHandle, state: State<'_, AudioState>, sound_name: String, volume: f32) {
    let file_path = match sound_name.as_str() {
        "3-2-1" => "audio/3-2-1.wav",
        "abort" => "audio/abort.wav",
        "auto_end" => "audio/auto_end.wav",
        "endgame" => "audio/endgame.wav",
        "match_end" => "audio/match_end.wav",
        "match_start" => "audio/match_start.wav",
        "pick_up_controllers" => "audio/pick_up_controllers.wav",
        "results" => "audio/results.wav",
        "reveal" => "audio/reveal.wav",
        "teleop_start" => "audio/teleop_start.wav",
        "unmute" => "audio/unmute.wav",
        _ => {
            eprintln!("Unknown sound name received: {}", sound_name);
            return;
        }
    };

    let resource_path = app
        .path()
        .resolve(file_path, tauri::path::BaseDirectory::Resource)
        .expect("failed to resolve resource");

    let tx = state.tx.lock().unwrap();
    let _ = tx.send(AudioRequest {
        path: resource_path,
        volume,
    });
}

fn main() {
    let (tx, rx) = mpsc::channel::<AudioRequest>();

    thread::spawn(move || {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

        while let Ok(request) = rx.recv() {
            if let Ok(data) = fs::read(request.path) {
                let cursor = Cursor::new(data);

                if let Ok(sink) = rodio::Sink::try_new(&stream_handle) {
                    match rodio::Decoder::new(cursor) {
                        Ok(source) => {
                            sink.set_volume(request.volume);
                            sink.append(source);
                            sink.detach();
                        }
                        Err(e) => eprintln!("Error decoding audio file: {}", e),
                    }
                }
            } else {
                eprintln!("Failed to read file into memory.");
            }
        }
    });

    tauri::Builder::default()
        .manage(AudioState { tx: Mutex::new(tx) })
        .invoke_handler(tauri::generate_handler![play_sound])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
