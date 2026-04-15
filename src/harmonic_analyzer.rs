//! Harmonic transition analyzer module
//!

use std::convert::TryFrom;

use crate::types::{key_to_camelot, Key};
const MAJOR_PROFILE: [f64; 12] = [
    6.18576717376709,
    5.271093368530273,
    3.1486291885375977,
    -0.31995293498039246,
    -0.7044875621795654,
    -0.22531749308109283,
    2.5953574180603027,
    3.1163809299468994,
    3.9606964588165283,
    4.81988000869751,
    5.719341278076172,
    6.218751430511475,
];
const MINOR_PROFILE: [f64; 12] = [
    5.981163024902344,
    5.9503092765808105,
    5.130553245544434,
    2.968756675720215,
    -0.572847306728363,
    -0.7587595582008362,
    -0.1461995393037796,
    2.594014883041382,
    2.8055129051208496,
    3.714909315109253,
    4.624205589294434,
    5.5326457023620605,
];
const KEY_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let x_mean = x.iter().sum::<f64>() / n;
    let y_mean = y.iter().sum::<f64>() / n;
    let numerator: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - x_mean) * (yi - y_mean))
        .sum();
    let x_var: f64 = x.iter().map(|xi| (xi - x_mean).powi(2)).sum::<f64>().sqrt();
    let y_var: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum::<f64>().sqrt();
    if x_var == 0.0 || y_var == 0.0 {
        return 0.0;
    }
    numerator / (x_var * y_var)
}

/// Returns None for silent (all-zero) frames so they are excluded from
/// key classification and transition scoring.
fn get_best_frame_key(chroma: &[f64]) -> Option<usize> {
    if chroma.iter().all(|&v| v == 0.0) {
        return None;
    }
    let mut max_corr = f64::NEG_INFINITY;
    let mut best_key_idx = 0;

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
        let corr = pearson_correlation(&rotated_chroma, profile);
        if corr > max_corr {
            max_corr = corr;
            best_key_idx = i;
        }
    }
    Some(best_key_idx)
}

/// Shannon entropy weight: tonal frames (low entropy, one pitch class dominant)
/// score higher than noisy/uniform frames (high entropy).
fn chroma_entropy_weight(chroma: &[f64]) -> f64 {
    const MAX_ENTROPY: f64 = 3.584962500721156; // log2(12)
    const EPSILON: f64 = 1e-10;
    let h: f64 = chroma
        .iter()
        .map(|&p| {
            if p > 0.0 {
                -p * (p + EPSILON).log2()
            } else {
                0.0
            }
        })
        .sum();
    (MAX_ENTROPY - h).max(0.0)
}

/// Builds the static 24×24 harmonic transition scoring matrix.
/// This is independent of the audio and only needs to be computed once.
pub fn build_transition_scores() -> Vec<Vec<f64>> {
    let mut ts = vec![vec![0.0f64; 24]; 24];
    for i in 0..24 {
        ts[i][i] = 1.0;
        let root = i % 12;
        let is_major = i < 12;
        let tonic_maj = root;
        let tonic_min = root + 12;
        let dominant_maj = (root + 7) % 12;
        let dominant_min = (root + 7) % 12 + 12;
        let subdominant_maj = (root + 5) % 12;
        let subdominant_min = (root + 5) % 12 + 12;
        if is_major {
            ts[dominant_maj][tonic_maj] = 5.0;
            ts[dominant_min][tonic_maj] = 4.0;
            ts[subdominant_maj][tonic_maj] = 2.0;
        } else {
            ts[dominant_min][tonic_min] = 5.0;
            ts[dominant_maj][tonic_min] = 5.0;
            ts[subdominant_min][tonic_min] = 2.0;
        }
    }
    ts
}

/// Returns the best key AND the confidence score for that key.
pub fn analyze_track(
    chroma_sequence: &[Vec<f64>],
    transition_scores: &[Vec<f64>],
) -> (Option<Key>, f64) {
    if chroma_sequence.is_empty() {
        return (None, 0.0);
    }

    // Classify each frame; silent frames yield None and are excluded from
    // smoothing and scoring to avoid spurious cross-silence transitions.
    let raw_frame_keys: Vec<Option<usize>> = chroma_sequence
        .iter()
        .map(|frame| get_best_frame_key(frame))
        .collect();

    // Smooth frame keys with a sliding window mode over non-silent frames only.
    const SMOOTH_WINDOW: usize = 5;
    let frame_keys: Vec<Option<usize>> = (0..raw_frame_keys.len())
        .map(|i| {
            let start = i.saturating_sub(SMOOTH_WINDOW / 2);
            let end = (i + SMOOTH_WINDOW / 2 + 1).min(raw_frame_keys.len());
            let mut counts = [0usize; 24];
            let mut any = false;
            for &k in &raw_frame_keys[start..end] {
                if let Some(idx) = k {
                    counts[idx] += 1;
                    any = true;
                }
            }
            if !any {
                return None;
            }
            counts
                .iter()
                .enumerate()
                .max_by_key(|&(_, c)| c)
                .map(|(idx, _)| idx)
        })
        .collect();

    // Precompute per-frame entropy weights.
    let weights: Vec<f64> = chroma_sequence
        .iter()
        .map(|frame| chroma_entropy_weight(frame))
        .collect();

    let mut global_key_scores = vec![0.0f64; 24];
    let mut prev_key: Option<usize> = None;
    for i in 0..frame_keys.len() {
        let Some(to_key) = frame_keys[i] else {
            // Silent frame: reset adjacency so the next tonal frame doesn't
            // inherit a transition score from across the silence.
            prev_key = None;
            continue;
        };
        let w = weights[i];
        let score = match prev_key {
            Some(from_key) => transition_scores[from_key][to_key],
            None => 1.0,
        };
        global_key_scores[to_key] += score * w;
        prev_key = Some(to_key);
    }

    let weight_sum: f64 = frame_keys
        .iter()
        .enumerate()
        .filter_map(|(i, k)| k.map(|_| weights[i]))
        .sum::<f64>()
        .max(1e-10);

    let (best_idx, best_score) = global_key_scores
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap_or((0, &0.0));

    let normalized_score = best_score / weight_sum;

    let key_root = KEY_NAMES[best_idx % 12];
    let key_mode = if best_idx < 12 { "Major" } else { "Minor" };
    let key = format!("{} {}", key_root, key_mode);

    let camelot_str = match key_to_camelot(key.as_str()) {
        Some(s) => s,
        None => return (None, 0.0),
    };
    let cam_key = match Key::try_from(camelot_str) {
        Ok(k) => k,
        Err(_) => return (None, 0.0),
    };

    (Some(cam_key), normalized_score)
}
