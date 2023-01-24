use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Bytes;
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
            DumpMessageType::ExitPool => "exit_pool".to_string(),
        }
    }
}

#[derive(Debug, EnumIter)]
pub enum DumpMessageType {
    ExitPool,
}

impl MessageType for HodlMessageType {
    fn get_type(&self) -> String {
        match *self {
            HodlMessageType::JoinPool => "join_pool".to_string(),
        }
    }
}

#[derive(Debug, EnumIter)]
pub enum HodlMessageType {
    JoinPool,
}

#[derive(Debug, EnumIter)]
pub enum IndetermineMessageType {
    SwapExactAmountIn,
}

// TODO: impl Display to apply for MessageType
impl Display for IndetermineMessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl MessageType for IndetermineMessageType {
    fn get_type(&self) -> String {
        match *self {
            IndetermineMessageType::SwapExactAmountIn => "SwapExactAmountIn".to_string(),
        }
    }
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

#[derive(Debug, EnumIter)]
pub enum EventType {
    TokenSwapped,
    PoolExited,
    PoolJoined,
}

impl Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            EventType::TokenSwapped => "token_swapped",
            EventType::PoolExited => "pool_exited",
            EventType::PoolJoined => "pool_joined",
        };
        write!(f, "{}", s)
    }
}
