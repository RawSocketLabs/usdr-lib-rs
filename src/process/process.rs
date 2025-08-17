use rustfft::num_complex::Complex32;
use sdr::IQBlock;
use std::f32::consts::PI;

pub fn get_new_iq_block(
    iq_block_rx: &mut tokio::sync::mpsc::Receiver<IQBlock>,
    iq_blocks: &mut Vec<IQBlock>,
) -> Result<(), tokio::sync::mpsc::error::TryRecvError> {
    let new_iq_block_result = iq_block_rx.try_recv();
    match new_iq_block_result {
        Ok(iq_block) => {
            iq_blocks.push(iq_block);
            Result::Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn freq_shift_num_complex(samples: &mut [Complex32], fs: f32, f_off: f32) {
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
    use sdr::{SdrControl, device::file::WavFile};

    use crate::process::DmrProcessor;

    use super::*;

    use rust_dsdcc::{ffi::DSDDecodeMode, *};

    #[test]
    fn test_process() {
        let mut wavfile = WavFile::new("../../resources/iq.wav");

        let mut data = wavfile.read_raw_iq(wavfile.reader.len() as usize).unwrap();
        freq_shift_num_complex(data.as_mut_slice(), 2000000.0, -722832.0);
        let mut processor = DmrProcessor::new(data);
        processor.run().unwrap();

        let dsddecoder = DSDDecoder::new();

        // dsddecoder.set_quiet();
        dsddecoder.set_decode_mode(DSDDecodeMode::DSDDecodeNXDN48, false);
        dsddecoder.set_decode_mode(DSDDecodeMode::DSDDecodeNXDN96, false);
        dsddecoder.set_decode_mode(DSDDecodeMode::DSDDecodeDMR, true);

        let mut last0_text = dsddecoder.get_slot_0_text();
        let mut last1_text = dsddecoder.get_slot_1_text();
        let mut i = 0;
        for sample in processor.get_processed_samples() {
            dsddecoder.run(sample);
            let text0 = dsddecoder.get_slot_0_text();
            let text1 = dsddecoder.get_slot_1_text();
            if text0 != last0_text || text1 != last1_text {
                eprintln!(
                    "{i:>10} Text: {} | {} | {} | {} | {} | {}",
                    text0,
                    text1,
                    dsddecoder.get_sync_type(),
                    dsddecoder.get_frame_type_text(),
                    dsddecoder.get_frame_subtype_text(),
                    dsddecoder.get_station_type(),
                );
                last0_text = text0;
                last1_text = text1;
            }
            i += 1;
        }

        println!("Processed {} samples", i);
    }
}
