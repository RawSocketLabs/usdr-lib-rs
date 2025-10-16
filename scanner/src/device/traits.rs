// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use sdr::SdrControl;

use crate::device::{DevChannels, SampleContext};

pub trait Sample<T: SdrControl> {
    fn sample(&mut self, channels: DevChannels, ctx: SampleContext);
}
