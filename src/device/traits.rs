use sdr::SdrControl;

use crate::device::SampleArgs;

pub trait Sample<T: SdrControl> {
    fn sample(&mut self, sample_args: SampleArgs);
}
