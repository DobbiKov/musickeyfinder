//! Harmonic transition analyzer module
//!

use crate::types::{key_to_camelot, Key};
const MAJOR_PROFILE: [f64; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];
const MINOR_PROFILE: [f64; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];
const KEY_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

fn get_best_frame_key(chroma: &[f64]) -> usize {
    let mut max_corr = -1.0;
    let mut best_key_idx = 0;
    let profiles = [MAJOR_PROFILE.as_slice(), MINOR_PROFILE.as_slice()].concat();

    for i in 0..24 {
        let profile_root = i % 12;
        let is_major = i < 12;
        let profile = if is_major {
            &MAJOR_PROFILE
        } else {
            &MINOR_PROFILE
        };
        let mut rotated_chroma = chroma.to_vec();
        rotated_chroma.rotate_left(profile_root);
        let corr: f64 = rotated_chroma
            .iter()
            .zip(profile.iter())
            .map(|(c, p)| c * p)
            .sum();
        if corr > max_corr {
            max_corr = corr;
            best_key_idx = i;
        }
    }
    best_key_idx
}

/// Returns the best key AND the confidence score for that key.
pub fn analyze_track(chroma_sequence: &[Vec<f64>]) -> (Option<Key>, f64) {
    if chroma_sequence.is_empty() {
        return (None, 0.0);
    }

    let frame_keys: Vec<usize> = chroma_sequence
        .iter()
        .map(|frame| get_best_frame_key(frame))
        .collect();
    let mut transition_scores = vec![vec![0.0; 24]; 24];
    for i in 0..24 {
        transition_scores[i][i] = 1.0;
        let root = i % 12;
        let is_major = i < 12;
        let tonic_maj = root;
        let tonic_min = root + 12;
        let dominant_maj = (root + 7) % 12;
        let dominant_min = (root + 7) % 12 + 12;
        let subdominant_maj = (root + 5) % 12;
        let subdominant_min = (root + 5) % 12 + 12;
        if is_major {
            transition_scores[dominant_maj][tonic_maj] = 5.0;
            transition_scores[dominant_min][tonic_maj] = 4.0;
            transition_scores[subdominant_maj][tonic_maj] = 2.0;
        } else {
            transition_scores[dominant_min][tonic_min] = 5.0;
            transition_scores[dominant_maj][tonic_min] = 5.0;
            transition_scores[subdominant_min][tonic_min] = 2.0;
        }
    }

    let mut global_key_scores = vec![0.0; 24];
    for global_key_candidate in 0..24 {
        let mut current_score = 0.0;
        if frame_keys[0] == global_key_candidate {
            current_score += 1.0;
        }
        for i in 1..frame_keys.len() {
            let from_key = frame_keys[i - 1];
            let to_key = frame_keys[i];
            if to_key == global_key_candidate {
                current_score += transition_scores[from_key][to_key];
            }
        }
        global_key_scores[global_key_candidate] = current_score;
    }

    let (best_idx, best_score) = global_key_scores
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap_or((0, &0.0));
    let key_root = KEY_NAMES[best_idx % 12];
    let key_mode = if best_idx < 12 { "Major" } else { "Minor" };
    let key = format!("{} {}", key_root, key_mode);
    let cam_key: Key = key_to_camelot(key.as_str()).into();
    (Some(cam_key), *best_score)
}
