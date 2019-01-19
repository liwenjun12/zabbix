use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    response: String,
    info: Option<String>,
}

impl Response {
    pub fn success(&self) -> bool {
        self.response == "success"
    }

    pub fn ok(&self) -> bool {
        (self.failed_cnt() == 0) && (self.processed_cnt() == self.total_cnt())
    }

    pub fn processed_cnt(&self) -> i32 {
        if let Some(result) = self.get_value_from_info("processed") {
            if let Ok(int_value) = result.parse::<i32>() {
                return int_value;
            }
        }
        -1
    }

    pub fn failed_cnt(&self) -> i32 {
        if let Some(result) = self.get_value_from_info("failed") {
            if let Ok(int_value) = result.parse::<i32>() {
                return int_value;
            }
        }
        -1
    }

    pub fn total_cnt(&self) -> i32 {
        if let Some(result) = self.get_value_from_info("total") {
            if let Ok(int_value) = result.parse::<i32>() {
                return int_value;
            }
        }
        -1
    }

    pub fn seconds_spent(&self) -> f32 {
        if let Some(result) = self.get_value_from_info("seconds_spent") {
            if let Ok(float_value) = result.parse::<f32>() {
                return float_value;
            }
        }
        -1.0
    }

    fn get_value_from_info(&self, name: &str) -> Option<String> {
        //{ response: "success", info: Some("processed: 6; failed: 0; total: 6; seconds spent: 0.000172") }
        let reg = regex::Regex::new(r"processed: (?P<processed>\d+); failed: (?P<failed>\d+); total: (?P<total>\d+); seconds spent: (?P<seconds_spent>\d.\d+)").unwrap();

        if let Some(v) = &self.info {
            if let Some(x) = reg.captures(v) {
                return Some(x[name].to_string());
            }
        }

        None
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Host {
    pub hostid: i64,
    pub host: String,
    pub name: String,
}

impl Host {
    pub fn new(hostid: i64, host: String, name: String) -> Self {
        Self { hostid, host, name }
    }

    pub fn from(data: Vec<HashMap<String, Value>>) -> HashSet<Self> {
        let mut result = HashSet::new();
        for d in data {
            //if 0 == d["status"].as_i64().unwrap() {
            let hostid = d["hostid"].as_i64().unwrap();
            let host = d["host"].as_str().unwrap();
            let name = d["name"].as_str().unwrap();
            result.insert(Self::new(hostid, host.to_string(), name.to_string()));
            //}
        }
        result
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct Item {
    pub hostid: i64,
    pub key_: String,
    delay: u32,
}

impl Item {
    pub fn new(hostid: i64, key_: String, delay: u32) -> Self {
        Self {
            hostid,
            key_,
            delay,
        }
    }

    pub fn from(data: Vec<HashMap<String, Value>>, compress: &[&str]) -> HashSet<Self> {
        let mut result = HashSet::new();
        for d in data {
            //if 0 == d["status"].as_i64().expect("status") {
            let delay = trans(d["delay"].as_str().expect("delay"));
            if delay == 0 {
                continue;
            }
            let hostid = d["hostid"].as_i64().unwrap();
            let mut key_ = d["key_"].as_str().expect("key_");

            for s in compress {
                key_ = key_.split(s).next().unwrap();
            }

            result.insert(Self::new(hostid, key_.to_string(), delay));
            //}
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
            let mut vikings = HashMap::new();
            vikings.insert("hostid".to_string(), json!(3010 + x % 2));
            vikings.insert("delay".to_string(), json!("30s"));
            vikings.insert("key_".to_string(), json!(format!("df[{}]", x)));

            data.push(vikings);
        }

        //let compress = ["[", "_"];
        let compress = ["["];

        let items = Item::from(data.clone(), &compress);
        assert_eq!(2, items.len());

        let items = Item::from(data.clone(), &[]);
        assert_eq!(5, items.len());
    }

    #[test]
    fn test_response() {
        //{ response: "success", info: Some("processed: 0; failed: 14; total: 14; seconds spent: 0.000172") }
        let resp = Response {
            response: "success".to_string(),
            info: Some("processed: 0; failed: 14; total: 14; seconds spent: 0.000172".to_string()),
        };

        assert!(resp.success());
        assert!(!resp.ok());
        assert_eq!(14, resp.total_cnt());
        assert_eq!(14, resp.failed_cnt());
        assert_eq!(0, resp.processed_cnt());

        let resp1 = Response {
            response: "success".to_string(),
            info: Some("processed: 14; failed: 0; total: 14; seconds spent: 0.000172".to_string()),
        };
        assert!(resp1.ok());

        let resp2 = Response {
            response: "success".to_string(),
            info: Some("processed: 10; failed: 4; total: 14; seconds spent: 0.000172".to_string()),
        };
        assert!(!resp2.ok());
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
