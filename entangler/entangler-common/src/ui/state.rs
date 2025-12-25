use alloc::{string::String, vec::Vec};

#[derive(Default, Debug)]
pub struct State {
    bearings: Vec<(String, (u64, u64))>,
}
