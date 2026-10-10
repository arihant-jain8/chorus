// cargo run --bin rung0_devices
/// Program to list all kinds of audio devices
use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();

    let devices = match host.output_devices() {
        Ok(d) => d,
        Err(e) => {
            println!("Could not enumerarte output devices: {e}");
            return;
        }
    };

    for (i, device) in devices.enumerate() {
        println!("\n{}: {}", i + 1, &device);

        match device.default_output_config() {
            Ok(cfg) => println!(
                "   default: {}ch, {}Hz, {:?}",
                cfg.channels(),
                cfg.sample_rate(),
                cfg.sample_format()
            ),
            Err(e) => println!("    default: <unavailable: {e}>"),
        }

        // default_output_config() asks a different question: "what format is this device actually running at right now?"
        // So when your program asks a device "can you do 8000 Hz, 8-bit?", the question never reaches the hardware. Windows answers on its behalf: "sure — I'll convert it for you." And Windows will say yes to essentially anything, for every device
        // match device.supported_output_configs() {
        //     Ok(ranges) => {
        //         let ranges: Vec<_> = ranges.collect();
        //         // ranges.sort();
        //         println!("  {} supported range(s)", ranges.len());
        //         for r in ranges {
        //             println!("       {}ch, {}-{} Hz, {:?}",
        //                 r.channels(),
        //                 r.min_sample_rate(),
        //                 r.max_sample_rate(),
        //                 r.sample_format()
        //             );
        //         }
        //     }
        //     Err(e) => println!("    configs: <unavailable {e}>"),
        // }
    }
}
