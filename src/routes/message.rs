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

enum DumpMessageType {
    BeginUnlocking,
    BeginUnlockingAll,
    LockTokens,
    SuperfluidUnbondLock,
    SuperfluidUndelegate,
}

enum HodlMessageType {
    JoinPool,
    LockTokens,
    LockAndSuperfluidDelegate,
    SuperfluidDelegate,
}

struct DumpMessage {
    message_type: DumpMessageType,
    transaction_hash: String,
}

struct HodlMessage {
    message_type: HodlMessageType,
    transaction_hash: String,
}