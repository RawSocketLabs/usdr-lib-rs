use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "sdrscanner", about = "Scan a frequency range for signal peaks")]
pub struct Cli {
    /// Start frequency in Hz
    #[clap(long, default_value = "400000000")]
    pub start_freq: u32,

    /// End frequency in Hz
    #[clap(long, default_value = "520000000")]
    pub end_freq: u32,

    /// Delay between switching frequencies in milliseconds
    #[clap(long, default_value = "0")]
    pub sleep_ms: u64,

    /// Sample Rate
    #[clap(long, default_value = "2000000")]
    pub rate: u32,

    /// Number of FFT bins
    #[clap(long, default_value = "4096")]
    pub fft_size: usize,

    /// File path to IQ recording for playback
    #[clap(long, group = "file_source")]
    pub file: Option<String>,

    /// Denote that the input IQ file is raw IQ data and not a WAV file.
    #[clap(long, action, requires = "file_source")]
    pub raw: bool,

    /// Full TUI display
    #[clap(long, action)]
    pub tui: bool,

    /// Bandwidth window for detecting peaks
    #[clap(long, default_value = "12500")]
    pub bandwidth: u32,
}
