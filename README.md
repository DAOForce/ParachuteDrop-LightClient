# ParachuteDrop-LightClient
## What's this?
* This module is the helper for Parachute Drop. It tracks the activity of users in Osmosis dedicated to Evmos token, especially looking at the dumping/hodling tokens by looking at the following messages.
* Messages that we look into in the light client
  * BeginUnlocking
  * BeginUnlockingAll
  * ExitPool
  * SuperfluidUnboundLock
  * SuperfluidUndelegate
  * JoinPool
  * LockTokens
  * LockAndSuperfluidDelegate
  * SuperfluidDelegate

## How to start?
```
cargo fmt
cargo clippy --all --all-targets --all-features
cargo test --all --all-targets --all-features
cargo build
cargo run
```

## References
* [cosmos-rust-package](https://github.com/Philipp-Sc/cosmos-rust-package)
* [ibc-proto-rs](https://github.com/cosmos/ibc-proto-rs)
* [hermes](https://github.com/informalsystems/hermes)
* [osmosis docs](https://docs.osmosis.zone/osmosis-core/modules/lockup/)
* [target pool](https://app.osmosis.zone/pool/722)
* [querying block events](https://docs.tendermint.com/v0.34/app-dev/indexing-transactions.html#querying_block_events)
* [events](https://docs.cosmos.network/v0.46/core/events.html)
* [evmos swagger](https://api.evmos.dev/#/Service/GetTxsEvent)
* [cosmos sdk](https://github.com/osmosis-labs/cosmos-sdk/blob/osmosis-main/proto/cosmos/tx/v1beta1/service.proto)
* [grpcurl](https://docs.osmosis.zone/apis/grpc/interact-grpc-curl/)
* [Rust actix web](https://choiseokwon.tistory.com/332)
* [actix docs](https://actix.rs/docs/extractors)
