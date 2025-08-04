use rustfft::num_complex::Complex32;
use rustradio;
use rustradio::fir::low_pass_complex;
use rustradio::window::WindowType;
use rustradio::{blocks::QuadratureDemod, graph::GraphRunner, mtgraph::MTGraph};
use sdr::{IQBlock, SdrError};
use std::f32::consts::PI;

/// Struct for handling DMR processing
pub struct DmrProcessor {
    graph: MTGraph,
    snk_hook: rustradio::vector_sink::Hook<f32>,
}

impl DmrProcessor {
    /// Creates a new DmrProcessor with the given IQ data.  This will construct a rustradio flowgraph for transforming IQ data into a format suitable for DMR processing.
    ///
    /// It is expected that the signal of interest is already centered within the input IQBlock.
    pub fn new(data: IQBlock) -> Self {
        let mut g = MTGraph::new();
        let (src, prev) = rustradio::blocks::VectorSource::new(data);

        let taps = low_pass_complex(2000000.0, 12500.0, 2000.0, &WindowType::Hamming);

        let (lowpass, prev) = rustradio::fir::FirFilter::builder(&taps)
            .deci(16)
            .build(prev);
        let (fm_demod, prev) = QuadratureDemod::new(prev, 3.5);
        let (resample, prev) =
            rustradio::blocks::RationalResampler::new(prev, 48000, 125000).unwrap();
        let (mul_const, prev) = rustradio::blocks::MultiplyConst::new(prev, 32767.0);
        let snk = rustradio::blocks::VectorSink::new(prev, 8695740);
        let snk_hook = snk.hook();
        g.add(Box::new(src));
        g.add(Box::new(lowpass));
        g.add(Box::new(fm_demod));
        g.add(Box::new(resample));
        g.add(Box::new(mul_const));
        g.add(Box::new(snk));
        DmrProcessor { graph: g, snk_hook }
    }

    /// Runs the DMR processing graph.
    pub fn run(&mut self) -> Result<(), rustradio::Error> {
        self.graph.run()
    }

    /// Retrieves the processed samples from the sink hook.
    pub fn get_processed_samples(&mut self) -> Vec<i16> {
        self.snk_hook
            .data()
            .samples()
            .iter()
            .map(|f| *f as i16)
            .collect()
    }
}

fn freq_shift_num_complex(samples: &mut [Complex32], fs: f32, f_off: f32) {
    const TWO_PI: f32 = 2.0 * PI;
    let phase_inc = -TWO_PI * f_off / fs;
    let mut phase = 0.0_f32;

    for s in samples.iter_mut() {
        // build unit-phasor via polar form
        let phasor = Complex32::from_polar(1.0, phase);
        *s *= phasor;

        phase += phase_inc;
        // wrap into (-π, π]
        if phase > PI {
            phase -= TWO_PI;
        } else if phase <= -PI {
            phase += TWO_PI;
        }
    }
}

#[cfg(test)]
mod unit {
    use std::{fs, io::Write};

    use sdr::{SdrControl, device::file::WavFile};

    use super::*;

    use rust_dsdcc::*;

    #[test]
    fn test_process() {
        let mut wavfile = WavFile::new("iq.wav");

        let mut data = wavfile.read_raw_iq(wavfile.reader.len() as usize).unwrap();
        freq_shift_num_complex(data.as_mut_slice(), 2000000.0, -722832.0);
        let mut processor = DmrProcessor::new(data);
        processor.run().unwrap();

        let dsddecoder = DSDDecoder::new().unwrap();

        dsddecoder.set_quiet();
        
        for sample in processor.get_processed_samples() {
            dsddecoder.run(sample);
        }
    }
}
