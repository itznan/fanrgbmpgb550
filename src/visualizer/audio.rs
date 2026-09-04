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
) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(dev) => dev,
            None => {
                eprintln!("[Visualizer Audio Error]: Default output device not found.");
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

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

        let block_size = 1024;
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(block_size);

        let hanning: Vec<f32> = (0..block_size)
            .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / block_size as f32).cos()))
            .collect();

        // Calculate frequency bin indices for bass_min..bass_max
        let freq_step = sample_rate / block_size as f32;
        let bass_bins: Vec<usize> = (0..block_size / 2)
            .filter(|&i| {
                let freq = i as f32 * freq_step;
                freq >= bass_min && freq <= bass_max
            })
            .collect();

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

        let mut prev_bass = 0.0f32;

        while running.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(10));

            let samples = {
                let mut buf = pcm_buffer.lock().unwrap();
                if buf.len() >= block_size {
                    let chunk: Vec<f32> = buf.drain(..block_size).collect();
                    chunk
                } else {
                    continue;
                }
            };

            let mut fft_input: Vec<Complex<f32>> = samples
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
