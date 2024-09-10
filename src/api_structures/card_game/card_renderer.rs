pub enum DynCodeSegmentType {
    RawText,
    Code,
}

pub struct DynCode {
    segment_type: DynCodeSegmentType,
    raw_string: String,
}
