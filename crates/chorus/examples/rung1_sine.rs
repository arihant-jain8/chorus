// cargo run --bin rung1_sine
/// Program to output one tone into one of the audio output device
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();

    let default_device = host.default_output_device().unwrap();
    let config = default_device
        .default_output_config()
        .expect("Error reading default ouptut device config")
        .config();
    let sample_rate = config.sample_rate;
    let channels = config.channels;

    println!(
        "Playing 440 Hz on  {}: {}ch, {}Hz",
        &default_device.to_string(),
        channels,
        sample_rate
    );

    // --- persistent state owned by the callback ---
    let mut sample_index: f32 = 0.0; //running sample count; survives across callback via `move`
    let sample_rate_f = sample_rate as f32;
    let freq = 440.0_f32;
    let amplitude = 0.15_f32;

    // two-pi times freq, precomputed so the callback does less work
    let step = std::f32::consts::TAU * freq / sample_rate_f; // TAU is 2π

    let last_len = Arc::new(AtomicUsize::new(0));
    let last_len_cb = Arc::clone(&last_len);

    // step 3: build the stream
    let stream = default_device
        .build_output_stream(
            config,
            // cpal calls this repeatedly on the audio thread
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // record buffer length BEFORE the mutable loop borrows `data`
                let len = data.len();
                last_len_cb.store(len, Ordering::Relaxed);

                // step 4: fill the interleaved buffer, one frame at a time
                for frame in data.chunks_mut(channels as usize) {
                    // 1. Compute ONE sample value for this moment in time
                    let value = amplitude * (sample_index * step).sin(); /* sine of the current phase */

                    // 2. Write that same value to every channel in this frame
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }

                    // 3. advance time by one frame
                    // print!("{sample_index} "); should not do println in the real callbacks, it adds delay
                    sample_index += 1.0;
                }
            },
            // error callback: fires if stream faults
            move |err| {
                eprintln!("an error occured on the output audio: {}", err);
            },
            None // no timeout
        )
        .expect("failed to build output stream");

    stream.play().expect("failed to play the stream");

    // keep main (and thus `stream`) alive so we actually hear it
    // std::thread::sleep(std::time::Duration::from_secs(3));
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let len = last_len.load(Ordering::Relaxed);
        println!("Output buffer len = {}", len);
    }
}
