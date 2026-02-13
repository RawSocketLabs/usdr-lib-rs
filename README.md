# Shrike - SDR Scanner

A software-defined radio (SDR) spectrum scanner application written in Rust, featuring real-time signal detection and a terminal user interface (TUI).

## Overview

Shrike scans frequency ranges using SDR hardware to detect and analyze RF signals. It performs FFT-based spectrum analysis with configurable peak detection and squelch settings. The scanner can automatically locate DMR (Digital Mobile Radio) signals and perform demodulation and decoding of DMR transmissions.

### Key Features

- **Multi-device support**: Works with RTL-SDR, BladeRF, and uSDR devices
- **Real-time spectrum scanning**: Configurable sample rates and FFT sizes
- **Peak detection**: Automatic signal detection with adjustable squelch and bandwidth
- **DMR support**: Locate, demodulate, and decode DMR digital radio signals
- **Terminal UI**: Real-time visualization via ratatui-based TUI
- **IQ file playback**: Analyze recorded IQ samples from WAV or raw files

## Project Structure

This is a Cargo workspace containing three crates:

| Crate | Description |
|-------|-------------|
| `shared` | Common utilities and data structures |
| `scanner` (`sdrscanner`) | Core SDR scanning and signal processing |
| `tui` | Terminal user interface for visualization |

## Prerequisites

### Source Code Layout

Shrike depends on the `libsdr` library, which must be cloned adjacent to the Shrike source directory.

Your workspace directory should be structured as follows:

```
workspace/
├── shrike/
├── libsdr/
```

### Device-Specific Libraries

Install the libraries for the SDR device(s) you plan to use:

#### RTL-SDR

**macOS (Homebrew):**
```bash
brew install librtlsdr
```

**Linux:**

Follow the [Getting Started on Linux Guide](https://www.rtl-sdr.com/rtl-sdr-quick-start-guide/).

#### BladeRF

Follow the installation instructions from the [BladeRF project wiki](https://github.com/Nuand/bladeRF/wiki#user-content-bladeRF_software_buildinstallation).

#### USDR

Follow the installation instructions from the [uSDR documentation](https://docs.wsdr.io/software/install.html).

## Building

### Feature Flags

The scanner supports different SDR devices via feature flags:

| Feature | Description |
|---------|-------------|
| `rtlsdr` | RTL-SDR device support (default) |
| `bladerf` | BladeRF device support |
| `usdr` | USDR device support |

### Default Build (RTL-SDR)

```bash
cargo build --release
```

### Building with Specific Device Support

#### Single Device

```bash
# RTL-SDR only (default)
cargo build --release --features rtlsdr

# BladeRF only
cargo build --release --no-default-features --features bladerf

# USDR only
cargo build --release --no-default-features --features usdr
```

#### Multiple Devices

```bash
# RTL-SDR and BladeRF
cargo build --release --features "rtlsdr,bladerf"

# RTL-SDR and USDR
cargo build --release --features "rtlsdr,usdr"

# BladeRF and USDR
cargo build --release --no-default-features --features "bladerf,usdr"

# All supported devices
cargo build --release --features "rtlsdr,bladerf,usdr"
```

### Building Specific Workspace Members

```bash
# Build only the scanner
cargo build --release -p sdrscanner --features rtlsdr

# Build only the TUI
cargo build --release -p tui
```

## Running

### Scanner

Run the scanner with default settings:

```bash
cargo run --release -p sdrscanner -- --ranges "400000000,520000000"
```

### Common CLI Options

| Option | Default | Description |
|--------|---------|-------------|
| `--ranges` | `400000000,520000000` | Frequency range(s) to scan (Hz) |
| `--rate` | `2000000` | Sample rate in Hz |
| `--fft-size` | `4096` | Number of FFT bins |
| `--squelch` | `-100.0` | Squelch level for peak detection (dB) |
| `--bandwidth` | `12500` | Bandwidth window for peak detection (Hz) |
| `--scan-mode` | `SweepAndProcess` | Scanning mode |
| `--sleep-ms` | `50` | Delay between frequency switches (ms) |

### IQ File Playback

Analyze a recorded IQ file instead of live scanning:

```bash
# WAV file
cargo run --release -p sdrscanner -- --file recording.wav --center-frequency 100000000

# Raw IQ file
cargo run --release -p sdrscanner -- --file recording.iq --raw --center-frequency 100000000
```

### TUI

Launch the terminal user interface:

```bash
cargo run --release -p tui
```

## Configuration Examples

### Scan Multiple Frequency Ranges

```bash
cargo run --release -p sdrscanner -- --ranges "144000000,148000000 430000000,440000000"
```

### Ignore Specific Frequencies

```bash
cargo run --release -p sdrscanner -- \
  --ranges "400000000,520000000" \
  --freq-ranges-to-ignore "460000000,462000000"
```

## Logging

Control log verbosity with the `--log-level` option:

```bash
cargo run --release -p sdrscanner -- --log-level debug
```

Available levels: `trace`, `debug`, `info`, `warn`, `error`

Logs are written to the `logs/` directory with daily rotation.

## License

Metrea LLC Intellectual Property. Originally developed by Raw Socket Labs LLC.
