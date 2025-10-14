use sdr::sample::Freq;

pub enum DevMsg {
    ChangeFreq(Freq),
}
