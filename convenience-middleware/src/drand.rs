use std::{borrow::BorrowMut, error::Error};

use actix_web::web::Data;
use dotenvy::var;
use drand_verify::{derive_randomness, G2PubkeyRfc, Pubkey};
use log::{error, warn};
use serde_json::json;

use crate::{
    models::structs::{AppState, DrandBeacon, PayloadWithBeacon},
    rollup::{input::RollupInput, server::send_report},
};

pub fn is_querying_pending_beacon(rollup_input: &RollupInput) -> Result<bool, Box<dyn Error>> {
    let result = rollup_input.decoded_inspect()?;
    Ok(result == "pendingdrandbeacon")
}

pub async fn send_pending_beacon_report(app_state: &Data<AppState>) {
    let manager = app_state.input_buffer_manager.lock().await;
    let x = manager.pending_beacon_timestamp.get();
    let report = json!({ "payload": format!("{x:#x}") });
    let _ = send_report(report).await.unwrap();
}

/**
 * Check if the request is a drand beacon
 * Example of a drand beacon request
 *
 * {"beacon":{"round":3828300,"randomness":"7ff726d290836da706126ada89f7e99295c672d6768ec8e035fd3de5f3f35cd9","signature":"ab85c071a4addb83589d0ecf5e2389f7054e4c34e0cbca65c11abc30761f29a0d338d0d307e6ebcb03d86f781bc202ee"}}
 */
pub fn get_drand_beacon(payload: &str) -> Result<DrandBeacon, Box<dyn std::error::Error>> {
    let key = var("DRAND_PUBLIC_KEY").expect("Public Key not found");

    let payload = payload.trim_start_matches("0x");
    let payload = hex::decode(payload)?;
    let payload = std::str::from_utf8(&payload).map(|s| s.to_owned())?;

    let payload = serde_json::from_str::<PayloadWithBeacon>(&payload)?;

    let mut pk = [0u8; 96];
    hex::decode_to_slice(&key, pk.borrow_mut())?;

    let pk = G2PubkeyRfc::from_fixed(pk).map_err(|e| e.to_string())?;

    let signature = hex::decode(&payload.beacon.signature)?;

    let round = payload.beacon.round;

    match pk.verify(round, b"", &signature) {
        Ok(valid) => {
            if !valid {
                let msg = format!(
                    "Invalid beacon signature for round {}; signature: {}; public_key: {};",
                    round, &payload.beacon.signature, key
                );

                warn!("{msg}");
                return Err(msg.into());
            }
            let mut beacon = payload.beacon.to_owned();

            // make sure that the signature is the source of randomness
            beacon.randomness = hex::encode(derive_randomness(&signature));
            Ok(beacon)
        }
        Err(e) => {
            error!("Drand VerificationError: {}", e.to_string());
            Err(Box::new(e))
        }
    }
}
