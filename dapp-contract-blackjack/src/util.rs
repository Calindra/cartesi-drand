pub mod random {
    use std::ops::Range;

    use rand::prelude::*;
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;
    use uuid::Uuid;

    pub struct Random {
        seed: String,
    }

    impl Random {
        pub fn new(seed: String) -> Self {
            Random { seed }
        }

        pub fn generate_random_seed(&self, range: Range<usize>) -> usize {
            let mut rng: Pcg64 = Seeder::from(self.seed.clone()).make_rng();
            rng.gen_range(range)
        }
    }

    pub fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
}

pub mod json {
    use serde_json::{json, Value};

    pub fn decode_payload(payload: &str) -> Option<Value> {
        let payload = payload.trim_start_matches("0x");

        let payload = hex::decode(payload).ok()?;
        let payload = String::from_utf8(payload).ok()?;

        let payload = serde_json::from_str::<Value>(payload.as_str()).ok()?;

        Some(payload)
    }

    pub fn generate_message(payload: Value) -> Value {
        let payload = hex::encode(payload.to_string());
        let payload = format!("0x{}", payload);

        json!({
            "data": {
                "payload": payload,
            }
        })
    }
}
