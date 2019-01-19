//! 基于 rust 实现的 zabbix proxy，实现了基本的代理功能。
//!
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

use failure::Error;

type Result<T> = std::result::Result<T, Error>;

mod request;
pub use self::request::{ZabbixDiscovery, ZabbixHost, ZabbixMetric, ZabbixRequest};

mod response;
pub use self::response::{Host, Item, ProxyResponse, Response};

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use zabbix::ZabbixProtocol;

#[derive(Debug, Clone, Copy)]
pub struct ZabbixProxy {
    name: String,
    proto: ZabbixProtocol,
}

impl ZabbixProxy {
    pub const PROXY_CONFIG: &'static str = "proxy config";
    pub const HISTORY_DATA: &'static str = "history data";
    pub const PROXY_HEARTBEAT: &'static str = "proxy heartbeat";
    pub const AUTO_REGISTRATION: &'static str = "auto registration";

    pub fn new(name: &str, server: &str, port: u16) -> Self {
        let name = String::from(name);
        let proto = ZabbixProtocol::new(server, port);
        Self { name, proto }
    }

    fn send_request(&self, req: &ZabbixRequest, is_config: bool) -> Result<ProxyResponse> {
        let read_data = self.proto.send(&req.str())?;
        let response = if is_config {
            ProxyResponse::CONFIG(serde_json::from_slice(&read_data)?)
        } else {
            ProxyResponse::RESPONSE(serde_json::from_slice(&read_data)?)
        };

        Ok(response)
    }

    ///
    /// 从ZABBNIX服务端获取代理配置信息
    ///
    pub fn get_config(&self) -> Option<Value> {
        let req = ZabbixRequest::new(Self::PROXY_CONFIG, &self.name, Value::Null);
        if let Ok(r) = self.send_request(&req, true) {
            if let ProxyResponse::CONFIG(c) = r {
                return Some(c);
            }
        }
        None
    }

    ///
    /// 自动注册主机
    ///
    pub fn auto_register(&self, hosts: Vec<ZabbixHost>) -> Result<bool> {
        let hosts = serde_json::to_value(hosts)?;
        let req = ZabbixRequest::new(Self::AUTO_REGISTRATION, &self.name, hosts);
        if let Ok(r) = self.send_request(&req, false) {
            if let ProxyResponse::RESPONSE(c) = r {
                return Ok(c.success());
            }
        }
        Ok(false)
    }

    ///
    /// 向服务端发送心跳信息
    ///
    pub fn heart_beat(&self) -> Result<bool> {
        let req = ZabbixRequest::new(Self::PROXY_HEARTBEAT, &self.name, Value::Null);
        if let Ok(r) = self.send_request(&req, false) {
            if let ProxyResponse::RESPONSE(c) = r {
                return Ok(c.success());
            }
        }
        Ok(false)
    }

    ///
    /// 向服务端发送历史数据
    ///
    pub fn send_data(&self, data: &[ZabbixMetric]) -> Result<bool> {
        trace!("request = {:?}", Self::HISTORY_DATA);
        if !data.is_empty() {
            trace!("key[0] = {:?}", &data[0].key);
            trace!("host[0] = {:?}", &data[0].host);
            trace!("data[0] = {:?}", &data[0].value);
        } else {
            trace!("NODATA");
        }
        let data = serde_json::to_value(data)?;
        let req = ZabbixRequest::new(Self::HISTORY_DATA, &self.name, data);
        if let Ok(r) = self.send_request(&req, false) {
            if let ProxyResponse::RESPONSE(c) = r {
                trace!("{:?}", c);
                return Ok(c.success() && c.ok());
            }
        }
        Ok(false)
    }

    ///
    /// 返回服务器配置的代理主机和监控项
    ///
    /// compress = &[] 不压缩 item
    /// compress = &["["] 表示将 item 合并，去除 []
    /// compress = &["[", "_"] 表示将 item 合并，去除 [] 和 _
    ///
    pub fn get_proxy_config(&self, compress: &[&str]) -> Option<(HashSet<Host>, HashSet<Item>)> {
        if let Some(v) = self.get_config() {
            return Some((
                Host::from(get_item(&v["hosts"]["fields"], &v["hosts"]["data"])),
                Item::from(
                    get_item(&v["items"]["fields"], &v["items"]["data"]),
                    compress,
                ),
            ));
        }
        None
    }
}

fn get_item(field: &Value, data: &Value) -> Vec<HashMap<String, Value>> {
    let mut result = Vec::new();

    if let Some(field) = field.as_array() {
        let field = field.iter().map(|x| x.as_str());
        let data = data.as_array().unwrap();
        for x in data.iter() {
            let mut hm: HashMap<String, Value> = HashMap::new();
            let y = field.clone().zip(x.as_array().unwrap().iter());
            for z in y {
                hm.insert(z.0.unwrap().to_string(), z.1.clone());
            }
            result.push(hm);
        }
    }
    result
}
