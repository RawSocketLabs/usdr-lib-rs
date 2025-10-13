use sdr::Freq;

pub enum DevMsg {
    ChangeFreq(Freq),
    DeviceFreqUpdated
}
