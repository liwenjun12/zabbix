//! 基于 rust 实现的 zabbix proxy，实现了基本的代理功能。
//!
use super::Result;
use super::protocol::ZabbixProtocol;
use super::request::{ZabbixHost, ZabbixMetric, ZabbixRequest};
use super::response::Response;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ProxyResponse {
    RESPONSE(Response),
    CONFIG(Value),
}

/// zabbix proxy
/// 实现了 proxy 的基本功能
///
#[derive(Debug, Clone)]
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
        let data = serde_json::to_value(data)?;
        let req = ZabbixRequest::new(Self::HISTORY_DATA, &self.name, data);
        if let Ok(r) = self.send_request(&req, false) {
            if let ProxyResponse::RESPONSE(c) = r {
                //trace!("{:?}", c);
                return Ok(c.success() && c.ok());
            }
        }
        Ok(false)
    }
}

/// 扩展代理功能
impl ZabbixProxy {
    pub fn get_proxy_config(&self, compress: &[&str]) -> Option<(HashSet<Host>, HashSet<Item>)> {
        if let Some(v) = self.get_config() {
            let h = Host::from(get_item(&v["hosts"]["fields"], &v["hosts"]["data"]));
            let i = Item::from(
                get_item(&v["items"]["fields"], &v["items"]["data"]),
                compress,
            );

            return Some((h, i));
        }
        None
    }

    pub fn get_proxy_config_item(&self, compress: &[&str]) -> Option<Vec<ItemHost>> {
        if let Some((hosts, items)) = self.get_proxy_config(compress) {
            let ih = items
                .into_iter()
                .filter(|p| {
                    hosts
                        .iter()
                        .map(|q| q.hostid)
                        .collect::<Vec<_>>()
                        .contains(&p.hostid)
                })
                .map(|p| ItemHost {
                    host: hosts
                        .iter()
                        .filter(|q| q.hostid == p.hostid)
                        .collect::<Vec<_>>()[0]
                        .clone(),
                    item: p,
                })
                .collect();

            return Some(ih);
        }
        None
    }

    pub fn get_proxy_config_host(&self, compress: &[&str]) -> Option<Vec<HostItem>> {
        if let Some((hosts, items)) = self.get_proxy_config(compress) {
            let it = |x| {
                items
                    .clone()
                    .into_iter()
                    .filter(|p| p.hostid == x)
                    .collect::<Vec<_>>()
            };
            let hi = hosts
                .into_iter()
                .map(|p| HostItem {
                    items: it(p.hostid),
                    host: p,
                })
                .collect();

            return Some(hi);
        }
        None
    }
}

fn get_item(field: &Value, data: &Value) -> Vec<HashMap<String, Value>> {
    let mut result = Vec::new();
    if let Some(field) = field.as_array() {
        let field = field.iter().map(|x| x.as_str());
        if let Some(data) = data.as_array() {
            for x in data.iter() {
                let mut hm: HashMap<String, Value> = HashMap::new();
                let y = field.clone().zip(x.as_array().unwrap().iter());
                for z in y {
                    hm.insert(z.0.unwrap().to_string(), z.1.clone());
                }
                result.push(hm);
            }
        }
    }
    result
}

/*
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
*/

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct HostItem {
    pub host: Host,
    pub items: Vec<Item>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ItemHost {
    pub item: Item,
    pub host: Host,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Host {
    pub hostid: i64,
    pub host: String,
    //pub name: String,
}

impl Host {
    pub fn new(hostid: i64, host: String) -> Self {
        Self { hostid, host }
    }

    pub fn from(data: Vec<HashMap<String, Value>>) -> HashSet<Self> {
        let mut result = HashSet::new();
        for d in data {
            if let Some(0) = d["status"].as_i64() {
                let hostid = d["hostid"].as_i64().unwrap();
                let host = d["host"].as_str().unwrap();
                result.insert(Self::new(hostid, host.to_string()));
            }
        }
        result
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Item {
    pub itemid: i64,
    pub hostid: i64,
    pub key_: String,
    pub delay: u32,
}

impl Item {
    pub fn new(itemid: i64, hostid: i64, key_: String, delay: u32) -> Self {
        Self {
            itemid,
            hostid,
            key_,
            delay,
        }
    }

    pub fn from(data: Vec<HashMap<String, Value>>, compress: &[&str]) -> HashSet<Self> {
        let mut result = HashSet::new();
        for d in data {
            if let Some(0) = d["status"].as_i64() {
                let delay = trans(d["delay"].as_str().expect("delay"));
                if delay == 0 {
                    continue;
                }
                let itemid = d["itemid"].as_i64().unwrap();
                let hostid = d["hostid"].as_i64().unwrap();
                let mut key_ = d["key_"].as_str().expect("key_");

                for s in compress {
                    key_ = key_.split(s).next().unwrap();
                }

                result.insert(Self::new(itemid, hostid, key_.to_string(), delay));
            }
        }
        result
    }
}

fn trans(input: &str) -> u32 {
    if let Ok(result) = input.parse() {
        return result;
    }

    if let Ok(x) = input.to_lowercase().parse::<humantime::Duration>() {
        return x.as_secs() as u32;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_from() {
        let mut data: Vec<HashMap<String, Value>> = vec![];

        for x in 0..5 {
            let mut item = HashMap::new();
            item.insert("status".to_string(), json!(0));
            item.insert("itemid".to_string(), json!(1));
            item.insert("hostid".to_string(), json!(3010 + x % 2));
            item.insert("delay".to_string(), json!("30s"));
            item.insert("key_".to_string(), json!(format!("df[{}]", x)));
            data.push(item);
        }

        //let compress = ["[", "_"];
        let compress = ["["];

        let items = Item::from(data.clone(), &compress);
        assert_eq!(2, items.len());

        let items = Item::from(data.clone(), &[]);
        assert_eq!(5, items.len());
    }

    #[test]
    fn test_trans() {
        assert_eq!(trans("15"), 15);
        assert_eq!(trans("15s"), 15);
        assert_eq!(trans("15S"), 15);
        assert_eq!(trans("5m"), 300);
        assert_eq!(trans("2h"), 7200);
        assert_eq!(trans("1d"), 86400);
    }
}
