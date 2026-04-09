use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(help = "Path to the track")]
    path: PathBuf,

    #[arg(long, help = "Export chroma frames at best tuning to a CSV file")]
    export: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let path = cli.path.as_path().to_str().unwrap();

    if let Some(out) = cli.export {
        musickeyfinder::export_chroma(path, out.as_path().to_str().unwrap());
    } else {
        musickeyfinder::analyze_key(path);
    }
}
