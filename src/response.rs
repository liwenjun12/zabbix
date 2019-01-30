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

#[cfg(test)]
mod tests {
    use super::*;

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
}
