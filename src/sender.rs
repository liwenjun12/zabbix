//! zabbix sender
//!
use super::protocol::ZabbixProtocol;

///
#[derive(Debug, Clone)]
pub struct ZabbixSender {
    name: String,
    proto: ZabbixProtocol,
}

impl ZabbixSender {
    pub fn new(name: &str, server: &str, port: u16) -> Self {
        let name = String::from(name);
        let proto = ZabbixProtocol::new(server, port);
        Self { name, proto }
    }
}
