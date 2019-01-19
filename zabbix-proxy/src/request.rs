use chrono::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZabbixRequest {
    request: &'static str,
    host: String,
    clock: i64,
    ns: i64,
    data: Value,
}

impl ZabbixRequest {
    pub fn new(request: &'static str, host: &str, data: Value) -> Self {
        let host = String::from(host);
        //let clock = Utc::now().timestamp();
        let clock = Local::now().timestamp();
        let ns = 0;
        Self {
            request,
            host,
            clock,
            ns,
            data,
        }
    }

    pub fn str(&self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]//, PartialEq)]
pub struct ZabbixMetric {
    pub host: String,
    pub key: String,
    pub value: String,
    clock: i64,
    //ns: i64,
}

impl ZabbixMetric {
    pub fn new(host: &str, key: &str, value: &str) -> Self {
        let host = String::from(host);
        let key = String::from(key);
        let value = String::from(value);
        let clock = Local::now().timestamp();
        Self {
            host,
            key,
            value,
            clock,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZabbixDiscovery {
    data: Vec<HashMap<String, String>>,
}

impl ZabbixDiscovery {
    pub fn new(param: &str, value: Vec<String>) -> Self {
        let mut data = Vec::new();

        for v in value {
            let k = String::from(param);
            //let k = String::from("{#APPNO}");
            let mut d = HashMap::new();
            d.insert(k, v);
            data.push(d);
        }

        Self { data }
    }

    pub fn str(&self) -> String {
        match serde_json::to_string(&self) {
            Ok(c) => c,
            Err(_) => "[]".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZabbixHost {
    host: String,
    host_metadata: &'static str,
    ip: &'static str,
    port: u16,
    clock: i64,
}

impl ZabbixHost {
    pub fn new(host: String) -> Self {
        // let host = String::from(host);
        let host_metadata = "DBMP";
        let ip = "127.0.0.1";
        let port = 10050;
        let clock = Local::now().timestamp();
        Self {
            host,
            host_metadata,
            ip,
            port,
            clock,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zabbix_request() {
        let req = ZabbixRequest::new("REQUEST", "HOST", Value::Null);
        //"{\"request\":\"REQUEST\",\"host\":\"HOST\",\"clock\":1547466205,\"ns\":0,\"data\":null}"

        assert!(req.str().contains("\"request\":\"REQUEST\""));
        assert!(req.str().contains("\"host\":\"HOST\""));
        assert_eq!("REQUEST", req.request);
        assert_eq!("HOST", req.host);

        //hosts: Vec<ZabbixHost>
        let hosts = [
            ZabbixHost::new("host1".to_string()),
            ZabbixHost::new("host2".to_string()),
        ];
        let hosts =
            serde_json::to_value(hosts).unwrap_or_else(|_| Value::String("NOHOST".to_string()));
        let req1 = ZabbixRequest::new("REQUEST", "HOST", hosts);
        //"{\"request\":\"REQUEST\",\"host\":\"HOST\",\"clock\":1547467120,\"ns\":0,\"data\":[{\"clock\":1547467120,\"host\":\"host1\",\"host_metadata\":\"DBMP\",\"ip\":\"127.0.0.1\",\"port\":10050},{\"clock\":1547467120,\"host\":\"host2\",\"host_metadata\":\"DBMP\",\"ip\":\"127.0.0.1\",\"port\":10050}]}"

        assert!(req1.data.is_array());
        assert!(req1.str().contains("\"host\":\"host1\""));
    }

    #[test]
    fn test_zabbix_discovery() {
        let data = vec!["A".to_string(), "B".to_string()];
        let req = ZabbixDiscovery::new("{#APPNO}", data);
        //"{\"data\":[{\"{#APPNO}\":\"A\"},{\"{#APPNO}\":\"B\"}]}"

        assert!(req.str().contains("\"{#APPNO}\":\"A\""));
        assert!(req.str().contains("\"{#APPNO}\":\"B\""));
        assert_eq!(2, req.data.len());
    }
}
