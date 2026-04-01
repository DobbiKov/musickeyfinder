# musickeyfinder — Technical Report

## Overview

`musickeyfinder` is a Rust library and CLI tool for detecting the musical key
of an audio file. It outputs the result in **Camelot notation** (e.g. `8B` for
C Major), which is the standard used by DJs for harmonic mixing. The library
operates entirely offline with no machine learning model — it uses classical
signal processing and music theory.

---

## Architecture

The library is split into four modules:

| Module | Responsibility |
|---|---|
| `audio` | Decode audio files to raw PCM samples |
| `chroma` | Compute chromagram features via FFT |
| `harmonic_analyzer` | Detect key using Krumhansl-Schmuckler profiles and harmonic transition scoring |
| `types` | Key representation and Camelot mapping |

The public entry point is `analyze_key(file_path: &str) -> Option<Key>` in `lib.rs`.

---

## Pipeline

### Step 1 — Audio Loading (`audio.rs`)

Audio decoding is handled by **Symphonia**, which supports MP3, FLAC, WAV, OGG, AAC, ALAC, and more via the `all-codecs` feature.

After decoding, multi-channel audio is averaged to **mono** by summing all channels and dividing by channel count:

```
mono[i] = (ch1[i] + ch2[i] + ... + chN[i]) / N
```

The output is a `Vec<f32>` of normalized float samples plus the sample rate.

---

### Step 2 — Tuning Detection (`lib.rs`)

Before computing chroma features, the library runs a **tuning sweep** across 11 candidate A4 reference frequencies ranging from ~427 Hz to ~453 Hz (−50 to +50 cents in 10-cent steps):

```
a4_freq = 440.0 × 2^(cents_offset / 1200)
```

For each candidate tuning, the full chroma + key analysis pipeline is run. The tuning that produces the highest confidence score wins. This makes the detector robust to recordings that are not tuned to standard A4 = 440 Hz.

---

### Step 3 — Chromagram Extraction (`chroma.rs`)

For each candidate tuning, the audio is sliced into overlapping frames using:

| Parameter | Value |
|---|---|
| FFT size | 8192 samples |
| Hop size | 2048 samples (75% overlap) |
| Window function | Hann |
| Frequency ceiling | 5000 Hz |

**Per frame:**

1. Apply a Hann window to suppress spectral leakage
2. Run a forward FFT (via `rustfft`)
3. Take magnitudes of the positive-frequency half
4. Map each frequency bin to its MIDI pitch using the tuning reference:

```
midi_pitch = 12 × log₂(freq / a4_freq) + 69
```

5. Accumulate magnitude into one of 12 chroma bins via `midi_pitch % 12` (C=0, C#=1, … B=11)
6. L1-normalize the 12-bin vector so all frames are on equal footing regardless of loudness

The result is a sequence of 12-dimensional chroma vectors — one per frame — representing the relative energy of each pitch class through time.

---

### Step 4 — Per-Frame Key Classification (`harmonic_analyzer.rs`)

Each chroma frame is classified into one of 24 candidate keys (12 major + 12 minor) using the **Krumhansl-Schmuckler key-finding algorithm** (1990).

Two reference profiles encode the perceptual salience of each scale degree within a key:

```
Major: [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88]
Minor: [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17]
```

For each candidate key (root × mode combination):

1. Rotate the chroma vector so the candidate root aligns with index 0
2. Compute the dot product of the rotated chroma with the appropriate profile
3. The candidate with the highest dot product is the frame's local key estimate

This is a simplified version of K-S that uses a dot product rather than the full Pearson correlation — it is faster but sensitive to chroma magnitude, not just shape.

---

### Step 5 — Frame Smoothing (`harmonic_analyzer.rs`)

Raw per-frame key estimates are noisy (a single transient can flip the local estimate). A **5-frame sliding window mode** is applied before the global scoring step:

```
smoothed_key[i] = mode(raw_frame_keys[i-2 .. i+2])
```

This suppresses isolated outliers while preserving genuine key changes.

---

### Step 6 — Global Key Scoring via Harmonic Transitions (`harmonic_analyzer.rs`)

Rather than taking a simple vote, the library scores each of the 24 global key candidates using the **harmonic weight of each frame transition**.

A 24×24 transition score matrix is pre-built using music theory:

| Relationship | Score |
|---|---|
| Same key (identity) | 1.0 |
| Dominant (V) major → tonic major | 5.0 |
| Dominant minor → tonic major | 4.0 |
| Dominant → tonic minor | 5.0 |
| Subdominant (IV) → tonic | 2.0 |

For each global candidate key `K`, the score accumulates as:

```
score(K) += transition_score[frame[i-1] → frame[i]]   if frame[i] == K
```

The intuition: a frame that arrives via a dominant resolution is much stronger evidence for the tonic key than a frame that just appears in isolation.

The raw score is then **normalized by frame count** so it represents a per-frame average and is comparable across tracks of different lengths.

---

### Step 7 — Camelot Mapping (`types.rs`)

The winning key (e.g. `G Major`) is mapped to its **Camelot wheel position** (e.g. `9B`) via a static lookup table. The result is returned as a `Key` struct holding a number (1–12) and a letter (A = minor, B = major).

---

## Key Design Decisions

### Why Camelot notation?
Camelot encoding makes harmonic compatibility trivial for DJs: compatible keys share a number or are adjacent on the wheel. The library targets DJ tooling rather than general music analysis.

### Why a tuning sweep?
Many electronic music productions are tuned slightly sharp or flat. Without correcting for this, a fixed A4 = 440 Hz reference causes systematic chroma misalignment, shifting energy into the wrong pitch bins and degrading accuracy.

### Why dot product instead of Pearson correlation?
The dot product is faster and sufficient in practice since the chroma vectors are already L1-normalized. A Pearson correlation would additionally normalize the profile vectors, which are fixed constants — the ranking of keys would be nearly identical.

---

## Limitations

| Limitation | Detail |
|---|---|
| 5000 Hz frequency ceiling | Cuts off 3+ octaves of piano range; high harmonics are discarded |
| Linear magnitude accumulation | Louder notes dominate; no log compression or harmonic/percussive separation |
| One-frame transition lookback | Not a full HMM/Viterbi decode; long-range harmonic context is not modelled |
| No modulation handling | Assumes a single static key throughout the track |
| Dot product, not correlation | Sensitive to frame-level loudness variation despite L1 normalization of chroma |

---

## Dependencies

| Crate | Purpose |
|---|---|
| `symphonia` | Multi-format audio decoding (MP3, FLAC, WAV, OGG, AAC, ALAC, …) |
| `rustfft` | FFT computation |
| `realfft` | Real-valued FFT utilities (available, not currently used in hot path) |
| `clap` | CLI argument parsing |
| `thiserror` | Ergonomic error type derivation |
