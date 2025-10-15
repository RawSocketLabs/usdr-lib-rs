use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "sdrscanner", about = "Scan a frequency range for signal peaks")]
pub struct Cli {
    // ── Input source ───────────────────────────────────────────────
    /// File path to IQ recording for playback (alternative to live scanning)
    #[arg(long, group = "file_source", help_heading = "Input source")]
    pub file: Option<String>,

    /// Center frequency for file playback
    #[arg(long, conflicts_with = "ranges", default_value = "100000000", help_heading = "Input source")]
    pub center_frequency: u32,

    /// Denote that the input IQ file is raw IQ data and not a WAV file
    #[arg(long, action, requires = "file_source", help_heading = "Input source")]
    pub raw: bool,

    /// Disable throttling of the input IQ file
    #[arg(long, action, requires = "file_source", help_heading = "Input source")]
    pub no_throttle: bool,

    // ── Scanning ───────────────────────────────────────────────────
    /// Range of frequencies to scan (space-separated pairs: "start1,end1 start2,end2")
    #[arg(long, value_parser, num_args=1.., value_delimiter=' ',
          default_value="400000000,520000000", conflicts_with = "file",
          help_heading = "Scanning")]
    pub ranges: Vec<String>,

    /// Sample rate in Hz
    #[arg(long, default_value = "2000000", help_heading = "Scanning")]
    pub rate: u32,

    /// Number of FFT bins
    #[arg(long, default_value = "4096", help_heading = "Scanning")]
    pub fft_size: usize,

    /// Scan mode (SweepAndProcess, SweepThenProcess)
    #[arg(long, default_value = "SweepAndProcess", help_heading = "Scanning")]
    pub scan_mode: String,

    /// Delay between switching frequencies in milliseconds
    #[arg(long, default_value = "50", help_heading = "Scanning")]
    pub sleep_ms: u64,

    /// Frequency ranges to ignore during scanning (space-separated pairs)
    #[arg(long, help_heading = "Scanning")]
    pub freq_ranges_to_ignore: Option<Vec<String>>,

    /// Number of scan blocks required before processing metadata
    #[arg(long, help_heading = "Scanning")]
    pub scans_before_processing: Option<usize>,

    // ── Detection ──────────────────────────────────────────────────
    /// Squelch level for detecting peaks (dB)
    #[arg(long, default_value = "-100.0", help_heading = "Detection")]
    pub squelch: f32,

    /// Bandwidth window for detecting peaks in Hz
    #[arg(long, default_value = "12500", help_heading = "Detection")]
    pub bandwidth: u32,

    /// Time to observe signal for baseline averaging during peak detection (ms)
    #[arg(long, default_value = "20", help_heading = "Detection")]
    pub peak_detection_time_ms: u32,

    /// Maximum number of bursts to attempt to recover when processing peaks
    #[arg(long, default_value = "6", help_heading = "Detection")]
    pub max_number_of_bursts: usize,

    /// Minimum number of bursts required to recover when processing peaks
    #[arg(long, default_value = "2", help_heading = "Detection")]
    pub min_number_of_bursts: usize,

    // ── Logging ────────────────────────────────────────────────────
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", help_heading = "Logging")]
    pub log_level: String,

    /// Directory for log files
    #[arg(long, default_value = "./logs", help_heading = "Logging")]
    pub log_dir: String,

    /// Enable file logging (console logging always enabled)
    #[arg(long, action, help_heading = "Logging")]
    pub log_to_file: bool,
}
