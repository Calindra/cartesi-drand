use std::borrow::BorrowMut;

use actix_web::web::Data;
use drand_verify::{G2Pubkey, Pubkey};
use serde_json::json;

use crate::{
    models::models::{AppState, DrandBeacon, PayloadWithBeacon},
    rollup::{self, RollupInput},
};

pub(crate) fn is_querying_pending_beacon(rollup_input: &RollupInput) -> bool {
    rollup_input.decoded_inspect() == "pendingdrandbeacon"
}

pub(crate) async fn send_pending_beacon_report(app_state: &Data<AppState>) {
    let manager = app_state.input_buffer_manager.lock().await;
    let x = manager.pending_beacon_timestamp.get();
    let report = json!({ "payload": format!("{x:#x}") });
    let _ = rollup::server::send_report(report).await.unwrap();
}

pub(crate) fn get_drand_beacon(payload: &str) -> Option<DrandBeacon> {
    let key = std::env::var("DRAND_PUBLIC_KEY").unwrap();
    let payload = || {
        let payload = payload.trim_start_matches("0x");
        let payload = hex::decode(payload).ok()?;
        std::str::from_utf8(&payload).ok().map(|s| s.to_owned())
    };

    let payload = payload()?;

    let payload = serde_json::from_str::<PayloadWithBeacon>(&payload).ok()?;

    let key = key.as_str();
    let mut pk = [0u8; 96];
    hex::decode_to_slice(key, pk.borrow_mut()).ok()?;

    let pk = G2Pubkey::from_fixed(pk).ok()?;

    let signature = hex::decode(&payload.beacon.signature).ok()?;

    let round = payload.beacon.round;

    pk.verify(round, b"", &signature)
        .ok()
        .map(|_| payload.beacon)
}
