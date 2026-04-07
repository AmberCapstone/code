use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

pub const STATUS: &str = "tcp://127.0.0.1:5550";
pub const COMMAND: &str = "tcp://127.0.0.1:5551";
pub const BASESTATION: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5552));
pub const INFLUX: &str = "http://127.0.0.1:8086";
