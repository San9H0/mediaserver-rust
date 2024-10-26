#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NALUType {
    Undefined,
    NonIDR = 1,
    DataPartitionA = 2,
    DataPartitionB = 3,
    DataPartitionC = 4,
    IDR = 5,
    SEI = 6,
    SPS = 7,
    PPS = 8,
    AccessUnitDelimiter = 9,
    EndOfSequence = 10,
    EndOfStream = 11,
    FillerData = 12,
    SPSExtension = 13,
    Prefix = 14,
    SubsetSPS = 15,
    Reserved16 = 16,
    Reserved17 = 17,
    Reserved18 = 18,
    SliceLayerWithoutPartitioning = 19,
    SliceExtension = 20,
    SliceExtensionDepth = 21,
    Reserved22 = 22,
    Reserved23 = 23,
    STAPA = 24,
    STAPB = 25,
    MTAP16 = 26,
    MTAP24 = 27,
    FUA = 28,
    FUB = 29,
}

impl NALUType {
    pub fn from_byte(byte: u8) -> NALUType {
        match byte & 0x1F {
            1 => NALUType::NonIDR,
            2 => NALUType::DataPartitionA,
            3 => NALUType::DataPartitionB,
            4 => NALUType::DataPartitionC,
            5 => NALUType::IDR,
            6 => NALUType::SEI,
            7 => NALUType::SPS,
            8 => NALUType::PPS,
            9 => NALUType::AccessUnitDelimiter,
            10 => NALUType::EndOfSequence,
            11 => NALUType::EndOfStream,
            12 => NALUType::FillerData,
            13 => NALUType::SPSExtension,
            14 => NALUType::Prefix,
            15 => NALUType::SubsetSPS,
            16 => NALUType::Reserved16,
            17 => NALUType::Reserved17,
            18 => NALUType::Reserved18,
            19 => NALUType::SliceLayerWithoutPartitioning,
            20 => NALUType::SliceExtension,
            21 => NALUType::SliceExtensionDepth,
            22 => NALUType::Reserved22,
            23 => NALUType::Reserved23,
            24 => NALUType::STAPA,
            25 => NALUType::STAPB,
            26 => NALUType::MTAP16,
            27 => NALUType::MTAP24,
            28 => NALUType::FUA,
            29 => NALUType::FUB,
            _ => NALUType::Undefined,
        }
    }
}