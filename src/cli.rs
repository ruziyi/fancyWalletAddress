//! src/cli.rs

use clap::Parser;

/// A high-performance Tron (TRX) vanity address generator.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Comma-separated list of desired address suffixes (e.g., "8888,COOL,Tron").
    #[arg(long, required = true, value_delimiter = ',')]
    pub suffixes: Vec<String>,

    /// The number of addresses to find before exiting.
    #[arg(long, default_value_t = 1)]
    pub count: usize,

    /// Number of CPU threads to use. Defaults to all available cores.
    #[arg(long)]
    pub threads: Option<usize>,

    /// (Placeholder) Attempt to use GPU for acceleration.
    #[arg(long, default_value_t = false)]
    pub gpu: bool,

    /// Show real-time calculation speed (checks per second).
    #[arg(long, default_value_t = false)]
    pub show_speed: bool,
}
