//! 采用 rust 实现的 zabbix 应用开发包，实现了 proxy, agent, sender 基本功能。
//! 同时提供了 zabbix 通信协议 api, 方便用户定制开发。
//!
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate failure;

use failure::Error;

type Result<T> = std::result::Result<T, Error>;

mod protocol;
pub use self::protocol::ZabbixProtocol;

mod request;
pub use self::request::{ZabbixDiscovery, ZabbixHost, ZabbixMetric, ZabbixRequest};

mod response;
pub use self::response::Response;

mod proxy;
pub use self::proxy::{ZabbixProxy, Host, HostItem, Item, ItemHost};

mod agent;
pub use self::agent::ZabbixAgent;

mod sender;
pub use self::sender::ZabbixSender;
