use super::frame::{Call, CallError, CallResult};

#[derive(Debug, Clone)]
pub struct Invalid {
    pub unique_id: Option<String>,
    pub message: String,
    pub err_msg: String,
}

#[derive(Debug, Clone)]
pub enum CallResponse<T> {
    CallResult(CallResult),
    CallError(CallError<T>),
}

pub enum OcppMessage<T> {
    Call(Call),
    CallResponse(CallResponse<T>),
    Invalid(Invalid),
}

pub trait EncodeDecode {
    fn encode(&self) -> String;
}

impl<T> CallResponse<T> {
    pub fn get_unique_id(&self) -> String {
        match self {
            CallResponse::CallResult(t) => t.unique_id.clone(),
            CallResponse::CallError(t) => t.unique_id.clone(),
        }
    }
}

impl<T: serde::de::DeserializeOwned> OcppMessage<T> {
    pub fn decode(message: String) -> OcppMessage<T> {
        let raw: serde_json::Value = match serde_json::from_str(&message) {
            Ok(val) => val,
            Err(e) => {
                return OcppMessage::Invalid(Invalid {
                    unique_id: None,
                    message,
                    err_msg: format!("JSON parse error: {}", e),
                });
            }
        };

        let arr = match raw {
            serde_json::Value::Array(arr) => arr,
            _ => {
                return OcppMessage::Invalid(Invalid {
                    unique_id: None,
                    message,
                    err_msg: "Expected JSON array".into(),
                });
            }
        };

        match arr.first().and_then(|v| v.as_u64()) {
            Some(2) if arr.len() == 4 => {
                let unique_id = arr[1].as_str().map(|s| s.to_string());
                let action = arr[2].as_str().map(|s| s.to_string());
                let payload = arr[3].clone();

                if let (Some(unique_id), Some(action)) = (unique_id.clone(), action) {
                    OcppMessage::Call(Call {
                        unique_id,
                        action,
                        payload,
                    })
                } else {
                    OcppMessage::Invalid(Invalid {
                        unique_id,
                        message,
                        err_msg: "Invalid Call structure".into(),
                    })
                }
            }

            Some(3) if arr.len() == 3 => {
                let unique_id = arr[1].as_str().map(|s| s.to_string());
                let payload = arr[2].clone();
                
                if let Some(unique_id) = unique_id {
                    OcppMessage::CallResponse(CallResponse::CallResult(CallResult {
                        unique_id,
                        payload,
                    }))
                } else {
                    OcppMessage::Invalid(Invalid {
                        unique_id: None,
                        message,
                        err_msg: "Invalid CallResult structure".into(),
                    })
                }
            }

            Some(4) if arr.len() == 5 => {
                let unique_id = arr[1].as_str().map(|s| s.to_string());
                let error_code = serde_json::from_value::<T>(arr[2].clone());
                let error_description = arr[3].as_str().map(|s| s.to_string());
                let error_details = arr[4].clone();

                if let (Some(unique_id), Ok(error_code), Some(error_description)) = (unique_id.clone(), error_code, error_description) {
                    OcppMessage::CallResponse(CallResponse::CallError(CallError {
                        unique_id,
                        error_code,
                        error_description,
                        error_details,
                    }))
                } else {
                    OcppMessage::Invalid(Invalid {
                        unique_id,
                        message,
                        err_msg: "Invalid CallError structure".into(),
                    })
                }
            }

            _ => OcppMessage::Invalid(Invalid {
                unique_id: None,
                message,
                err_msg: "Unknown or malformed message".into(),
            }),
        }
    }
}

impl EncodeDecode for Call {
    fn encode(&self) -> String {
        serde_json::to_string(&(2, &self.unique_id, &self.action, &self.payload)).unwrap()
    }
}

impl EncodeDecode for CallResult {
    fn encode(&self) -> String {
        serde_json::to_string(&(3, &self.unique_id, &self.payload)).unwrap()
    }
}
impl<T: ToString> EncodeDecode for CallError<T> {
    fn encode(&self) -> String {
        serde_json::to_string(&(
            4,
            &self.unique_id,
            self.error_code.to_string(),
            &self.error_description,
            &self.error_details,
        ))
        .unwrap()
    }
}

impl<T: ToString> CallResponse<T> {
    pub fn encode(self) -> String {
        match self {
            CallResponse::CallResult(t) => t.encode(),
            CallResponse::CallError(t) => t.encode(),
        }
    }
}
