pub mod models {
    use std::{borrow::BorrowMut, cell::Cell, collections::VecDeque, sync::Arc};

    use log::info;
    use serde::{Deserialize, Serialize};
    use sha3::{Digest, Sha3_256};
    use tokio::sync::Mutex;

    use crate::rollup::RollupInput;

    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(non_snake_case)]
    pub struct DrandEnv {
        pub DRAND_PUBLIC_KEY: String,
        pub DRAND_PERIOD: Option<u64>,
        pub DRAND_GENESIS_TIME: Option<u64>,
        pub DRAND_SAFE_SECONDS: Option<u64>,
    }

    #[derive(Serialize)]
    pub(crate) struct Item {
        pub(crate) request: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct RequestRollups {
        status: String,
    }

    pub(crate) struct Flag {
        pub(crate) is_holding: bool,
    }

    #[derive(Deserialize)]
    pub(crate) struct Timestamp {
        pub(crate) timestamp: u64,
    }

    pub(crate) struct Beacon {
        pub(crate) timestamp: u64,
        pub(crate) round: u64,
        pub(crate) randomness: String,
    }

    impl Beacon {
        pub(crate) fn some_from(drand_beacon: &DrandBeacon, timestamp: u64) -> Option<Beacon> {
            Some(Beacon {
                timestamp,
                round: drand_beacon.round,
                randomness: drand_beacon.randomness.to_string(),
            })
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct PayloadWithBeacon {
        pub(crate) beacon: DrandBeacon,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct DrandBeacon {
        pub(crate) round: u64,
        pub(crate) signature: String,
        pub(crate) randomness: String,
    }

    pub(crate) struct InputBufferManager {
        pub(crate) messages: VecDeque<Item>,
        pub(crate) flag_to_hold: Flag,
        pub(crate) request_count: Cell<usize>,
        pub(crate) last_beacon: Cell<Option<Beacon>>,
        pub(crate) pending_beacon_timestamp: Cell<u64>,
        pub(crate) randomness_salt: Cell<u64>,
        pub(crate) is_inspecting: bool,
    }

    pub(crate) struct AppState {
        pub(crate) input_buffer_manager: Arc<Mutex<InputBufferManager>>,
        pub(crate) drand_period: u64,
        pub(crate) drand_genesis_time: u64,
        pub(crate) safe_seconds: u64,
        pub(crate) version: String,
    }

    impl AppState {
        pub(crate) fn new() -> AppState {
            let manager = InputBufferManager::default();
            let drand_period = std::env::var("DRAND_PERIOD")
                .expect("Missing env DRAND_PERIOD")
                .parse::<u64>()
                .unwrap();
            let drand_genesis_time = std::env::var("DRAND_GENESIS_TIME")
                .expect("Missing env DRAND_GENESIS_TIME")
                .parse::<u64>()
                .unwrap();
            let safe_seconds = std::env::var("DRAND_SAFE_SECONDS")
                .expect("Missing env DRAND_SAFE_SECONDS")
                .parse::<u64>()
                .unwrap();
            let version: Option<&str> = option_env!("CARGO_PKG_VERSION");
            AppState {
                input_buffer_manager: Arc::new(Mutex::new(manager)),
                drand_period,
                drand_genesis_time,
                safe_seconds,
                version: version.unwrap_or("unknown").to_string(),
            }
        }
        pub(crate) fn get_randomness_for_timestamp(&self, query_timestamp: u64) -> Option<String> {
            let mut manager = match self.input_buffer_manager.try_lock() {
                Ok(manager) => manager,
                Err(_) => return None,
            };
            let safe_query_timestamp = query_timestamp + self.safe_seconds;
            match manager.last_beacon.take() {
                Some(beacon) => {
                    info!(
                        "beacon time {} vs {} request time",
                        beacon.timestamp, query_timestamp
                    );
                    // Check the beacon timestamp against the safe query timestamp
                    if safe_query_timestamp < beacon.timestamp {
                        let salt = manager.randomness_salt.take() + 1;
                        manager.randomness_salt.set(salt);

                        let mut hasher = Sha3_256::new();
                        hasher.update([beacon.randomness.as_bytes(), &salt.to_le_bytes()].concat());
                        let randomness = hasher.finalize();
                        manager.flag_to_hold.release();
                        manager.last_beacon.set(Some(beacon));
                        Some(hex::encode(randomness))
                    } else {
                        manager.set_pending_beacon_timestamp(safe_query_timestamp);
                        manager.last_beacon.set(Some(beacon));
                        None
                    }
                }
                None => {
                    manager.set_pending_beacon_timestamp(safe_query_timestamp);
                    None
                }
            }
        }
        pub(crate) fn keep_newest_beacon(&self, drand_beacon: DrandBeacon) {
            let beacon_time = (drand_beacon.round * self.drand_period) + self.drand_genesis_time;
            info!(
                "Calculated beacon time {} for round {}",
                beacon_time, drand_beacon.round
            );
            let manager = self.input_buffer_manager.try_lock().unwrap();
            if let Some(current_beacon) = manager.last_beacon.take() {
                if current_beacon.round < drand_beacon.round {
                    info!("Set new beacon");
                    manager
                        .last_beacon
                        .set(Beacon::some_from(&drand_beacon, beacon_time));
                } else {
                    info!("Keep current beacon");
                    manager.last_beacon.set(Some(current_beacon));
                }
            } else {
                info!("No beacon, initializing");
                manager
                    .last_beacon
                    .set(Beacon::some_from(&drand_beacon, beacon_time));
            }
        }
        pub(crate) async fn store_input(&self, rollup_input: &RollupInput) {
            let mut manager = self.input_buffer_manager.lock().await;
            let request = serde_json::to_string(rollup_input).unwrap();
            manager.messages.push_back(Item { request });
        }
        pub(crate) async fn consume_input(&self) -> Option<Item> {
            let mut manager = self.input_buffer_manager.lock().await;
            return manager.consume_input();
        }
        pub(crate) async fn set_inspecting(&self, value: bool) {
            let mut manager = self.input_buffer_manager.lock().await;
            manager.is_inspecting = value;
        }
        pub(crate) fn is_inspecting(&self) -> bool {
            #[cfg(target_arch = "riscv64")]
            {
                let manager = match self.input_buffer_manager.try_lock() {
                    Ok(manager) => manager,
                    Err(_) => return false,
                };
                return manager.is_inspecting;
            }
            false
        }
    }

    impl Flag {
        fn new() -> Flag {
            Flag { is_holding: false }
        }

        pub(crate) fn hold_up(&mut self) {
            self.is_holding = true;
        }

        pub(crate) fn release(&mut self) {
            self.is_holding = false;
        }
    }

    impl Default for InputBufferManager {
        fn default() -> Self {
            InputBufferManager {
                messages: VecDeque::new(),
                flag_to_hold: Flag::new(),
                request_count: Cell::new(0),
                last_beacon: Cell::new(None),
                pending_beacon_timestamp: Cell::new(0),
                randomness_salt: Cell::new(0),
                is_inspecting: false,
            }
        }
    }

    impl InputBufferManager {
        pub(crate) fn set_pending_beacon_timestamp(&mut self, timestamp: u64) {
            let current = self.pending_beacon_timestamp.take();
            // mantendo o mais recente para economizar transacoes
            if current == 0 || current < timestamp {
                info!("pending beacon timestamp {} changed", timestamp);
                self.pending_beacon_timestamp.set(timestamp);
            } else {
                info!("pending beacon timestamp {} still the same", current);
                self.pending_beacon_timestamp.set(current);
            }
        }

        pub(crate) fn consume_input(&mut self) -> Option<Item> {
            info!("Consuming input");
            let buffer = self.messages.borrow_mut();

            if buffer.is_empty() || self.flag_to_hold.is_holding {
                return None;
            }

            let data = buffer.pop_front();
            self.request_count.set(self.request_count.get() - 1);
            data
        }

        pub(crate) fn await_beacon(&mut self) {
            info!("Awaiting beacon");

            self.flag_to_hold.hold_up();
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use super::models::{AppState, Beacon, DrandBeacon, InputBufferManager};

    fn create_drand_beacon(round: u64) -> DrandBeacon {
        DrandBeacon {
            round,
            signature: String::from("signature"),
            randomness: String::from("randomness"),
        }
    }

    fn create_app_state() -> AppState {
        let version: Option<&str> = option_env!("CARGO_PKG_VERSION");
        AppState {
            input_buffer_manager: Arc::new(Mutex::new(InputBufferManager::default())),
            drand_period: 3,
            drand_genesis_time: 1677685200,
            safe_seconds: 5,
            version: version.unwrap_or("unknown").to_string(),
        }
    }

    #[actix_web::test]
    async fn test_app_state_init_beacon() {
        let app = create_app_state();
        let beacon = create_drand_beacon(2);
        app.keep_newest_beacon(beacon);
        let manager = app.input_buffer_manager.lock().await;
        assert_eq!(2, manager.last_beacon.take().unwrap().round);
    }

    #[actix_web::test]
    async fn test_app_state_keep_current_beacon() {
        let app = create_app_state();
        {
            let manager = app.input_buffer_manager.lock().await;
            manager.last_beacon.set(Some(Beacon {
                timestamp: 1677685206,
                round: 2,
                randomness: "".to_string(),
            }))
        }
        {
            let beacon = create_drand_beacon(1);
            app.keep_newest_beacon(beacon);
            let manager = app.input_buffer_manager.lock().await;
            assert_eq!(2, manager.last_beacon.take().unwrap().round);
        }
    }

    #[actix_web::test]
    async fn test_app_state_keep_new_beacon() {
        let app = create_app_state();
        {
            let manager = app.input_buffer_manager.lock().await;
            manager.last_beacon.set(Some(Beacon {
                timestamp: 1677685206,
                round: 2,
                randomness: "".to_string(),
            }))
        }
        {
            let beacon = create_drand_beacon(3);
            app.keep_newest_beacon(beacon);
            let manager = app.input_buffer_manager.lock().await;
            assert_eq!(3, manager.last_beacon.take().unwrap().round);
        }
    }
}
