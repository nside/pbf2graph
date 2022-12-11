use clap::Parser;
pub use pbf2graph::{from_pbf, write_csv, RoadGraph};
use std::error::Error;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    pbf_file: String,

    /// Number of times to greet
    #[arg(short, long)]
    output_dir: String,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let pbf_path = Path::new(&args.pbf_file);
    let graph = from_pbf(pbf_path)?;

    let csv_dir = Path::new(&args.output_dir);
    write_csv(&graph, csv_dir)?;

    Ok(())
}
