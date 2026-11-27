//! Audio capture worker for BassVisualizer using cpal and rustfft.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Default)]
pub struct AudioMetrics {
    pub bass: f32,
    pub flux: f32,
}

fn build_mono_stream<T, F>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    channels: usize,
    max_buf: usize,
    pcm_buffer: Arc<Mutex<Vec<f32>>>,
    to_f32: F,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::SizedSample,
    F: Fn(T) -> f32 + Send + 'static,
{
    device.build_input_stream(
        &config,
        move |data: &[T], _| {
            let mut buf = pcm_buffer.lock().unwrap_or_else(|e| e.into_inner());
            for chunk in data.chunks_exact(channels) {
                let mono = chunk.iter().map(|&s| to_f32(s)).sum::<f32>() / channels as f32;
                buf.push(mono);
            }
            let excess = buf.len().saturating_sub(max_buf);
            if excess > 0 {
                buf.drain(..excess);
            }
        },
        |err| eprintln!("[Visualizer Audio Stream Error]: {}", err),
        None,
    )
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
                eprintln!(
                    "[Visualizer Audio Warning] Device '{}' not found, falling back to default.",
                    target
                );
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
                eprintln!(
                    "[Visualizer Audio Error]: Failed to get default output config: {}",
                    e
                );
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        let sample_rate = config.sample_rate().0 as f32;
        let channels = (config.channels() as usize).max(1);

        let block_size = 8192;
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(block_size);

        let hanning: Vec<f32> = (0..block_size)
            .map(|i| {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / block_size as f32).cos())
            })
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
            let center_bin = (((bass_min + bass_max) * 0.5) / freq_step)
                .round()
                .clamp(1.0, (block_size / 2 - 1) as f32) as usize;
            bass_bins.push(center_bin);
        }
        println!(
            "[Visualizer Audio] Sub-bass window {:.1} Hz - {:.1} Hz mapped to {} FFT bins (bin step: {:.2} Hz)",
            bass_min,
            bass_max,
            bass_bins.len(),
            freq_step
        );

        let pcm_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(block_size)));
        let max_buf = block_size * 4;
        let sample_format = config.sample_format();
        let stream_cfg: cpal::StreamConfig = config.into();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => build_mono_stream::<f32, _>(
                &device,
                stream_cfg,
                channels,
                max_buf,
                Arc::clone(&pcm_buffer),
                |s| s,
            ),
            cpal::SampleFormat::I16 => build_mono_stream::<i16, _>(
                &device,
                stream_cfg,
                channels,
                max_buf,
                Arc::clone(&pcm_buffer),
                |s| s as f32 / 32768.0,
            ),
            cpal::SampleFormat::I32 => build_mono_stream::<i32, _>(
                &device,
                stream_cfg,
                channels,
                max_buf,
                Arc::clone(&pcm_buffer),
                |s| s as f32 / 2_147_483_648.0,
            ),
            cpal::SampleFormat::U16 => build_mono_stream::<u16, _>(
                &device,
                stream_cfg,
                channels,
                max_buf,
                Arc::clone(&pcm_buffer),
                |s| (s as f32 - 32768.0) / 32768.0,
            ),
            cpal::SampleFormat::F64 => build_mono_stream::<f64, _>(
                &device,
                stream_cfg,
                channels,
                max_buf,
                Arc::clone(&pcm_buffer),
                |s| s as f32,
            ),
            _ => {
                eprintln!(
                    "[Visualizer Audio Error]: Unsupported sample format: {:?}",
                    sample_format
                );
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
        let mut new_samples = Vec::<f32>::with_capacity(block_size);
        let mut fft_input = vec![Complex::new(0.0f32, 0.0f32); block_size];
        let mut prev_bass = 0.0f32;
        let mut idle_ticks = 0u32;

        while running.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(8));

            new_samples.clear();
            {
                let mut buf = pcm_buffer.lock().unwrap_or_else(|e| e.into_inner());
                if !buf.is_empty() {
                    std::mem::swap(&mut new_samples, &mut *buf);
                }
            }

            if new_samples.is_empty() {
                idle_ticks = idle_ticks.saturating_add(1);
                // WASAPI loopback stops delivering packets on pause; zero metrics after ~48ms idle.
                if idle_ticks == 6 {
                    sliding_window.fill(0.0);
                    prev_bass = 0.0;
                    let mut m = metrics.lock().unwrap_or_else(|e| e.into_inner());
                    m.bass = 0.0;
                    m.flux = 0.0;
                }
                continue;
            }
            idle_ticks = 0;

            let n = new_samples.len();
            if n >= block_size {
                sliding_window.copy_from_slice(&new_samples[n - block_size..]);
            } else {
                sliding_window.rotate_left(n);
                let start_idx = block_size - n;
                sliding_window[start_idx..].copy_from_slice(&new_samples);
            }

            for ((slot, &s), &w) in fft_input
                .iter_mut()
                .zip(sliding_window.iter())
                .zip(hanning.iter())
            {
                *slot = Complex::new(s * w, 0.0);
            }

            fft.process(&mut fft_input);

            let sum: f32 = bass_bins.iter().map(|&idx| fft_input[idx].norm()).sum();
            let bass = sum / bass_bins.len() as f32;

            let flux = (bass - prev_bass).max(0.0);
            prev_bass = bass;

            let mut m = metrics.lock().unwrap_or_else(|e| e.into_inner());
            m.bass = bass;
            m.flux = flux;
        }
    });
}
