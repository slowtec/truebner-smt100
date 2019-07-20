use serialport::{DataBits, FlowControl, Parity, StopBits};

pub const BAUD_RATE: u32 = 9600;
pub const DATA_BITS: DataBits = DataBits::Eight;
pub const STOP_BITS: StopBits = StopBits::One;
pub const PARITY: Parity = Parity::Even;
pub const FLOW_CONTROL: FlowControl = FlowControl::None;
