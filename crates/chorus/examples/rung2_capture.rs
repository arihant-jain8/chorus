/// Program to read input from microphone and displaying metric bars
// cargo run --bin rung2_capture
use std::{
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicU32, AtomicUsize, Ordering},
    },
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();

    // let devices = match host.input_devices() {
    //     Ok(d) => d,
    //     Err(e) => {
    //         eprintln!("Could not read input devices: {e}");
    //         return;
    //     }
    // };

    // for (i, device) in devices.enumerate() {
    //     println!("{}: {}", i + 1, device);
    // }

    let device = host
        .default_input_device()
        .expect("No input device available");
    // let device = host
    //     .input_devices()
    //     .expect("could not enumerate input devices")
    //     .find(|d: &cpal::Device| {
    //         let name = d.to_string(); // in cpal 0.18, Display gives the device name
    //         name.contains("realme") && name.contains("Hands-Free")
    //     })
    // .expect("realme Buds Hands-Free mic not found");
    let config = device
        .default_input_config()
        .expect("Error reading default input config")
        .config();

    println!(
        "Capturing from {}: {} ch, {} Hz",
        &device, config.channels, config.sample_rate
    );

    // --- shared telemetry: the callback writes here, main reads it ---
    // No AtomicF32 exists, so we store f32's raw bits in an Atomicu32
    let level = Arc::new(AtomicU32::new(0));
    let level_cb = Arc::clone(&level);

    // diagnostic: count how many times the capture callback actually fires
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_cb = Arc::clone(&calls);

    // experimental: printing size of data
    let last_len = Arc::new(AtomicUsize::new(0));
    let last_len_cb = Arc::clone(&last_len);

    // --- build INPUT stream ---
    let stream = device
        .build_input_stream(
            config,
            //note: &[f32] is READ-ONLY, cpal already filled it with mic samples
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // diagnostic: prove the callback is running
                calls_cb.fetch_add(1, Ordering::Relaxed);

                // compute RMS = sqrt(mean of squares) - perceived loudness
                let sum_sq: f32 = data.iter().map(|s| s * s).sum();
                let rms = (sum_sq / data.len() as f32).sqrt();

                // publish it: bounded math + one atomic store. Both real-time-safe
                level_cb.store(rms.to_bits(), Ordering::Relaxed);

                last_len_cb.store(data.len(), Ordering::Relaxed);
            },
            move |err| eprintln!("Input stream error: {err}"),
            None,
        )
        .expect("Failed to build input stream");

    stream.play().expect("Failed to play input stream");

    // --- control plane: main reads the atomic and prints. I/O lives here, never in the callback. ---
    for _ in 0..200 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        let rms = f32::from_bits(level.load(Ordering::Relaxed));

        // map the level to a bar. scale is arbitrary - tweak until it feels right.
        let bars = (rms * 200.0) as usize;
        let bar = "#".repeat(bars.min(50));
        let n = calls.load(Ordering::Relaxed);
        let buf_len = last_len.load(Ordering::Relaxed);
        print!(
            "\r[{:<50}] rms={:.4} calls={} buf_len={}",
            bar, rms, n, buf_len
        ); // \r returns to line start -> meter updates in place
        std::io::stdout().flush().unwrap();
    }
}
