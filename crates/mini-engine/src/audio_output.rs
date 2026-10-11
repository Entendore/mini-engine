//! Optional cpal output: pulls mixed samples from the shared core
//! [`Mixer`](mini_core::audio::Mixer). Keeping the returned
//! [`cpal::Stream`] alive keeps audio running — `run()` holds it for the
//! whole session.

use std::sync::{Arc, Mutex};

use mini_core::audio::Mixer;
use mini_core::error::{EngineError, Result};

/// Open the default OS output device against `mixer`.
pub fn try_start(mixer: Arc<Mutex<Mixer>>) -> Result<cpal::Stream> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| EngineError::AudioInit("no audio output device found".into()))?;
    let supported = device
        .default_output_config()
        .map_err(|e| EngineError::AudioInit(e.to_string()))?;
    if supported.sample_format() != cpal::SampleFormat::F32 {
        return Err(EngineError::AudioInit(format!(
            "unsupported sample format {:?}",
            supported.sample_format()
        )));
    }
    let config: cpal::StreamConfig = supported.config();
    let channels = supported.channels() as usize;
    if let Ok(mut m) = mixer.lock() {
        m.set_sample_rate(supported.sample_rate().0 as f32);
    }
    let cb_mixer = Arc::clone(&mixer);

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| match cb_mixer.lock() {
                Ok(mut mixer) => mixer.render(data, channels),
                Err(_) => data.fill(0.0),
            },
            |err| eprintln!("mini_engine audio: {err}"),
            None,
        )
        .map_err(|e| EngineError::AudioInit(e.to_string()))?;
    stream.play().map_err(|e| EngineError::AudioInit(e.to_string()))?;
    Ok(stream)
}
