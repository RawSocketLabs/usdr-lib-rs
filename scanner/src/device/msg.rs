// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use sdr::sample::Freq;

pub enum DevMsg {
    ChangeFreq(Freq),
}
