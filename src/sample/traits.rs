use sdr::SdrControl;

use crate::sample::{SampleChannels, SampleParams};

pub trait Sample<T: SdrControl> {
    fn sample(&mut self, channels: SampleChannels, params: SampleParams);
}
