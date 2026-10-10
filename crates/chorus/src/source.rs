use cpal::traits::{DeviceTrait, StreamTrait};
use ringbuf::{HeapProd, traits::Producer};

pub struct CaptureSource {
    _stream: cpal::Stream,
}

impl CaptureSource {
    pub fn new(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mut producers: Vec<HeapProd<f32>>,
    ) -> Result<Self, cpal::Error> {
        let device_name = device.to_string();

        let in_stream = device
            .build_input_stream(
                config.clone(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    for prod in producers.iter_mut() {
                        prod.push_slice(data);
                    }
                },
                move |err| {
                    if !matches!(err.kind(), cpal::ErrorKind::Xrun) {
                        eprintln!("Input error for device: {}", device_name);
                    }
                },
                None,
            )
            .expect(&format!(
                "Error while building input stream for device: {}",
                device.to_string()
            ));

        in_stream.play()?;
        // .expect(&format!("Error while playing input stream for device: {}", device.to_string()));

        Ok(CaptureSource { _stream: in_stream })
    }
}
