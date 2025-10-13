use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "sdrscanner", about = "Scan a frequency range for signal peaks")]
pub struct Cli {
    /// Range of frequencies to accept
    #[clap(long, value_parser, num_args=1.., value_delimiter=' ', default_value="400000000,520000000", conflicts_with = "file")]
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

    #[clap(long, action, requires = "file_source")]
    pub no_throttle: bool,

    #[clap(long, conflicts_with = "ranges", default_value = "100000000")]
    pub center_frequency: u32,

    /// Bandwidth window for detecting peaks
    #[clap(long, default_value = "12500")]
    pub bandwidth: u32,

    /// Scan Mode
    #[clap(long, default_value = "SweepAndProcess")]
    pub scan_mode: String,

    /// Time to observe signal for baseline averaging during peak detection (milliseconds)
    #[clap(long, default_value = "5")]
    pub peak_detection_time_ms: u32,

    /// Max number of bursts to attempt to recover when processing peaks for metadata
    #[clap(long, default_value = "5")]
    pub max_number_of_bursts: usize,

    /// Number of blocks required for processing metadata
    #[clap(long)]
    pub scans_before_processing: Option<usize>,

    /// Number of blocks required for processing metadata
    #[clap(long)]
    pub freq_ranges_to_ignore: Option<Vec<String>>,

    /// Log level (trace, debug, info, warn, error)
    #[clap(long, default_value = "info")]
    pub log_level: String,

    /// Directory for log files
    #[clap(long, default_value = "./logs")]
    pub log_dir: String,

    /// Enable file logging (console logging always enabled)
    #[clap(long, action)]
    pub log_to_file: bool,

}
