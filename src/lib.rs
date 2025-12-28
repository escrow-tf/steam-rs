//! Rust library for steam auth, inventories, trade offers, and mobile conf.
//!
//! To send a request, you must create the appropriate transport. [`tf2econ::PlayerItemsRequest`] implements [`From`]
//! for [`transport::PublicRequest`], so we need a [`transport::PublicTransport`]:
//!
//! ```
//! let api_key = env::var("STEAM_WEB_API_KEY").expect("STEAM_WEB_API_KEY must be set with an api key");
//! let public_transport = PublicTransport::new(api_key)?;
//!
//! // build a request to get gaben's tf2 inventory
//! let steam_id = "76561198010565263".parse::<SteamID>()?;
//! let request = PlayerItemsRequest::builder()
//!     .steam_id(steam_id)
//!     .build()
//!
//! let response = public_transport.send(request.into())?;
//! println!("{:?}", response.items);
//! ```
//!
//! Some requests require a [`transport::PrivateTransport`], like those found in [`tradeoffer`].

#![feature(assert_matches)]
#![feature(duration_constructors)]
#![warn(clippy::pedantic)]

/// Authenticate with the Steam network.
pub mod auth;

/// Query community profiles, like player inventories.
pub mod community;

/// Query steam's `IEconService` for trade offers.
pub mod econ;

/// Query, accept, or decline mobile confirmations.
pub mod mobile;

/// Utilities or parsing valid 64-bit Steam IDs.
pub mod steamid;

/// Various helpers and constants for evaluating Steam responses.
pub mod steamlang;

/// Query a player's TF2 (app 440) inventory.
pub mod tf2econ;

/// Mobile Auth TOTP and Confirmation TOTP generators.
pub mod totp;

/// Manage trade offers.
pub mod tradeoffer;

/// Send requests created by the other modules in this crate.
pub mod transport;

/// Query Steam for information about twofactor authentication.
pub mod twofactor;

/// Rendered Steam network protobufs.
pub mod steamproto {
    #![allow(clippy::pedantic)]
    include!(concat!(env!("OUT_DIR"), "/steamproto.rs"));
}
