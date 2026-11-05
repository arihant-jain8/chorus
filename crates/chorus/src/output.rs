use cpal::traits::{DeviceTrait, StreamTrait};
use ringbuf::traits::{Consumer, Split};
use ringbuf::{HeapProd, HeapRb};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub struct OutputSink {
    pub device_name: String,
    _stream: cpal::Stream,
    volume: Arc<AtomicU32>,
    muted: Arc<AtomicBool>,
}

impl OutputSink {
    pub fn new(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        capacity: usize,
        initial_volume: f32,
    ) -> Result<(OutputSink, HeapProd<f32>), cpal::Error> {
        let device_name = device.to_string();
        let volume = Arc::new(AtomicU32::new(initial_volume.to_bits()));
        let volume_cb = Arc::clone(&volume);
        let muted = Arc::new(AtomicBool::new(false));
        let muted_cb = Arc::clone(&muted);

        let rb = HeapRb::<f32>::new(capacity);
        let (prod, mut cons) = rb.split();

        let out_stream = device
            .build_output_stream(
                config.clone(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let vol = if muted_cb.load(Ordering::Relaxed) {
                        0.0
                    } else {
                        f32::from_bits(volume_cb.load(Ordering::Relaxed))
                    };
                    // read the samples from cons
                    let n = cons.pop_slice(data);
                    // set the volume of those samples
                    for s in &mut data[..n] {
                        *s *= vol;
                    }
                    // set the rest of the samples 0
                    for s in &mut data[n..] {
                        *s = 0.0;
                    }
                },
                move |err| {
                    if !matches!(err.kind(), cpal::ErrorKind::Xrun) {
                        eprintln!("Output stream error: {err}");
                    }
                },
                None,
            )
            .expect(&format!(
                "Error building output stream for device: {device_name}"
            ));

        out_stream.play()?;
        // .expect(&format!("Error playing output stream for device: {}", device_name));

        Ok((
            OutputSink {
                device_name,
                _stream: out_stream,
                volume,
                muted,
            },
            prod,
        ))
    }

    pub fn set_volume(&self, v: f32) {
        self.volume.store(v.to_bits(), Ordering::Relaxed);
    }

    pub fn get_volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    pub fn set_mute(&self, m: bool) {
        self.muted.store(m, Ordering::Relaxed);
    }
}
