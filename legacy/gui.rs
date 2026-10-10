use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui;

use chorus::router::Router;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Chorus",
        options,
        Box::new(|_cc| Ok(Box::new(ChorusApp::new()))),
    )
}

struct ChorusApp {
    source_device: cpal::Device,
    source_name: String,
    source_config: cpal::StreamConfig,
    source_volume: f32,
    source_muted: bool,

    sink_names: Vec<String>,
    sink_volumes: Vec<f32>,
    sink_muted: Vec<bool>,
    devices: Vec<DeviceRow>,

    router: Option<Router>,
}

struct DeviceRow {
    device: cpal::Device,
    name: String,
    selected: bool,
    volume: f32,
}

impl ChorusApp {
    fn new() -> Self {
        let (source_device, source_config, source_name, devices) = Self::enumerate();
        
        ChorusApp {
            source_device,
            source_config,
            source_name,
            source_volume: volumecontrol::AudioDevice::from_default()
                .and_then(|d| d.get_vol())
                .map(|v| v as f32 / 100.0)
                .unwrap_or(1.0),
            source_muted: false,

            sink_names: Vec::new(),
            sink_volumes: Vec::new(),
            sink_muted: Vec::new(),
            
            devices,

            router: None,
        }
    }

    fn enumerate() -> (cpal::Device, cpal::StreamConfig, String, Vec<DeviceRow>) {
        let host = cpal::default_host();
        let source_device = host
            .default_output_device()
            .expect("Error reading default output device");
        let source_config = source_device
            .default_output_config()
            .expect("Error reading source config")
            .config();
        let source_name = source_device.to_string();
        let devices = host
            .output_devices()
            .expect("Error enumerating output devices")
            .filter(|d| d.to_string() != source_name)
            .map(|d| DeviceRow {
                device: d.clone(),
                name: d.to_string(),
                selected: false,
                volume: 1.0,
            })
            .collect();
        (source_device, source_config, source_name, devices)
    }

    fn refresh(&mut self) {
        let (source_device, source_config, source_name, devices) = Self::enumerate();
        self.source_device = source_device;
        self.source_config = source_config;
        self.source_name = source_name;
        self.devices = devices;
        self.source_volume = volumecontrol::AudioDevice::from_default()
            .and_then(|d| d.get_vol())
            .map(|v| v as f32 / 100.0)
            .unwrap_or(1.0);
    }
}

impl eframe::App for ChorusApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.router.is_none() {
            // --------- Setup screen ---------
            ui.heading("Chorus");
            ui.separator();
            ui.separator();

            ui.label(format!("Source: {}", self.source_name));
            ui.add(egui::Slider::new(&mut self.source_volume, 0.0..=1.0));
            ui.separator();

            ui.label("Mirror to:");
            for row in &mut self.devices {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut row.selected, &row.name);
                    ui.add(egui::Slider::new(&mut row.volume, 0.0..=1.0));
                });
            }
            ui.separator();

            if ui.button("Start").clicked() {
                let sink_devices: Vec<(cpal::Device, f32)> = self
                    .devices
                    .iter()
                    .filter(|r| r.selected)
                    .map(|r| (r.device.clone(), r.volume))
                    .collect();

                // size of ring buffer to hold exactly 100ms of buffer data from the source device
                let capacity = (self.source_config.sample_rate as usize)
                    * (self.source_config.channels as usize)
                    / 10; // ~100ms os buffer, at any sample rate

                match Router::new(&self.source_device, &self.source_config, sink_devices, capacity) {
                    Ok(r) => {
                        self.sink_names = r.sinks().iter().map(|s| s.device_name.clone()).collect();
                        self.sink_volumes = r.sinks().iter().map(|s| s.get_volume()).collect();
                        self.sink_muted = vec![false; self.sink_names.len()];
                        let _ = r.set_source_volume(self.source_volume);
                        self.router = Some(r);
                    }
                    Err(e) => eprintln!("Error building router: {e}"),
                }
            }
        } else {
            // --------- Running screen ---------
            ui.heading("Chorus  - running");
            ui.separator();
            ui.separator();

            // Source (OS) Volume
            ui.horizontal(|ui| {
                ui.label(format!("Source: {}", self.source_name));

                let src = ui.add(egui::Slider::new(&mut self.source_volume, 0.0..=1.0));
                if src.changed() {
                    if let Some(router) = &self.router {
                        if let Err(e) = router.set_source_volume(self.source_volume) {
                            eprintln!("Source volume error: {e}");
                        }
                    }
                }
                
                let label = if self.source_muted { "Undeafen" } else { "Deafen" };
                if ui.button(label).clicked() {
                    self.source_muted = !self.source_muted;
                    if let Some(router) = &self.router {
                        if let Err(e) = router.set_source_mute(self.source_muted) {
                            eprintln!("Source mute error: {e}");
                        }
                    }
                }
            });
            ui.separator();

            // sinks (software gain)
            ui.label("Mirroring to: ");
            for i in 0..self.sink_names.len() {
                ui.horizontal(|ui| {
                    ui.label(&self.sink_names[i]);

                    let resp = ui.add(egui::Slider::new(&mut self.sink_volumes[i], 0.0..=1.0));
                    let label = if self.sink_muted[i] { "Undeafen" } else { "Deafen" };
                    let mute_clicked = ui.button(label).clicked();

                    if mute_clicked {
                        self.sink_muted[i] = !self.sink_muted[i];
                    }
                    if resp.changed() || mute_clicked {
                        if let Some(router) = &self.router {
                            let effective_vol = if self.sink_muted[i] { 0.0 } else { self.sink_volumes[i] };
                            router.set_volume(i, effective_vol);
                        }
                    }
                });
            }
            ui.separator();

            if ui.button("Stop").clicked() {
                self.router = None;
                self.refresh();
            }
        }
    }
}
