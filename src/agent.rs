//! zabbix agent
//!
use super::protocol::ZabbixProtocol;

///
#[derive(Debug, Clone)]
pub struct ZabbixAgent {
    name: String,
    proto: ZabbixProtocol,
}

impl ZabbixAgent {
    pub fn new(name: &str, server: &str, port: u16) -> Self {
        let name = String::from(name);
        let proto = ZabbixProtocol::new(server, port);
        Self { name, proto }
    }
}
