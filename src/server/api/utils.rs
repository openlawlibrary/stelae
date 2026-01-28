//! Utils file for all stelae endpoints

//! Centralized state management for the Actix web server
use std::collections::HashMap;

/// Converts json blob data into `HashMap`<String, String>
///
/// # Errors
/// Will error if unable to parse blob to `HashMap`
pub fn convert_vec_u8_to_hashmap(
    blob: &[u8],
) -> Result<HashMap<String, String>, serde_json::Error> {
    let pairs: Vec<[String; 2]> = serde_json::from_slice(blob)?;
    let mut map: HashMap<String, String> = HashMap::new();

    for [from, to] in pairs {
        map.insert(from, to);
    }
    Ok(map)
}
