use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

/// Attempt to convert either a number or string into a u64
/// Some manifests in-the-wild have strings where integers are expected
pub fn deserialize_into_u64<'de, D>(data: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let json_value: Value = Deserialize::deserialize(data)?;

    match json_value {
        Value::Number(number_value) => {
            let mut result: u64 = 0;
            if let Some(result_u64) = number_value.as_u64() {
                result = result_u64
            } else if let Some(result_f64) = number_value.as_f64() {
                result = result_f64 as u64
            }
            Ok(result)
        }
        Value::String(string_value) => {
            Ok(string_value.parse().map_err(|err| {
                de::Error::custom(format!(
                    "Failed parsing string to u64: {}",
                    err
                ))
            })?)
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
