use std::env;
use std::error::Error;

use types::Key;

mod audio;
mod chroma;
mod errors;
mod harmonic_analyzer;
mod types;

pub fn analyze_key(file_path: &str) -> Option<Key> {
    println!("Analyzing file: {}", file_path);

    let (audio_buffer, sample_rate) = match audio::load_audio_file(file_path) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error loading audio file: {}", e);
            std::process::exit(1);
        }
    };
    println!(
        "-> Audio loaded successfully ({} samples at {} Hz)",
        audio_buffer.len(),
        sample_rate
    );

    let mut best_key: Option<Key> = None;
    let mut best_score = -1.0;
    let mut best_tuning_hz = 440.0;

    println!("-> Starting tuning analysis...");
    // Test tunings from A4=427Hz (-50 cents) to A4=453Hz (+50 cents)
    for i in -5..=5 {
        let cents_offset = i as f32 * 10.0;
        let a4_freq = 440.0 * 2.0_f32.powf(cents_offset / 1200.0);

        let chroma_sequence =
            chroma::compute_chromagram_sequence(&audio_buffer, sample_rate, a4_freq);
        if chroma_sequence.is_empty() {
            continue;
        }

        let (key, score) = harmonic_analyzer::analyze_track(&chroma_sequence);
        let key_str = match key {
            None => "unknown".to_string(),
            Some(k) => String::from(k),
        };

        print!(
            "    - Testing A4={:.1} Hz... found {} (score: {:.2})\r",
            a4_freq, key_str, score
        );
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        if score > best_score {
            best_score = score;
            best_key = key;
            best_tuning_hz = a4_freq;
        }
    }

    if best_key.is_none() {
        println!("\n\n===================================");
        println!("Analysis Complete!");
        println!("The key is unknown");
        println!("===================================");
        return None;
    }

    println!("\n\n===================================");
    println!("Analysis Complete!");
    println!("Detected Tuning: A4 = {:.1} Hz", best_tuning_hz);
    println!("Estimated Key: {}", best_key.unwrap());
    println!("===================================");
    best_key
}
