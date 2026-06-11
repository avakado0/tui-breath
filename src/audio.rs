use std::sync::{Arc, Mutex};

#[cfg(feature = "audio")]
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AudioMode {
    Off,
    Beep,
    #[cfg(feature = "audio")]
    Tone,
}

pub struct Beeper {
    enabled: Arc<Mutex<bool>>,
}

impl Beeper {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(Mutex::new(false)),
        }
    }

    #[allow(dead_code)]
    pub fn toggle(&self) {
        if let Ok(mut enabled) = self.enabled.lock() {
            *enabled = !*enabled;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.lock().map(|e| *e).unwrap_or(false)
    }

    pub fn beep(&self) {
        if !self.is_enabled() {
            return;
        }

        let enabled = Arc::clone(&self.enabled);
        std::thread::spawn(move || {
            if let Ok(true) = enabled.lock().map(|e| *e) {
                play_beep();
            }
        });
    }
}

fn play_beep() {
    use std::io::Write;
    print!("\x07");
    let _ = std::io::stdout().flush();
}

impl Clone for Beeper {
    fn clone(&self) -> Self {
        Self {
            enabled: Arc::clone(&self.enabled),
        }
    }
}

#[cfg(feature = "audio")]
struct OscillatorState {
    phase: f64,
    current_freq: f64,
    current_amp: f64,
}

#[cfg(feature = "audio")]
pub struct ToneEngine {
    freq_target: Arc<AtomicU64>,
    amplitude_target: Arc<AtomicU64>,
    state: Arc<Mutex<OscillatorState>>,
    _stream: cpal::Stream,
}

#[cfg(feature = "audio")]
impl ToneEngine {
    pub fn try_new() -> Option<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;

        let config = device.default_output_config().ok()?;
        let sample_rate = config.sample_rate().0;

        let freq_target = Arc::new(AtomicU64::new(110.0_f64.to_bits()));
        let amplitude_target = Arc::new(AtomicU64::new(0.0_f64.to_bits()));
        let state = Arc::new(Mutex::new(OscillatorState {
            phase: 0.0,
            current_freq: 110.0,
            current_amp: 0.0,
        }));

        let freq_clone = freq_target.clone();
        let amp_clone = amplitude_target.clone();
        let state_clone = state.clone();

        let callback = move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
            use std::f64::consts::PI;

            let alpha = 0.0005;

            if let Ok(mut s) = state_clone.try_lock() {
                for sample in output.iter_mut() {
                    let target_freq = f64::from_bits(freq_clone.load(Ordering::Relaxed));
                    let target_amp = f64::from_bits(amp_clone.load(Ordering::Relaxed));

                    s.current_freq += (target_freq - s.current_freq) * alpha;
                    s.current_amp += (target_amp - s.current_amp) * alpha;

                    s.phase += 2.0 * PI * s.current_freq / sample_rate as f64;
                    s.phase %= 2.0 * PI;

                    *sample = (s.phase.sin() * s.current_amp) as f32;
                }
            }
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_output_stream(&config.config(), callback, err_fn)
                .ok()?,
            cpal::SampleFormat::I16 => {
                let state_i16 = state.clone();
                let freq_i16 = freq_target.clone();
                let amp_i16 = amplitude_target.clone();

                let callback_i16 =
                    move |output: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        use std::f64::consts::PI;

                        let alpha = 0.0005;

                        if let Ok(mut s) = state_i16.try_lock() {
                            for sample in output.iter_mut() {
                                let target_freq =
                                    f64::from_bits(freq_i16.load(Ordering::Relaxed));
                                let target_amp =
                                    f64::from_bits(amp_i16.load(Ordering::Relaxed));

                                s.current_freq +=
                                    (target_freq - s.current_freq) * alpha;
                                s.current_amp += (target_amp - s.current_amp) * alpha;

                                s.phase += 2.0 * PI * s.current_freq / sample_rate as f64;
                                s.phase %= 2.0 * PI;

                                *sample =
                                    (s.phase.sin() * s.current_amp * 32767.0) as i16;
                            }
                        }
                    };

                device
                    .build_output_stream(&config.config(), callback_i16, err_fn)
                    .ok()?
            }
            cpal::SampleFormat::U16 => {
                let state_u16 = state.clone();
                let freq_u16 = freq_target.clone();
                let amp_u16 = amplitude_target.clone();

                let callback_u16 =
                    move |output: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        use std::f64::consts::PI;

                        let alpha = 0.0005;

                        if let Ok(mut s) = state_u16.try_lock() {
                            for sample in output.iter_mut() {
                                let target_freq =
                                    f64::from_bits(freq_u16.load(Ordering::Relaxed));
                                let target_amp =
                                    f64::from_bits(amp_u16.load(Ordering::Relaxed));

                                s.current_freq +=
                                    (target_freq - s.current_freq) * alpha;
                                s.current_amp += (target_amp - s.current_amp) * alpha;

                                s.phase += 2.0 * PI * s.current_freq / sample_rate as f64;
                                s.phase %= 2.0 * PI;

                                *sample = ((s.phase.sin() * s.current_amp * 0.5 + 0.5)
                                    * 65535.0) as u16;
                            }
                        }
                    };

                device
                    .build_output_stream(&config.config(), callback_u16, err_fn)
                    .ok()?
            }
            _ => return None,
        };

        stream.play().ok()?;

        Some(Self {
            freq_target,
            amplitude_target,
            state,
            _stream: stream,
        })
    }

    pub fn set_frequency(&self, hz: f64) {
        self.freq_target.store(hz.to_bits(), Ordering::Relaxed);
    }

    pub fn set_amplitude(&self, amplitude: f64) {
        self.amplitude_target.store(amplitude.to_bits(), Ordering::Relaxed);
    }
}

#[cfg(feature = "audio")]
fn err_fn(err: cpal::StreamError) {
    eprintln!("Stream error: {}", err);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_mode_off_and_beep() {
        assert_eq!(AudioMode::Off, AudioMode::Off);
        assert_ne!(AudioMode::Off, AudioMode::Beep);
    }
}
