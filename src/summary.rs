//! Pure summary logic: parse user-supplied bytes as JSON and inventory the
//! top-level shape. No vendor schema knowledge; all fixtures are synthetic.

/// Maximum number of keys / child entries listed before truncating output.
pub const MAX_LISTED: usize = 64;

/// Read-only inventory of one JSON document.
pub struct Summary {
    pub size_bytes: u64,
    pub shape: &'static str,
    pub keys: Vec<String>,
    pub total_keys: usize,
    pub children: Vec<(String, usize)>,
    pub truncated: bool,
}

/// Classify a JSON value without inspecting vendor-specific fields.
pub fn shape_of(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Object(_) => "object",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Null => "null",
    }
}

/// Parse `bytes` as JSON and summarize it. Returns the serde error
/// untouched so the caller can report it as `not JSON: <reason>`.
pub fn summarize(bytes: &[u8]) -> Result<Summary, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    let shape = shape_of(&value);
    let mut summary = Summary {
        size_bytes: bytes.len() as u64,
        shape,
        keys: Vec::new(),
        total_keys: 0,
        children: Vec::new(),
        truncated: false,
    };
    if let serde_json::Value::Object(map) = &value {
        let mut keys: Vec<&String> = map.keys().collect();
        keys.sort();
        summary.total_keys = keys.len();
        summary.truncated = keys.len() > MAX_LISTED;
        for key in keys.into_iter().take(MAX_LISTED) {
            summary.keys.push(key.clone());
            match &map[key] {
                serde_json::Value::Object(child) => {
                    summary.children.push((key.clone(), child.len()));
                }
                serde_json::Value::Array(items) => {
                    summary.children.push((key.clone(), items.len()));
                }
                _ => {}
            }
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_keys_are_sorted() {
        let s = summarize(br#"{"b": 1, "a": 2}"#).unwrap();
        assert_eq!(s.shape, "object");
        assert_eq!(s.keys, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(s.total_keys, 2);
        assert!(s.children.is_empty());
        assert!(!s.truncated);
        assert_eq!(s.size_bytes, 16);
    }

    #[test]
    fn nested_child_counts_skip_scalars() {
        let s = summarize(br#"{"a": {"x": 1, "y": 2}, "b": [1, 2, 3], "c": 5}"#).unwrap();
        assert_eq!(s.children, vec![("a".to_string(), 2), ("b".to_string(), 3)]);
    }

    #[test]
    fn array_shape_has_no_keys() {
        let s = summarize(b"[1, 2, 3]").unwrap();
        assert_eq!(s.shape, "array");
        assert!(s.keys.is_empty());
        assert!(!s.truncated);
    }

    #[test]
    fn scalar_shapes_classified() {
        assert_eq!(summarize(b"\"hi\"").unwrap().shape, "string");
        assert_eq!(summarize(b"42").unwrap().shape, "number");
        assert_eq!(summarize(b"true").unwrap().shape, "bool");
        assert_eq!(summarize(b"null").unwrap().shape, "null");
    }

    #[test]
    fn key_list_truncates_at_cap() {
        let mut doc = String::from("{");
        for i in 0..70 {
            if i > 0 {
                doc.push(',');
            }
            doc.push_str(&format!("\"k{i:02}\": {i}"));
        }
        doc.push('}');
        let s = summarize(doc.as_bytes()).unwrap();
        assert_eq!(s.total_keys, 70);
        assert_eq!(s.keys.len(), MAX_LISTED);
        assert!(s.truncated);
        assert_eq!(s.keys[0], "k00");
    }

    #[test]
    fn invalid_json_errors() {
        assert!(summarize(b"not json").is_err());
    }
}
