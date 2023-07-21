pub mod util {
    use std::borrow::BorrowMut;

    use drand_verify::{G2Pubkey, Pubkey};
    use serde_json::Value;

    pub(crate) fn deserialize_obj(request: &str) -> Option<serde_json::Map<String, Value>> {
        let json = serde_json::from_str::<serde_json::Value>(request);

        match json {
            Ok(Value::Object(map)) => Some(map),
            _ => None,
        }
    }

    pub(crate) fn is_drand_beacon(request: &str) -> bool {
        let key = std::env::var("PK_UNCHAINED_TESTNET").unwrap();
    
        let json = match deserialize_obj(request) {
            Some(json) => json,
            None => return false,
        };
    
        if !json.contains_key("data") {
            return false;
        }
    
        let json = match json["data"].as_object() {
            Some(data) => data,
            None => return false,
        };
    
        if !json.contains_key("payload") {
            return false;
        }
    
        let payload = || {
            let payload = json["payload"].as_str()?;
            let payload = payload.trim_start_matches("0x");
            let payload = hex::decode(payload).ok()?;
            let payload = match std::str::from_utf8(&payload) {
                Ok(payload) => payload.to_owned(),
                Err(_) => return None,
            };
            Some(payload)
        };
    
        let payload = match payload() {
            Some(payload) => payload,
            None => return false,
        };
    
        let json = match deserialize_obj(payload.as_str()) {
            Some(json) => json,
            None => return false,
        };
    
        let json = match json.get("beacon") {
            Some(beacon) => beacon,
            None => return false,
        };
    
        let json = match json.as_object() {
            Some(beacon) => beacon,
            None => return false,
        };
    
        if !json.contains_key("signature") || !json.contains_key("round") || !json["round"].is_number()
        {
            return false;
        }
    
        let key = key.as_str();
        let mut pk = [0u8; 96];
        let is_decoded_err = hex::decode_to_slice(key, pk.borrow_mut()).is_err();
    
        if is_decoded_err {
            return false;
        }
    
        let pk = match G2Pubkey::from_fixed(pk) {
            Ok(pk) => pk,
            Err(_) => return false,
        };
    
        let signature = || {
            let signature = json["signature"].as_str()?;
            let signature = hex::decode(signature).ok()?;
            Some(signature)
        };
    
        let signature = match signature() {
            Some(signature) => signature,
            None => return false,
        };
    
        let round = json["round"].as_u64();
    
        let round = match round {
            Some(round) => round,
            None => return false,
        };
    
        let is_valid_key = pk.verify(round, b"", &signature).ok().is_some();
    
        is_valid_key
    }
}
