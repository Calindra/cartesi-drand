use std::borrow::BorrowMut;

use actix_web::web::Data;
use drand_verify::{derive_randomness, G2Pubkey, G2PubkeyRfc, Pubkey};
use log::{warn, error};
use serde_json::json;

use crate::{
    models::models::{AppState, DrandBeacon, PayloadWithBeacon},
    rollup::{self, RollupInput},
};

pub fn is_querying_pending_beacon(rollup_input: &RollupInput) -> bool {
    rollup_input.decoded_inspect() == "pendingdrandbeacon"
}

pub async fn send_pending_beacon_report(app_state: &Data<AppState>) {
    let manager = app_state.input_buffer_manager.lock().await;
    let x = manager.pending_beacon_timestamp.get();
    let report = json!({ "payload": format!("{x:#x}") });
    let _ = rollup::server::send_report(report).await.unwrap();
}

/**
 * Check if the request is a drand beacon
 * Example of a drand beacon request
 *
 * {"beacon":{"round":3828300,"randomness":"7ff726d290836da706126ada89f7e99295c672d6768ec8e035fd3de5f3f35cd9","signature":"ab85c071a4addb83589d0ecf5e2389f7054e4c34e0cbca65c11abc30761f29a0d338d0d307e6ebcb03d86f781bc202ee"}}
 */
pub fn get_drand_beacon(payload: &str) -> Option<DrandBeacon> {
    let key = std::env::var("DRAND_PUBLIC_KEY").unwrap();
    let payload = || {
        let payload = payload.trim_start_matches("0x");
        let payload = hex::decode(payload).ok()?;
        std::str::from_utf8(&payload).ok().map(|s| s.to_owned())
    };

    let payload = payload()?;

    let payload = serde_json::from_str::<PayloadWithBeacon>(&payload).ok()?;

    let key_s: &str = key.as_str();
    let mut pk = [0u8; 96];
    hex::decode_to_slice(key_s, pk.borrow_mut()).ok()?;

    let pk = G2PubkeyRfc::from_fixed(pk).ok()?;

    let signature = hex::decode(&payload.beacon.signature).ok()?;

    let round = payload.beacon.round;

    match pk.verify(round, b"", &signature) {
        Ok(valid) => {
            if !valid {
                warn!("Invalid beacon signature for round {}; signature: {}; public_key: {};", round, &payload.beacon.signature, key);
                return None
            }
            let mut beacon = payload.beacon.clone();

            // make sure that the signature is the source of randomness
            beacon.randomness = hex::encode(derive_randomness(&signature));
            Some(beacon)
        }
        Err(e) => {
            error!("Drand VerificationError: {}", e.to_string());
            None
        },
    }
}
