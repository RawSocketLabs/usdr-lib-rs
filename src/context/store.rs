// STD LIB
use std::collections::HashMap;

// THIRD PARTY
use chrono::{DateTime, Utc};

// VENDOR CRATES
use sdr::{FreqRange, FreqSample};

#[derive(Default)]
pub struct StoredInfo {
    observations: HashMap<FreqRange, Vec<Observation>>
}

#[derive(Clone)]
pub struct Observation {
    pub timestamp: DateTime<Utc>,
    pub sample: FreqSample,
}
