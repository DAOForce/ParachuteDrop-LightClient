use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

pub struct TotalMsgs {
    total_dump_msgs: i32,
    total_hodl_msgs: i32,
    address: String,
    target_chain: String,
    token_denom: String,
    messages: Messages,
}

pub struct Messages {
    dump_messages: Vec<DumpMessage>,
    hodl_messages: Vec<HodlMessage>,
}

pub trait MessageType: fmt::Display {
    fn get_type(&self) -> String;
}

impl MessageType for DumpMessageType {
    fn get_type(&self) -> String {
        match *self {
            DumpMessageType::BeginUnlocking => "begin_unlocking".to_string(),
            DumpMessageType::BeginUnlockingAll => "begin_unlocking_all".to_string(),
            DumpMessageType::ExitPool => "exit_pool".to_string(),
            DumpMessageType::SuperfluidUnboundLock => "superfluid_unbound_lock".to_string(),
            DumpMessageType::SuperfluidUndelegate => "superfluid_undelegate".to_string(),
        }
    }
}

#[derive(Debug, EnumIter)]
pub enum DumpMessageType {
    BeginUnlocking,
    BeginUnlockingAll,
    ExitPool,
    SuperfluidUnboundLock,
    SuperfluidUndelegate,
}

impl MessageType for HodlMessageType {
    fn get_type(&self) -> String {
        match *self {
            HodlMessageType::JoinPool => "join_pool".to_string(),
            HodlMessageType::LockTokens => "lock_tokens".to_string(),
            HodlMessageType::LockAndSuperfluidDelegate => "lock_and_superfluid_delegate".to_string(),
            HodlMessageType::SuperfluidDelegate => "superfluid_delegate".to_string(),
        }
    }
}

#[derive(Debug, EnumIter)]
pub enum HodlMessageType {
    JoinPool,
    LockTokens,
    LockAndSuperfluidDelegate,
    SuperfluidDelegate,
}

impl fmt::Display for HodlMessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for DumpMessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct DumpMessage {
    message_type: DumpMessageType,
    transaction_hash: String,
}

struct HodlMessage {
    message_type: HodlMessageType,
    transaction_hash: String,
}