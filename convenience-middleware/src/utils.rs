pub mod util {
    use std::borrow::BorrowMut;

    use drand_verify::{G2Pubkey, Pubkey};
    use serde_json::Value;

    use crate::models::models::{PayloadWithBeacon, DrandBeacon};

    pub(crate) fn deserialize_obj(request: &str) -> Option<serde_json::Map<String, Value>> {
        let json = serde_json::from_str::<serde_json::Value>(request);

        match json {
            Ok(Value::Object(map)) => Some(map),
            _ => None,
        }
    }

    pub(crate) fn get_drand_beacon(payload: &str) -> Option<DrandBeacon> {
        let key = std::env::var("PK_UNCHAINED_TESTNET").unwrap();
        let payload = || {
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
            None => return None,
        };
    
        let result_deserialization = serde_json::from_str::<PayloadWithBeacon>(payload.as_str());
        let payload = match result_deserialization {
            Ok(payload) => payload,
            Err(_) => return None,
        };
        
        let key = key.as_str();
        let mut pk = [0u8; 96];
        let is_decoded_err = hex::decode_to_slice(key, pk.borrow_mut()).is_err();
    
        if is_decoded_err {
            return None;
        }
    
        let pk = match G2Pubkey::from_fixed(pk) {
            Ok(pk) => pk,
            Err(_) => return None,
        };
    
        let signature = || {
            let signature = payload.beacon.signature.as_str();
            let signature = hex::decode(signature).ok()?;
            Some(signature)
        };
    
        let signature = match signature() {
            Some(signature) => signature,
            None => return None,
        };
    
        let round = payload.beacon.round;
    
        let is_valid_key = pk.verify(round, b"", &signature).ok().is_some();
    
        if is_valid_key {
            Some(payload.beacon)
        } else {
            None
        }
    }
}
