#![feature(assert_matches)]

pub mod community;
pub mod econ;
pub mod steamid;
pub mod steamlang;
pub mod tf2econ;
pub mod tradeoffer;
pub mod transport;
pub mod twofactor;

pub mod steamproto {
    include!(concat!(env!("OUT_DIR"), "/steamproto.rs"));
}
