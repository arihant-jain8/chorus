/// Program to output one microphone input to all the connected audio output devices
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};

fn main() {
    let host = cpal::default_host();

    let input_device = host
        .input_devices()
        .expect("Could not enumerate input devices.")
        .find(|d: &cpal::Device| {
            let name = d.to_string();
            name.contains("realme") && name.contains("Hands-Free")
        })
        .expect("Realme Buds not found.");

    let input_config = input_device
        .default_input_config()
        .expect("Error reading input config")
        .config();

    let output_devices = host
        .output_devices()
        .expect("Error reading output decvices");

    let mut producers = Vec::new();
    let mut output_streams = Vec::new();

    for out_device in output_devices {
        let out_config = cpal::StreamConfig {
            channels: input_config.channels,
            sample_rate: input_config.sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let rb = HeapRb::<f32>::new(3200);
        let (prod, mut cons) = rb.split();
        producers.push(prod);

        let stream = match out_device.build_output_stream(
            out_config,
            move |data: &mut [f32], _| {
                let n = cons.pop_slice(data);
                for s in &mut data[n..] {
                    *s = 0.0;
                }
            },
            move |err| {
                if !matches!(err.kind(), cpal::ErrorKind::Xrun) {
                    eprintln!("Out err: {err}");
                }
            },
            None,
        ) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping {}: {e}", out_device.to_string());
                continue;
            }
        };

        output_streams.push(stream);
    }

    let input_stream = input_device
        .build_input_stream(
            input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for prod in producers.iter_mut() {
                    prod.push_slice(data);
                }
            },
            move |err| {
                if !matches!(err.kind(), cpal::ErrorKind::Xrun) {
                    eprintln!("Input error: {err}");
                }
            },
            None,
        )
        .expect("Error building input stream");

    input_stream.play().expect("Error playing input stream");
    for out_stream in &output_streams {
        out_stream.play().expect("Erro playing output stream");
    }

    println!(
        "Fan-out on {} outputs. Ctrl + C to stop.",
        output_streams.len()
    );
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
