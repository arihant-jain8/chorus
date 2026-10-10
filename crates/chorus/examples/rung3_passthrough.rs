use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};

fn main() {
    let host = cpal::default_host();

    // select the mic - default
    // let input_device = host
    //     .default_input_device()
    //     .expect("No input device available");

    // select realme mic
    let input_device = host
        .input_devices()
        .expect("could not enumerate input devices")
        .find(|d: &cpal::Device| {
            let name = d.to_string(); // in cpal 0.18, Display gives the device name
            name.contains("realme") && name.contains("Hands-Free")
        })
        .expect("realme Buds Hands-Free mic not found");

    let input_config = input_device
        .default_input_config()
        .expect("Error reading input device default config")
        .config();

    println!(
        "Capturing from - {}: {}ch {}Hz",
        input_device, input_config.channels, input_config.sample_rate,
    );

    // select default output device
    let output_device = host.default_output_device().expect("No output devices.");

    // Same channels + sample_rate as the input -> the ring buffer's samples map 1:1
    // WASAPI shared mode converts this to DAC's native format for us (no resampling in our code)
    let output_config = cpal::StreamConfig {
        channels: input_config.channels,
        sample_rate: input_config.sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    println!(
        "Playback on - {}: {}ch {}Hz",
        output_device, output_config.channels, output_config.sample_rate,
    );

    let rb = HeapRb::<f32>::new(3200); // ~200ms at 16kHz mono; tune later
    let (mut prod, mut cons) = rb.split();

    // build input stream
    let input_stream = input_device
        .build_input_stream(
            input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                prod.push_slice(data);
            },
            move |err| {
                if !matches!(err.kind(), cpal::ErrorKind::Xrun) {
                    eprintln!("Input error: {err}");
                }
            },
            None,
        )
        .expect("Error while building input stream");

    // build output stream
    let output_stream = output_device
        .build_output_stream(
            output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let n = cons.pop_slice(data); // reads the data from the rb into data
                for s in &mut data[n..] {
                    *s = 0.0;
                } // fills the empty space in the data by 0s
            },
            move |err| eprintln!("Output error: {err}"),
            None,
        )
        .expect("failed to build output stream");

    input_stream
        .play()
        .expect("Error while playing input stream");
    output_stream
        .play()
        .expect("Error while playing output stream");

    println!("Passthrough running — talk into the mic. Press Ctrl+C to stop.");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
