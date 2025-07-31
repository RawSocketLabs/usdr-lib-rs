use futuresdr::blocks::ChannelSink;
use futuresdr::blocks::ChannelSource;
use futuresdr::blocks::VectorSink;
use futuresdr::futures::channel::mpsc;
use futuresdr::runtime::Flowgraph;
use futuresdr::blocks::XlatingFirBuilder;
use futuresdr::blocks::VectorSource;
use futuresdr::blocks::Head;
use futuresdr::blocks::FileSink;
use futuresdr::macros::connect;
use futuresdr::runtime::Runtime;
use futuresdr::runtime::Error;
use futuresdr::runtime::TypedBlock;
use sdr::device::file::WavFile;
use sdr::SdrControl;
use sdr::{IQSample, IQBlock};
use rustradio::fir::low_pass_complex;
use rustradio::window::WindowType;
use rustradio;
use rustfft::num_traits::AsPrimitive;
use rustradio::{blocks::QuadratureDemod, graph::{Graph, GraphRunner}, mtgraph::MTGraph};
use std::{fs, io};

use rustfft::num_complex::Complex32;
use std::f32::consts::PI;


struct DmrProcessor {
  pub source_tx: mpsc::Sender<Box<[IQSample]>>,
  pub sink_rx: mpsc::Receiver<Box<[IQSample]>>,
  source: TypedBlock<ChannelSource<IQSample>>,
  sink: TypedBlock<ChannelSink<IQSample>>,
}

impl DmrProcessor {
  pub fn new() -> Self {
    let (mut src_tx, src_rx) = mpsc::channel::<Box<[IQSample]>>(4096);
    let (mut sink_tx, sink_rx) = mpsc::channel::<Box<[IQSample]>>(4096);
    DmrProcessor {
      source_tx: src_tx,
      sink_rx: sink_rx,
      source: ChannelSource::new(src_rx),
      sink: ChannelSink::new(sink_tx),
    } 
  }

  pub fn run(self) -> Result<(), Error> {
        let mut fg = Flowgraph::new();

        let src = self.source;
				let snk = self.sink;

        // let taps = low_pass(2000000.0, 12500.0, 2000.0, &WindowType::Hamming);
        // let filter = XlatingFirBuilder::with_taps(taps, 16, -722832.0, 2000000.0);

        // connect!(fg, src > filter > snk);

        // Runtime::new().run(fg)?;
    Ok(())
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

pub fn do_it() {
        let mut g = MTGraph::new();
        // let (src, prev) = rustradio::blocks::VectorSource::new(data);
        let (src, prev) = rustradio::blocks::FileSource::new("shifted.wav").unwrap();

        let taps = low_pass_complex(2000000.0, 12500.0, 2000.0, &WindowType::Hamming);

        let (lowpass, prev) = rustradio::fir::FirFilter::builder(&taps).deci(16).build(prev);
        let (fm_demod, prev) = QuadratureDemod::new(prev, 4.14466);
        let (resample, prev) = rustradio::blocks::RationalResampler::new(prev, 48000, 125000).unwrap();
        let (mul_const, prev) = rustradio::blocks::MultiplyConst::new(prev, 32767.0);
        let snk = rustradio::blocks::VectorSink::new(prev, 8695740);
        let hooker = snk.hook();
        g.add(Box::new(src));
        g.add(Box::new(lowpass));
        g.add(Box::new(fm_demod));
        g.add(Box::new(resample));
        g.add(Box::new(mul_const));
        g.add(Box::new(snk));
        g.run().unwrap();

        let samples: Vec<u8> = hooker.data().samples().iter().map(|f| *f as i16).flat_map(|i| i.to_le_bytes()).collect();

        fs::write("winner.wav", &samples).unwrap();
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn test_process() -> Result<(), Error> {
        // let mut wavfile = WavFile::new("iq.wav");
 
        // let mut data = wavfile.read_raw_iq(362332376).unwrap();
        // println!("Read data");
        // freq_shift_num_complex(&mut data, 2000000.0, -722832.0);

        let mut g = MTGraph::new();
        // let (src, prev) = rustradio::blocks::VectorSource::new(data);
        let (src, prev) = rustradio::blocks::FileSource::new("shifted.wav").unwrap();

        let taps = low_pass_complex(2000000.0, 12500.0, 2000.0, &WindowType::Hamming);

        let (lowpass, prev) = rustradio::fir::FirFilter::builder(&taps).deci(16).build(prev);
        let (fm_demod, prev) = QuadratureDemod::new(prev, 4.14466);
        let (resample, prev) = rustradio::blocks::RationalResampler::new(prev, 48000, 125000).unwrap();
        let (mul_const, prev) = rustradio::blocks::MultiplyConst::new(prev, 32767.0);
        let snk = rustradio::blocks::VectorSink::new(prev, 8695740);
        let hooker = snk.hook();
        g.add(Box::new(src));
        g.add(Box::new(lowpass));
        g.add(Box::new(fm_demod));
        g.add(Box::new(resample));
        g.add(Box::new(mul_const));
        g.add(Box::new(snk));
        g.run().unwrap();

        let samples: Vec<u8> = hooker.data().samples().iter().map(|f| *f as i16).flat_map(|i| i.to_le_bytes()).collect();

        fs::write("winner.wav", &samples).unwrap();

				Ok(())
    }
}
