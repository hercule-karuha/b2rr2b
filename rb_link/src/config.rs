pub type MsgSizeType = u32;
pub const MSG_SIZE_BYTES: usize = std::mem::size_of::<MsgSizeType>();
pub const SHUT_DOWN_ID: u32 = 0xFFFFFFFF;
