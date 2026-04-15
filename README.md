# Music Key Finder

A small library (including CLI) for track key analysis. 

Pass your track and get its Key in alphanumeric format (e.g: `11B`).

## Installation
```sh
cargo add musickeyfinder --git https://github.com/DobbiKov/musickeyfinder
```

## Usage
```sh
cargo run <path to your file>
```

## Notes
### Finetuning and Genre
Minor and Mojor profiles were finetuned on Drum and Bass Neurofunk tracks. The
library may provide innacurate results on other genres of music. 

### Other Ideas
An ML model may be trained in order to improve key detection.
