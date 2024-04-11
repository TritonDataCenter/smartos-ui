use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

/// Attempt to convert either a number or string into an u64 this is used where
/// json in-the-wild has strings or floats where integers are expected.
///
/// The property that's being parsed can be one of a few JSON types, we expect
/// either a Number (which is either a signed or unsigned integer or a float)
/// or a String that can be parsed into either an u64 or f64.
/// In the case of a string that can't be parsed, 0 is returned
/// For other type, an error is returned.
pub fn deserialize_into_u64<'de, D>(data: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let json_value: Value = Deserialize::deserialize(data)?;
    let mut result: u64 = 0;

    match json_value {
        // Some valid JSON number (1, -43, 0.5, -1.9, etc)
        Value::Number(number_value) => {
            let mut result: u64 = 0;
            // Try to get the number as a u64
            if let Some(result_u64) = number_value.as_u64() {
                result = result_u64
            // Try to get the number as a f64
            } else if let Some(result_f64) = number_value.as_f64() {
                // Convert the f64 into a u64
                result = result_f64 as u64
            }
            Ok(result)
        }
        Value::String(string_value) => {
            // Try to parse the string as a u64
            if let Ok(result_u64) = string_value.parse::<u64>() {
                result = result_u64
            // Try to parse the string as a u64
            } else if let Ok(result_f64) = string_value.parse::<f64>() {
                // Convert the f64 into a u64
                result = result_f64 as u64
            }
            Ok(result)
        }
        _ => Err(de::Error::unknown_variant(
            json_value.to_string().as_str(),
            &["String", "u64"],
        )),
    }
}

/// Attempt to convert either a boolean or string into a boolean
/// Some manifests in-the-wild have strings where bools are expected
pub fn deserialize_into_bool<'de, D>(data: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let json_value: Value = Deserialize::deserialize(data)?;

    match json_value {
        Value::Bool(bool_value) => Ok(bool_value),
        Value::String(string_value) => {
            let val = string_value.to_lowercase();
            match val.as_str() {
                "true" | "yes" => Ok(true),
                "false" | "no" | "" => Ok(false),
                _ => Err(de::Error::unknown_variant(
                    val.as_str(),
                    &["true", "false"],
                )),
            }
        }
        _ => Err(de::Error::custom("Expected either boolean or String")),
    }
}
