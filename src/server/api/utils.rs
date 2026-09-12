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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_returns_expected_map() {
        let json = br#"[["/old-path", "/new-path"], ["/outdated", "/current"]]"#;
        let map = convert_vec_u8_to_hashmap(json).expect("should parse valid json");

        let mut expected: HashMap<String, String> = HashMap::new();
        expected.insert("/old-path".to_owned(), "/new-path".to_owned());
        expected.insert("/outdated".to_owned(), "/current".to_owned());

        assert_eq!(map, expected);
    }

    #[test]
    fn test_empty_array_returns_empty_map() {
        let json = b"[]";
        let map = convert_vec_u8_to_hashmap(json).expect("should parse empty array");
        assert!(map.is_empty());
    }

    #[test]
    fn test_single_pair() {
        let json = br#"[["/a", "/b"]]"#;
        let map = convert_vec_u8_to_hashmap(json).expect("should parse single pair");

        let mut expected: HashMap<String, String> = HashMap::new();
        expected.insert("/a".to_owned(), "/b".to_owned());

        assert_eq!(map, expected);
    }

    #[test]
    fn test_duplicate_keys_last_value_wins() {
        // HashMap insertion order follows array order, so later duplicates
        // overwrite earlier ones.
        let json = br#"[["/old-path", "/first"], ["/old-path", "/second"]]"#;
        let map = convert_vec_u8_to_hashmap(json).expect("should parse json with duplicate keys");

        assert_eq!(map.len(), 1);
        assert_eq!(map.get("/old-path"), Some(&"/second".to_owned()));
    }

    #[test]
    fn test_invalid_json_returns_err() {
        let json = b"not valid json";
        let result = convert_vec_u8_to_hashmap(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_pair_wrong_arity_returns_err() {
        // Pairs must be exactly two-element arrays.
        let json = br#"[["/old-path", "/new-path", "/extra"]]"#;
        let result = convert_vec_u8_to_hashmap(json);
        assert!(result.is_err());
    }
}
