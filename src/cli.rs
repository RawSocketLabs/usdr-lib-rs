use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "sdrscanner", about = "Scan a frequency range for signal peaks")]
pub struct Cli {
    /// Range of frequencies to accept
    #[clap(long, value_parser, num_args=1.., value_delimiter=' ', default_value="400000000,520000000")]
    pub ranges: Vec<String>,

    /// Delay between switching frequencies in milliseconds
    #[clap(long, default_value = "50")]
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

    /// Bandwidth window for detecting peaks
    #[clap(long, default_value = "12500")]
    pub bandwidth: u32,
}
