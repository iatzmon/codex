//! Schema version validation utilities for hooks.

use serde_json::Value;

use super::{HookEventPayload, HookOutcome};

/// Registry of supported hook payload schema versions.
#[derive(Debug, Default)]
pub struct HookSchemaRegistry;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HookPayloadValidationError {
    #[error("invalid payload: {0}")]
    Invalid(String),
}

impl HookSchemaRegistry {
    pub fn validate_payload(json: &str) -> Result<(), HookPayloadValidationError> {
        let value: Value = serde_json::from_str(json)
            .map_err(|err| HookPayloadValidationError::Invalid(err.to_string()))?;
        let obj = value.as_object().ok_or_else(|| {
            HookPayloadValidationError::Invalid("payload must be an object".into())
        })?;

        match obj.get("schemaVersion") {
            Some(Value::String(version)) if version == "1.0" => {}
            _ => {
                return Err(HookPayloadValidationError::Invalid(
                    "unsupported schemaVersion".into(),
                ));
            }
        }

        match obj.get("event") {
            Some(Value::String(_)) => {}
            _ => {
                return Err(HookPayloadValidationError::Invalid(
                    "event must be provided".into(),
                ));
            }
        }

        // Deserialize to ensure the payload matches the strongly typed model.
        serde_json::from_value::<HookEventPayload>(value)
            .map_err(|err| HookPayloadValidationError::Invalid(err.to_string()))?;

        Ok(())
    }

    pub fn supported_decisions() -> Vec<HookOutcome> {
        vec![
            HookOutcome::Allow,
            HookOutcome::Ask,
            HookOutcome::Deny,
            HookOutcome::Block,
            HookOutcome::Continue,
        ]
    }
}
