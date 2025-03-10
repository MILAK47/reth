//! reth P2P networking.
//!
//! Ethereum's networking protocol is specified in [devp2p](https://github.com/ethereum/devp2p).
//!
//! In order for a node to join the ethereum p2p network it needs to know what nodes are already
//! part of that network. This includes public identities (public key) and addresses (where to reach
//! them).
//!
//! ## Bird's Eye View
//!
//! See also diagram in [`NetworkManager`]
//!
//! The `Network` is made up of several, separate tasks:
//!
//!    - `Transactions Task`: is a spawned
//!      [`TransactionsManager`](crate::transactions::TransactionsManager) future that:
//!
//!        * Responds to incoming transaction related requests
//!        * Requests missing transactions from the `Network`
//!        * Broadcasts new transactions received from the
//!          [`TransactionPool`](reth_transaction_pool::TransactionPool) over the `Network`
//!
//!    - `ETH request Task`: is a spawned
//!      [`EthRequestHandler`](crate::eth_requests::EthRequestHandler) future that:
//!
//!        * Responds to incoming ETH related requests: `Headers`, `Bodies`
//!
//!    - `Discovery Task`: is a spawned [`Discv4`](reth_discv4::Discv4) future that handles peer
//!      discovery and emits new peers to the `Network`
//!
//!    - [`NetworkManager`] task advances the state of the `Network`, which includes:
//!
//!        * Initiating new _outgoing_ connections to discovered peers
//!        * Handling _incoming_ TCP connections from peers
//!        * Peer management
//!        * Route requests:
//!             - from remote peers to corresponding tasks
//!             - from local to remote peers
//!
//! ## Usage
//!
//! ### Configure and launch a standalone network
//!
//! The [`NetworkConfig`] is used to configure the network.
//! It requires an instance of [`BlockReader`](reth_provider::BlockReader).
//!
//! ```
//! # async fn launch() {
//! use reth_network::{config::rng_secret_key, NetworkConfig, NetworkManager};
//! use reth_primitives::mainnet_nodes;
//! use reth_provider::test_utils::NoopProvider;
//!
//! // This block provider implementation is used for testing purposes.
//! let client = NoopProvider::default();
//!
//! // The key that's used for encrypting sessions and to identify our node.
//! let local_key = rng_secret_key();
//!
//! let config = NetworkConfig::builder(local_key).boot_nodes(mainnet_nodes()).build(client);
//!
//! // create the network instance
//! let network = NetworkManager::new(config).await.unwrap();
//!
//! // keep a handle to the network and spawn it
//! let handle = network.handle().clone();
//! tokio::task::spawn(network);
//!
//! # }
//! ```
//!
//! ### Configure all components of the Network with the [`NetworkBuilder`]
//!
//! ```
//! use reth_network::{config::rng_secret_key, NetworkConfig, NetworkManager};
//! use reth_primitives::mainnet_nodes;
//! use reth_provider::test_utils::NoopProvider;
//! use reth_transaction_pool::TransactionPool;
//! async fn launch<Pool: TransactionPool>(pool: Pool) {
//!     // This block provider implementation is used for testing purposes.
//!     let client = NoopProvider::default();
//!
//!     // The key that's used for encrypting sessions and to identify our node.
//!     let local_key = rng_secret_key();
//!
//!     let config =
//!         NetworkConfig::builder(local_key).boot_nodes(mainnet_nodes()).build(client.clone());
//!
//!     // create the network instance
//!     let (handle, network, transactions, request_handler) = NetworkManager::builder(config)
//!         .await
//!         .unwrap()
//!         .transactions(pool)
//!         .request_handler(client)
//!         .split_with_handle();
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `serde` (default): Enable serde support for configuration types.
//! - `test-utils`: Various utilities helpful for writing tests
//! - `geth-tests`: Runs tests that require Geth to be installed locally.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![warn(missing_debug_implementations, missing_docs, rustdoc::all)] // TODO(danipopes): unreachable_pub
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(any(test, feature = "test-utils"))]
/// Common helpers for network testing.
pub mod test_utils;

mod builder;
mod cache;
pub mod config;
mod discovery;
pub mod error;
pub mod eth_requests;
mod fetch;
mod flattened_response;
mod import;
mod listener;
mod manager;
mod message;
mod metrics;
mod network;
pub mod peers;
mod session;
mod state;
mod swarm;
pub mod transactions;

pub use builder::NetworkBuilder;
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use discovery::{Discovery, DiscoveryEvent};
pub use fetch::FetchClient;
pub use manager::{NetworkEvent, NetworkManager};
pub use message::PeerRequest;
pub use network::{NetworkEvents, NetworkHandle};
pub use peers::PeersConfig;
pub use session::{
    ActiveSessionHandle, ActiveSessionMessage, Direction, PeerInfo, PendingSessionEvent,
    PendingSessionHandle, PendingSessionHandshakeError, SessionCommand, SessionEvent, SessionId,
    SessionLimits, SessionManager, SessionsConfig,
};

pub use reth_eth_wire::{DisconnectReason, HelloBuilder, HelloMessage};
