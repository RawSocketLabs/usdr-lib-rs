use crate::context::{DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE, DEFAULT_BLOCKS_REQUIRED_FOR_METADATA};

pub struct BlockParameters {
    process: bool,
    num_required_for_average: usize,
    num_required_for_metadata: usize,
}

impl BlockParameters {

    pub fn new(num_required_for_average: Option<usize>, num_required_for_metadata: Option<usize>) -> Self {
        Self {
            process: true,
            num_required_for_average: num_required_for_average.unwrap_or(DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE),
            num_required_for_metadata: num_required_for_metadata.unwrap_or(DEFAULT_BLOCKS_REQUIRED_FOR_METADATA),
        }

    }
    pub fn default() -> Self {
        Self {
            process: true,
            num_required_for_average: DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE,
            num_required_for_metadata: DEFAULT_BLOCKS_REQUIRED_FOR_METADATA,
        }
    }
}
