use serde_json::Value;
use jsonschema::{Draft, JSONSchema};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorAlert {
    #[serde(rename = "deviceId")]
    device_id: String,
    #[serde(rename = "readingTime")]
    reading_time: DateTime<Utc>,
    #[serde(rename = "errorCode")]
    error_code: String,
    #[serde(rename = "errorMessage")]
    error_message: String,
}

impl ErrorAlert {
    pub fn new(device_id: String, error_code: String, error_message: String) -> Self {
        Self {
            device_id,
            reading_time: Utc::now(),
            error_code,
            error_message,
        }
    }

    pub fn generate_alert(device_id: String, error_code: String, error_message: String) -> String {
        let error_alert = ErrorAlert::new(device_id, error_code, error_message);
        serde_json::to_string(&error_alert).unwrap()
    }
}

pub fn parse_json_schema(schema_str: &str) -> Result<Value, String> {
    // Parse the schema string
    let schema: Value = serde_json::from_str(schema_str).map_err(|e| e.to_string())?;
    Ok(schema)
}

pub fn validate_json(schema: Value, instance: Value, device_id: &str,) -> Result<String, String> {
    // Compile the schema and validate the instance
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .map_err(|e| e.to_string())?;
    let result = compiled.validate(&instance);
    if let Err(errors) = result {
        let mut error_messages = Vec::new();
        for error in errors {
            error_messages.push(format!("JSON schema validation error: {}. Instance path: {}.", error, error.instance_path));
        }
        let alert = ErrorAlert::generate_alert(
            device_id.to_string(), 
            "004".to_string(), 
            error_messages.join("; ")
        );
        Err(alert)
    } else {
        Ok("Validation successful!".to_string())
    }
}
