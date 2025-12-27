#![feature(assert_matches)]
#![feature(duration_constructors)]

pub mod community;
pub mod econ;
pub mod mobile;
pub mod steamid;
pub mod steamlang;
pub mod tf2econ;
pub mod totp;
pub mod tradeoffer;
pub mod transport;
pub mod twofactor;

pub mod steamproto {
    include!(concat!(env!("OUT_DIR"), "/steamproto.rs"));
}
