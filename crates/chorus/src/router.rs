use cpal::{BufferSize, Device, StreamConfig};
use volumecontrol::AudioDevice;

use crate::output::OutputSink;
use crate::source::CaptureSource;

pub struct Router {
    _source: CaptureSource,
    sinks: Vec<OutputSink>,
}

impl Router {
    pub fn new(
        source_device: &Device,
        source_config: &StreamConfig,
        sink_specs: Vec<(cpal::Device, f32)>,
        capacity: usize,
    ) -> Result<Router, cpal::Error> {
        let mut sinks = Vec::new();
        let mut producers = Vec::new();

        for (sink_device, vol) in sink_specs {
            let output_config = cpal::StreamConfig {
                channels: source_config.channels,
                sample_rate: source_config.sample_rate,
                buffer_size: BufferSize::Default,
            };

            match OutputSink::new(&sink_device, &output_config, capacity, vol) {
                Ok((sink, prod)) => {
                    println!("Streaming on: {}", &sink_device);
                    sinks.push(sink);
                    producers.push(prod);
                }
                Err(e) => println!("Skipping {}: {}", &sink_device, e),
            }
        }

        let _source = CaptureSource::new(source_device, source_config, producers)?;

        Ok(Router { _source, sinks })
    }

    pub fn set_sink_volume(&self, index: usize, v: f32) {
        if index < self.sinks.len() {
            self.sinks[index].set_volume(v);
        }
    }

    pub fn sink_counts(&self) -> usize {
        self.sinks.len()
    }

    pub fn sinks(&self) -> &[OutputSink] {
        &self.sinks
    }

    pub fn set_source_volume(&self, v: f32) -> Result<(), volumecontrol::AudioError> {
        let vol = (v.clamp(0.0, 1.0) * 100.0).round() as u8;
        AudioDevice::from_default()?.set_vol(vol)?;
        Ok(())
    }

    pub fn set_source_mute(&self, muted: bool) -> Result<(), volumecontrol::AudioError> {
        volumecontrol::AudioDevice::from_default()?.set_mute(muted)
    }

    pub fn set_sink_mute(&self, index: usize, muted: bool) {
        if index < self.sinks.len() {
            self.sinks[index].set_mute(muted);
        }
    }
}
