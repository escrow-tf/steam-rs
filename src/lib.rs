#![feature(assert_matches)]

pub mod steamid;
pub mod steamlang;
pub mod tf2econ;
pub mod transport;

pub mod steamproto {
    include!(concat!(env!("OUT_DIR"), "/steamproto.rs"));
}
