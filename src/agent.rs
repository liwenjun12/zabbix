//! 基于 rust 实现的 zabbix proxy，实现了基本的代理功能。
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
