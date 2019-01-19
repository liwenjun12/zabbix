//! 采用 rust 实现的 zabbix 协议库

#[macro_use]
extern crate failure;
// #[macro_use] extern crate log;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::net::TcpStream;

use std::io;
use std::io::prelude::*;

use failure::Error;

type Result<T> = std::result::Result<T, Error>;

/// 定义了 zabbix server 的地址和端口
#[derive(Debug, Clone)]
pub struct ZabbixProtocol {
    server: String,
    port: u16,
}

impl ZabbixProtocol {
    pub const ZBX_HDR: &'static [u8; 5] = b"ZBXD\x01";
    pub const ZBX_HDR_SIZE: usize = 13;

    pub fn new(server: &str, port: u16) -> Self {
        let server = String::from(server);
        Self { server, port }
    }

    fn create_packet(&self, data: &str) -> (Vec<u8>, usize) {
        let data = data.as_bytes();

        let mut buf = [0; 8];
        let length = data.len();
        LittleEndian::write_u64(&mut buf, length as u64);

        let mut packet: Vec<u8> = Vec::with_capacity(Self::ZBX_HDR_SIZE + length);
        packet.extend(Self::ZBX_HDR);
        packet.extend(&buf);
        packet.extend(data);
        (packet, Self::ZBX_HDR_SIZE + length)
    }

    pub fn send(&self, data: &str) -> Result<Vec<u8>> {
        let addr = format!("{0}:{1}", self.server, self.port);
        // trace!("send addr = {:?}", addr);

        let mut s = TcpStream::connect(addr)?;
        let (pkt, _) = self.create_packet(data);
        let _ = s.write(&pkt)?;

        let mut zbx_hdr = [0; Self::ZBX_HDR_SIZE];
        let _ = s.read(&mut zbx_hdr)?;
        if Self::ZBX_HDR != &zbx_hdr[..5] {
            return Err(format_err!("packet header invalid"));
        }

        // let mut rdr = io::Cursor::new(zbx_hdr);
        // rdr.set_position(5);
        let mut rdr = io::Cursor::new(&zbx_hdr[5..]);
        let data_length = rdr.read_u64::<LittleEndian>()?;
        if data_length == 0 {
            return Err(format_err!("packet data length = 0"));
        }

        let mut read_data = vec![];
        s.take(data_length).read_to_end(&mut read_data)?;
        Ok(read_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zabbix_protocol_create_packet() {
        let data = "data";
        let zbx = ZabbixProtocol::new("127.0.0.1", 10051);
        let (pkt, size) = zbx.create_packet(data);
        // [90, 66, 88, 68, 1, 4, 0, 0, 0, 0, 0, 0, 0, 100, 97, 116, 97]

        let mut rdr = io::Cursor::new(&pkt[5..ZabbixProtocol::ZBX_HDR_SIZE]);
        let length = rdr.read_u64::<LittleEndian>().unwrap();

        assert_eq!(
            ZabbixProtocol::ZBX_HDR,
            &pkt[..5],
            "Header {:?}",
            ZabbixProtocol::ZBX_HDR
        );
        assert_eq!(data.len() + ZabbixProtocol::ZBX_HDR_SIZE, size);
        assert_eq!(length, data.len() as u64);
        assert_eq!(data.as_bytes(), &pkt[ZabbixProtocol::ZBX_HDR_SIZE..]);
    }
}
