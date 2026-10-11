use std::io::{self, Write};

use chorus::router::Router;
use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();

    let input_device = host
        .default_output_device()
        .expect("Error reading default output device");

    let input_config = input_device
        .default_output_config()
        .expect(&format!(
            "Error reading input config for {}",
            input_device.to_string()
        ))
        .config();

    let sink_devices: Vec<(cpal::Device, f32)> = host
        .output_devices()
        .expect("Error enumerating output devices")
        .into_iter()
        .filter(|d| d.to_string() != input_device.to_string())
        .map(|d| (d, 1.0))
        .collect();

    // size of ring buffer to hold exactly 100ms of buffer data from the source device
    let capacity = (input_config.sample_rate as usize) * (input_config.channels as usize) / 10;

    let router = Router::new(&input_device, &input_config, sink_devices, capacity)
        .expect("Error building router");

    println!("{} Active Outputs - ", router.sink_counts());
    for (i, sink) in router.sinks().iter().enumerate() {
        println!("  {i}: {}", sink.device_name);
    }

    println!(
        "\n Type `<index> <volume>` (eg. `0 0.5`) or `s <volume>` for the main device or `q` to quit."
    );
    loop {
        print!("cmd> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        if input == "q" {
            break;
        }

        let tokens: Vec<&str> = input.split_whitespace().collect();
        if tokens.len() != 2 {
            println!("bad input - expected: <index> <volume>");
            continue;
        }

        if tokens[0] == "s" {
            match tokens[1].parse::<f32>() {
                Ok(vol) => match router.set_source_volume(vol) {
                    Ok(()) => println!("setting source -> {vol}"),
                    Err(e) => println!("Source volume error: {}", e),
                },
                Err(_) => println!("Bad input - couldnt parse volume"),
            }
            continue;
        }

        match (tokens[0].parse::<usize>(), tokens[1].parse::<f32>()) {
            (Ok(ind), Ok(vol)) => {
                if ind < router.sink_counts() {
                    router.set_volume(ind, vol);
                    println!("setting {ind} -> {vol}");
                } else {
                    println!(
                        "index out of bounds. expected range 0 - {}",
                        router.sink_counts()
                    )
                }
            }
            _ => println!("bad input - couldn't parse numbers"),
        }
    }
}
