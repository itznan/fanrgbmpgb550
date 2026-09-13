//! Audio capture worker for BassVisualizer using cpal and rustfft.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

#[derive(Debug, Clone, Default)]
pub struct AudioMetrics {
    pub bass: f32,
    pub flux: f32,
}

pub fn start_audio_capture(
    running: Arc<AtomicBool>,
    metrics: Arc<Mutex<AudioMetrics>>,
    bass_min: f32,
    bass_max: f32,
    device_name: Option<String>,
) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let device = if let Some(ref target) = device_name {
            let target_lower = target.to_lowercase();
            let mut found = None;
            if let Ok(devices) = host.output_devices() {
                for dev in devices {
                    if let Ok(name) = dev.name() {
                        if name.to_lowercase().contains(&target_lower) {
                            println!("[Visualizer Audio] Using specified audio device: {}", name);
                            found = Some(dev);
                            break;
                        }
                    }
                }
            }
            if found.is_none() {
                eprintln!("[Visualizer Audio Warning] Device '{}' not found, falling back to default.", target);
            }
            found.or_else(|| host.default_output_device())
        } else {
            host.default_output_device()
        };

        let device = match device {
            Some(dev) => dev,
            None => {
                eprintln!("[Visualizer Audio Error]: Default output device not found.");
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        if let Ok(name) = device.name() {
            println!("[Visualizer Audio] Capturing from: {}", name);
        }

        let config = match device.default_output_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("[Visualizer Audio Error]: Failed to get default output config: {}", e);
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;

        let block_size = 8192;
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(block_size);

        let hanning: Vec<f32> = (0..block_size)
            .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / block_size as f32).cos()))
            .collect();

        // Calculate frequency bin indices for bass_min..bass_max
        let freq_step = sample_rate / block_size as f32;
        let mut bass_bins: Vec<usize> = (1..block_size / 2)
            .filter(|&i| {
                let freq = i as f32 * freq_step;
                freq >= bass_min && freq <= bass_max
            })
            .collect();
        if bass_bins.is_empty() {
            bass_bins.push(1);
        }
        println!(
            "[Visualizer Audio] Sub-bass window {:.1} Hz - {:.1} Hz mapped to {} FFT bins (bin step: {:.2} Hz)",
            bass_min,
            bass_max,
            bass_bins.len(),
            freq_step
        );

        let pcm_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let pcm_buf_clone = Arc::clone(&pcm_buffer);

        let err_fn = |err| eprintln!("[Visualizer Audio Stream Error]: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let mut buf = pcm_buf_clone.lock().unwrap();
                    for chunk in data.chunks_exact(channels) {
                        let mono = chunk.iter().sum::<f32>() / channels as f32;
                        buf.push(mono);
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => {
                let pcm_buf_i16 = Arc::clone(&pcm_buffer);
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _| {
                        let mut buf = pcm_buf_i16.lock().unwrap();
                        for chunk in data.chunks_exact(channels) {
                            let mono = chunk.iter().map(|&s| s as f32 / 32768.0).sum::<f32>() / channels as f32;
                            buf.push(mono);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => {
                eprintln!("[Visualizer Audio Error]: Unsupported sample format.");
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Visualizer Audio Error]: Build input stream failed: {}", e);
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("[Visualizer Audio Error]: Stream play failed: {}", e);
            running.store(false, Ordering::SeqCst);
            return;
        }

        let mut sliding_window = vec![0.0f32; block_size];
        let mut prev_bass = 0.0f32;

        while running.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(8));

            let new_samples: Vec<f32> = {
                let mut buf = pcm_buffer.lock().unwrap();
                if buf.is_empty() {
                    continue;
                }
                buf.drain(..).collect()
            };

            let n = new_samples.len();
            if n >= block_size {
                sliding_window.copy_from_slice(&new_samples[n - block_size..]);
            } else {
                sliding_window.rotate_left(n);
                let start_idx = block_size - n;
                sliding_window[start_idx..].copy_from_slice(&new_samples);
            }

            let mut fft_input: Vec<Complex<f32>> = sliding_window
                .iter()
                .zip(hanning.iter())
                .map(|(&s, &w)| Complex::new(s * w, 0.0))
                .collect();

            fft.process(&mut fft_input);

            let mut bass = 0.0f32;
            if !bass_bins.is_empty() {
                let sum: f32 = bass_bins
                    .iter()
                    .map(|&idx| fft_input[idx].norm())
                    .sum();
                bass = sum / bass_bins.len() as f32;
            }

            let flux = (bass - prev_bass).max(0.0);
            prev_bass = bass;

            if let Ok(mut m) = metrics.lock() {
                m.bass = bass;
                m.flux = flux;
            }
        }
    });
}
