use crate::robust_modbus::Word;
use tokio_modbus::{Address, Quantity};
pub(crate) struct Coils {
    pub addr: Address,
    pub cnt: Quantity,
}

pub(crate) struct DiscreteInputs {
    pub addr: Address,
    pub cnt: Quantity,
}

pub(crate) struct HoldingRegisters {
    pub addr: Address,
    pub cnt: Quantity,
}

pub(crate) struct InputRegisters {
    pub addr: Address,
    pub cnt: Quantity,
}

pub(crate) struct ReadWriteMultipleRegisters<'a> {
    pub read_addr: Address,
    pub read_count: Quantity,
    pub write_addr: Address,
    pub write_data: &'a [Word],
}
