use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();

    let device = host
        .input_devices()
        .expect("could not enumerate input devices")
        .find(|d: &cpal::Device| d.to_string().contains("Audiocular"))
        .expect("Audiocular D07 mic not found");

    println!("Device: {device}");

    let default_cfg = device
        .default_input_config()
        .expect("no default input config");
    println!(
        "default_input_config: {}ch, {}Hz, {:?}",
        default_cfg.channels(),
        default_cfg.sample_rate(),
        default_cfg.sample_format()
    );

    let config = default_cfg.config();
    println!(
        "\nOpening stream at {}ch {}Hz ...",
        config.channels, config.sample_rate
    );

    let calls = Arc::new(AtomicUsize::new(0));
    let calls_cb = Arc::clone(&calls);
    let errored = Arc::new(AtomicBool::new(false));
    let errored_cb = Arc::clone(&errored);
    let peak = Arc::new(AtomicU32::new(0));
    let peak_cb = Arc::clone(&peak);

    let stream = device
        .build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                calls_cb.fetch_add(1, Ordering::Relaxed);
                let local_peak = data.iter().fold(0.0f32, |m, s| m.max(s.abs()));
                let prev = f32::from_bits(peak_cb.load(Ordering::Relaxed));
                if local_peak > prev {
                    peak_cb.store(local_peak.to_bits(), Ordering::Relaxed);
                }
            },
            move |err| {
                errored_cb.store(true, Ordering::Relaxed);
                eprintln!("STREAM ERROR: {err}");
            },
            None,
        )
        .expect("failed to build input stream");

    stream.play().expect("failed to play");

    // give it 3 seconds — make noise into the mic during this window
    std::thread::sleep(std::time::Duration::from_secs(3));

    println!(
        "\nAfter 3s: callback fired {} times, errored = {}, peak amplitude = {:.6}",
        calls.load(Ordering::Relaxed),
        errored.load(Ordering::Relaxed),
        f32::from_bits(peak.load(Ordering::Relaxed))
    );
}
