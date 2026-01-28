// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use sdr::{Burst, IQBlock};
use std::f32::consts::PI;
use sdr::decode::dmr::burst::{DataInfo, FeatureSetID, FullLinkControlData, TerminatorWithLinkControl, VoiceLinkControlHeader};
use sdr::dsp::{low_pass_taps, polyphase_decimating_fir_complex, rational_resample, WindowType};
use sdr::dsp::fm::quadrature_demod;
use shared::{MetadataGroupVoice, Message, DmrMetadata, MetadataCSBK};

const CHANNEL_RATE: usize = 125000;
pub const DMR_BANDWIDTH: usize = 12500;
pub const AUDIO_RATE: usize = 48000;
const SYMBOL_RATE: usize = 4800;

pub fn preprocess_dmr_samples(data: IQBlock, rate: u32) -> Vec<i16> {
    let decimation = rate as usize / CHANNEL_RATE;
    // NOTE: Slightly increasing gain by a factor of 1.2 seems to improve the signal quality.
    //  we should investigate this further to identify the ideal gain value.
    let gain = (CHANNEL_RATE as f32 / (2.0 * PI * SYMBOL_RATE as f32)) * 1.2;
    let taps = low_pass_taps(rate as f32, DMR_BANDWIDTH as f32, DMR_BANDWIDTH as f32, WindowType::Hamming);

    let processed_samples = polyphase_decimating_fir_complex(data.inner(), taps, decimation);
    let processed_samples = quadrature_demod(processed_samples, gain);
    let processed_samples = rational_resample(processed_samples, AUDIO_RATE, CHANNEL_RATE);
    let processed_samples = processed_samples.iter().map(|sample| (sample * i16::MAX as f32) as i16).collect();

    processed_samples
}

pub trait MetadataGroupVoiceCreator {
    fn new(fid: FeatureSetID, group: u32, source: u32) -> Self;
}

impl MetadataGroupVoiceCreator for MetadataGroupVoice {
    fn new(fid: FeatureSetID, group: u32, source: u32) -> Self {
        Self { fid, group, source }
    }
}

pub trait ScanDmrMetadataExt {
    fn update_from_burst(&mut self, burst: Burst);
}

impl ScanDmrMetadataExt for DmrMetadata {


    fn update_from_burst(&mut self, burst: Burst) {
        match burst {
            Burst::Data(data_burst) => {
                self.syncs.insert(data_burst.pattern);
                self.sync_count += 1;
                self.color_codes.insert(data_burst.slot_type.color_code().value());
                self.slot_data_types.insert(data_burst.slot_type.data_type());
                match data_burst.info {
                    DataInfo::VoiceLinkControlHeader(VoiceLinkControlHeader { link_control, .. }) |
                    DataInfo::TerminatorWithLinkControl(TerminatorWithLinkControl { link_control, .. }) => {
                        match link_control.data {
                            FullLinkControlData::GroupVoiceChannelUser(data) => {
                                self.messages.insert(Message::GroupVoice(MetadataGroupVoice::new(link_control.feature_set_id, data.group_address().value(), data.source_address().value())));
                            }
                            FullLinkControlData::GPSInfo(data) => {
                                println!("GPS Info: {:?}", data);
                            }
                            FullLinkControlData::TalkerAliasBlock1(data) => {
                                println!("Talker Alias Block 1: {:?}", data);
                            }
                            FullLinkControlData::TalkerAliasBlock2(data) => {
                                println!("Talker Alias Block 2: {:?}", data);
                            }
                            FullLinkControlData::TalkerAliasBlock3(data) => {
                                println!("Talker Alias Block 3: {:?}", data);
                            }
                            FullLinkControlData::TalkerAliasHeader(data) => {
                                println!("Talker Alias Header: {:?} {:?} {:?}", data.data_format(), data.data_length(), data.data());

                            }
                            FullLinkControlData::UnitToUnitVoiceChannelUser(data) => {
                                println!("Unit To Unit Voice Channel User: {:?}", data);
                            }
                            _ => {}
                        }
                    }
                    DataInfo::ControlSignalingBlock(csb) => {
                        self.messages.insert(Message::CSBK(MetadataCSBK {fid: csb.feature_set_id}));
                    }
                    _ => {}
                }
            }
            Burst::Voice(voice_burst) => {
                self.syncs.insert(voice_burst.pattern);
                self.sync_count += 1;
            }
            _ => {}
        }
    }
}
