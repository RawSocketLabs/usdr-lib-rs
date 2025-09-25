use std::f32::consts::PI;
use rustradio::{
    blocks::QuadratureDemod, fir::low_pass_complex, graph::GraphRunner, mtgraph::MTGraph,
    window::WindowType,
};
use sdr::{FreqSample, IQBlock};

const CHANNEL_RATE: usize = 125000;
const DMR_BANDWIDTH: usize = 12500;
const AUDIO_RATE: usize = 48000;
const SYMBOL_RATE: usize = 4800;

pub struct SignalMetadata {
    pub timestamp: i64,
    pub peak: FreqSample,
    pub processed_samples: Vec<i16>,
}

/// Struct for handling DMR processing
pub struct SignalPreProcessor {
    graph: MTGraph,
    snk_hook: rustradio::vector_sink::Hook<f32>,
}

impl SignalPreProcessor {
    /// Creates a new DmrProcessor with the given IQ data.  This will construct a rustradio flowgraph for transforming IQ data into a format suitable for DMR processing.
    ///
    /// It is expected that the signal of interest is already centered within the input IQBlock.
    pub fn new(data: IQBlock, rate: f32) -> Self {
        let decimation_factor = rate as usize / CHANNEL_RATE;
        let gain = CHANNEL_RATE as f32 / (2.0 * PI * SYMBOL_RATE as f32);

        let mut g = MTGraph::new();
        let (src, prev) = rustradio::blocks::VectorSource::new(data);

        let taps = low_pass_complex(rate, DMR_BANDWIDTH as f32, 2000.0, &WindowType::Hamming);

        let (lowpass, prev) = rustradio::fir::FirFilter::builder(&taps)
            .deci(decimation_factor)
            .build(prev);
        let (fm_demod, prev) = QuadratureDemod::new(prev, gain);
        let (resample, prev) =
            rustradio::blocks::RationalResampler::new(prev, AUDIO_RATE, CHANNEL_RATE).unwrap();
        let (mul_const, prev) = rustradio::blocks::MultiplyConst::new(prev, i16::MAX as f32);
        let snk = rustradio::blocks::VectorSink::new(prev, 8695740);
        let snk_hook = snk.hook();
        g.add(Box::new(src));
        g.add(Box::new(lowpass));
        g.add(Box::new(fm_demod));
        g.add(Box::new(resample));
        g.add(Box::new(mul_const));
        g.add(Box::new(snk));
        SignalPreProcessor { graph: g, snk_hook }
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
