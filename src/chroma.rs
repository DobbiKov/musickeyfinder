//! Chroma featrue extraction module

use rustfft::{num_complex::Complex, FftPlanner};

const FFT_SIZE: usize = 8192;
const HOP_SIZE: usize = 2048;

/// Computes a sequence of chromagrams using a specific tuning reference.
pub fn compute_chromagram_sequence(audio: &[f32], sample_rate: u32, a4_freq: f32) -> Vec<Vec<f64>> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let window: Vec<f32> = (0..FFT_SIZE)
        .map(|i| {
            0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos())
        })
        .collect();
    let mut chroma_sequence = Vec::new();

    for frame_start in (0..audio.len()).step_by(HOP_SIZE) {
        let frame_end = frame_start + FFT_SIZE;
        if frame_end > audio.len() {
            break;
        }

        let mut buffer: Vec<Complex<f32>> = audio[frame_start..frame_end]
            .iter()
            .zip(window.iter())
            .map(|(sample, w)| Complex::new(sample * w, 0.0))
            .collect();
        fft.process(&mut buffer);

        let magnitudes: Vec<f32> = buffer
            .iter()
            .take(FFT_SIZE / 2 + 1)
            .map(|c| c.norm())
            .collect();

        let mut frame_chroma = vec![0.0f64; 12];
        for k in 1..magnitudes.len() {
            let freq = k as f32 * sample_rate as f32 / FFT_SIZE as f32;
            if freq > 0.0 && freq < 5000.0 {
                // Use the provided a4_freq for pitch calculation
                let midi_pitch = 12.0 * (freq / a4_freq).log2() + 69.0;
                if midi_pitch > 0.0 {
                    let chroma_bin = (midi_pitch.round() as i32 % 12) as usize;
                    frame_chroma[chroma_bin] += magnitudes[k] as f64;
                }
            }
        }

        let sum: f64 = frame_chroma.iter().sum();
        if sum > 0.0 {
            chroma_sequence.push(frame_chroma.iter().map(|v| v / sum).collect());
        }
    }
    chroma_sequence
}
