#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Call {
    pub unique_id: String,
    pub action: String,
    pub payload: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CallResult {
    pub unique_id: String,
    pub payload: serde_json::Value,
}

impl CallResult {
    pub fn new<T: serde::Serialize>(unique_id: String, payload: T) -> Self {
        Self {
            unique_id,
            payload: serde_json::json!(payload),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CallError<T> {
    pub unique_id: String,
    pub error_code: T,
    pub error_description: String,
    pub error_details: serde_json::Value,
}

impl<T> CallError<T> {
    pub fn new(unique_id: String, error_code: T) -> Self {
        Self {
            unique_id,
            error_code,
            error_description: String::new(),
            error_details: serde_json::json!({}),
        }
    }
}
