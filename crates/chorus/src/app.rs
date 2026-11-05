use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::router::Router;

use cpal::Device;
use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;
use volumecontrol::AudioDevice;

#[derive(Clone, Debug, Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SinkState {
    pub name: String,
    pub volume: f32,
    pub muted: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct State {
    pub running: bool,

    pub source_name: String,
    pub source_volume: f32,
    pub source_muted: bool,

    pub devices: Vec<DeviceInfo>,
    pub sinks: Vec<SinkState>,
}

// Instructions from audience to DJ
pub enum Command {
    SetSinkVolume { index: usize, volume: f32 },
    SetSourceVolume(f32),
    SetSinkMute { index: usize, muted: bool },
    SetSourceMute(bool),
}

// Announcements from DJ
pub enum Event {
    DevicesChanged,
}

#[derive(Clone)]
pub struct Chorus {
    tx: Sender<Command>,
    state: Arc<Mutex<State>>,
}

fn build(state: &Arc<Mutex<State>>) -> (Router, String) {
    let host = cpal::default_host();

    let source_device = host
        .default_output_device()
        .expect("No default output device");
    let source_config = source_device
        .default_output_config()
        .expect("No default output config")
        .config();
    let source_name = source_device.to_string();

    let all_outputs: Vec<Device> = host
        .output_devices()
        .expect("Could not enumerate output devices")
        .collect();
    let device_list: Vec<DeviceInfo> = all_outputs
        .iter()
        .map(|d| DeviceInfo {
            name: d.to_string(),
            is_default: d.to_string() == source_name,
        })
        .collect();
    let sink_specs: Vec<(Device, f32)> = all_outputs
        .into_iter()
        .filter(|d| d.to_string() != source_name && !d.to_string().contains("Hands-Free"))
        .map(|d| (d, 0.5))
        .collect();

    let capacity = (source_config.sample_rate as usize) * (source_config.channels as usize) / 10;

    let router = Router::new(&source_device, &source_config, sink_specs, capacity)
        .expect("Failed to build router");

    println!("\nRouter Started\nDefault(source) : {}", source_name);
    // Setting all the sink device's volume to 100 default
    for (i, sink) in router.sinks().iter().enumerate() {
        match AudioDevice::from_name(&sink.device_name) {
            Ok(dev) => match dev.set_vol(100) {
                Ok(()) => println!("{}: {}", i, sink.device_name),
                Err(e) => eprintln!("couldn't set volume for {}: {}", sink.device_name, e),
            },
            Err(e) => eprintln!("no endpoint match for {}: {}", sink.device_name, e),
        }
    }

    // (3) seed the whiteboard
    {
        let mut s = state.lock().unwrap();
        s.running = true;

        s.source_name = source_name.clone();
        s.source_volume = AudioDevice::from_default()
            .and_then(|d| d.get_vol())
            .map(|v| v as f32 / 100.0)
            .unwrap_or(1.0);
        s.source_muted = false;

        s.devices = device_list;
        s.sinks = router
            .sinks()
            .iter()
            .map(|sink| SinkState {
                name: sink.device_name.clone(),
                volume: sink.get_volume(),
                muted: false,
            })
            .collect();
    }

    (router, source_name)
}

impl Chorus {
    pub fn launch() -> (Self, Receiver<Event>) {
        // (1) OUTSIDE the thread - make the slot + whiteboard
        let (tx, rx) = channel::<Command>();
        let (event_tx, event_rx) = channel::<Event>();
        let state = Arc::new(Mutex::new(State::default()));
        let state_thread = Arc::clone(&state);

        std::thread::spawn(move || {
            // (2) build the booth: the thread owns ALL the cpal stuff
            let (mut router, mut source_name) = build(&state_thread);

            loop {
                // (4) DJ loop - handle cards forever
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(Command::SetSinkVolume { index, volume }) => {
                        router.set_sink_volume(index, volume);
                        let mut s = state_thread.lock().unwrap();
                        if let Some(sink) = s.sinks.get_mut(index) {
                            sink.volume = volume;
                        }
                    }

                    Ok(Command::SetSourceVolume(volume)) => {
                        match router.set_source_volume(volume) {
                            Ok(()) => {
                                state_thread.lock().unwrap().source_volume = volume;
                            }
                            Err(e) => eprintln!("source volume error: {}", e),
                        }
                    }

                    Ok(Command::SetSourceMute(muted)) => match router.set_source_mute(muted) {
                        Ok(()) => state_thread.lock().unwrap().source_muted = muted,
                        Err(e) => eprintln!("source mute error: {}", e),
                    },

                    Ok(Command::SetSinkMute { index, muted }) => {
                        router.set_sink_mute(index, muted);
                        let mut s = state_thread.lock().unwrap();
                        if let Some(sink) = s.sinks.get_mut(index) {
                            sink.muted = muted;
                        }
                    }

                    // 1s passed with no card - did the default change?
                    Err(RecvTimeoutError::Timeout) => {
                        if let Some(dev) = cpal::default_host().default_output_device() {
                            let new_default = dev.to_string();

                            if new_default != source_name {
                                println!("\n\ndefault changed -> re-tapping: {}", new_default);
                                let (new_router, new_source_name) = build(&state_thread);
                                router = new_router;
                                source_name = new_source_name;

                                // sending DevicesChanged signal to the UI
                                let _ = event_tx.send(Event::DevicesChanged);
                            }
                        }
                    }

                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        (Chorus { tx, state }, event_rx)
    }

    pub fn state(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    // handler functions (to send the command to router object)
    pub fn set_sink_volume(&self, index: usize, volume: f32) {
        let _ = self.tx.send(Command::SetSinkVolume { index, volume });
    }

    pub fn set_source_volume(&self, volume: f32) {
        let _ = self.tx.send(Command::SetSourceVolume(volume));
    }

    pub fn set_source_mute(&self, muted: bool) {
        let _ = self.tx.send(Command::SetSourceMute(muted));
    }

    pub fn set_sink_mute(&self, index: usize, muted: bool) {
        let _ = self.tx.send(Command::SetSinkMute { index, muted });
    }
}
