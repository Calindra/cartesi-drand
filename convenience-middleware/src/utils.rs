pub mod util {
    use serde_json::Value;
    pub(crate) fn deserialize_obj(request: &str) -> Option<serde_json::Map<String, Value>> {
        let json = serde_json::from_str::<serde_json::Value>(request);

        match json {
            Ok(Value::Object(map)) => Some(map),
            _ => None,
        }
    }
}
