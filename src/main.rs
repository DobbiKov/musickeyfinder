use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(help = "Path to the track")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    musickeyfinder::analyze_key(cli.path.as_path().to_str().unwrap());
}
