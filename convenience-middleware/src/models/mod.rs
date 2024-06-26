pub mod structs {
    use std::{borrow::BorrowMut, cell::Cell, collections::VecDeque, error::Error, sync::Arc};

    use dotenvy::var;
    use log::info;
    use serde::{Deserialize, Serialize};
    #[cfg(test)]
    use serde_json::json;
    use sha3::{Digest, Sha3_256};
    use tokio::sync::Mutex;

    use crate::rollup::input::RollupInput;

    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(non_snake_case)]
    pub struct DrandEnv {
        pub DRAND_PUBLIC_KEY: String,
        pub DRAND_PERIOD: Option<u64>,
        pub DRAND_GENESIS_TIME: Option<u64>,
        pub DRAND_SAFE_SECONDS: Option<u64>,
    }

    #[derive(Serialize)]
    pub struct Item {
        pub request: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct RequestRollups {
        status: String,
    }

    pub struct Flag {
        is_holding: bool,
    }

    #[derive(Deserialize)]
    pub struct Timestamp {
        pub timestamp: u64,
    }

    #[derive(Default)]
    pub struct Beacon {
        pub timestamp: u64,
        pub round: u64,
        pub randomness: String,
    }

    #[derive(Default)]
    pub struct BeaconBuilder(Beacon);

    impl Beacon {
        pub fn builder() -> BeaconBuilder {
            BeaconBuilder::default()
        }
    }

    impl BeaconBuilder {
        pub fn with_timestamp(mut self, timestamp: u64) -> BeaconBuilder {
            self.0.timestamp = timestamp;
            self
        }

        #[cfg(test)]
        pub fn with_round(mut self, round: u64) -> BeaconBuilder {
            self.0.round = round;
            self
        }

        #[cfg(test)]
        pub fn with_randomness(mut self, randomness: String) -> BeaconBuilder {
            self.0.randomness = randomness;
            self
        }

        pub fn with_drand_beacon(mut self, drand_beacon: &DrandBeacon) -> BeaconBuilder {
            self.0.round = drand_beacon.round;
            self.0.randomness = drand_beacon.randomness.to_string();
            self
        }

        pub fn build(self) -> Beacon {
            self.0
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PayloadWithBeacon {
        pub beacon: DrandBeacon,
    }

    #[derive(Default, Serialize, Deserialize, Debug, Clone)]
    pub struct DrandBeacon {
        pub round: u64,
        pub signature: String,
        pub randomness: String,
    }

    #[cfg(test)]
    #[derive(Default)]
    pub struct DrandBeaconBuilder(DrandBeacon);

    #[cfg(test)]
    impl DrandBeacon {
        pub fn builder() -> DrandBeaconBuilder {
            DrandBeaconBuilder::default()
        }

        pub fn wrap(&self) -> serde_json::Value {
            let payload = serde_json::to_value(self).unwrap();
            json!(
            {
                "beacon": payload,
                "input":"0x00",
            })
        }
    }

    #[cfg(test)]
    impl DrandBeaconBuilder {
        pub fn with_round(mut self, round: u64) -> DrandBeaconBuilder {
            self.0.round = round;
            self
        }

        pub fn with_signature(mut self, signature: String) -> DrandBeaconBuilder {
            self.0.signature = signature;
            self
        }

        pub fn with_randomness(mut self, randomness: String) -> DrandBeaconBuilder {
            self.0.randomness = randomness;
            self
        }

        pub fn build(self) -> DrandBeacon {
            self.0
        }
    }

    pub struct InputBufferManager {
        pub messages: VecDeque<Item>,
        pub flag_to_hold: Flag,
        pub request_count: Cell<usize>,
        pub last_beacon: Cell<Option<Beacon>>,
        pub pending_beacon_timestamp: Cell<u64>,
        pub randomness_salt: Cell<u64>,
        pub is_inspecting: bool,
    }

    pub struct AppState {
        pub input_buffer_manager: Arc<Mutex<InputBufferManager>>,
        pub drand_period: u64,
        pub drand_genesis_time: u64,
        pub safe_seconds: u64,
        pub version: String,
    }

    impl AppState {
        pub fn new() -> AppState {
            let manager = InputBufferManager::default();
            let drand_period = var("DRAND_PERIOD")
                .expect("Missing env DRAND_PERIOD")
                .parse::<u64>()
                .unwrap();
            let drand_genesis_time = var("DRAND_GENESIS_TIME")
                .expect("Missing env DRAND_GENESIS_TIME")
                .parse::<u64>()
                .unwrap();
            let safe_seconds = var("DRAND_SAFE_SECONDS")
                .expect("Missing env DRAND_SAFE_SECONDS")
                .parse::<u64>()
                .unwrap();
            let version: Option<&str> = option_env!("CARGO_PKG_VERSION");
            let version = version.unwrap_or("unknown").to_string();
            AppState {
                input_buffer_manager: Arc::new(Mutex::new(manager)),
                drand_period,
                drand_genesis_time,
                safe_seconds,
                version,
            }
        }
        pub fn get_randomness_for_timestamp(&self, query_timestamp: u64) -> Option<String> {
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
        pub fn keep_newest_beacon(&self, drand_beacon: DrandBeacon) {
            let beacon_time = (drand_beacon.round * self.drand_period) + self.drand_genesis_time;
            info!(
                "Calculated beacon time {} for round {}",
                beacon_time, drand_beacon.round
            );
            let manager = self.input_buffer_manager.try_lock().unwrap();
            if let Some(current_beacon) = manager.last_beacon.take() {
                if current_beacon.round < drand_beacon.round {
                    info!("Set new beacon");

                    let beacon = Beacon::builder()
                        .with_drand_beacon(&drand_beacon)
                        .with_timestamp(beacon_time)
                        .build();

                    manager.last_beacon.set(Some(beacon));
                } else {
                    info!("Keep current beacon");
                    manager.last_beacon.set(Some(current_beacon));
                }
            } else {
                info!("No beacon, initializing");

                let beacon = Beacon::builder()
                    .with_drand_beacon(&drand_beacon)
                    .with_timestamp(beacon_time)
                    .build();

                manager.last_beacon.set(Some(beacon));
            }
        }
        pub async fn store_input(&self, rollup_input: &RollupInput) -> Result<(), Box<dyn Error>> {
            let mut manager = self.input_buffer_manager.lock().await;
            let item = rollup_input.get_item();
            match item {
                Ok(item) => {
                    manager.messages.push_back(item);
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        pub async fn consume_input(&self) -> Option<Item> {
            let mut manager = self.input_buffer_manager.lock().await;
            manager.consume_input()
        }
        pub async fn set_inspecting(&self, value: bool) {
            let mut manager = self.input_buffer_manager.lock().await;
            manager.is_inspecting = value;
        }
        pub fn is_inspecting(&self) -> bool {
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

        // TODO: Check if this is necessary
        // pub fn hold_up(&mut self) {
        //     self.is_holding = true;
        // }

        pub fn release(&mut self) {
            self.is_holding = false;
        }

        #[cfg(test)]
        pub fn is_holding(&self) -> bool {
            self.is_holding.to_owned()
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
        pub fn set_pending_beacon_timestamp(&mut self, timestamp: u64) {
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

        pub fn consume_input(&mut self) -> Option<Item> {
            info!("Consuming input");
            let buffer = self.messages.borrow_mut();

            if buffer.is_empty() || self.flag_to_hold.is_holding {
                return None;
            }

            let data = buffer.pop_front();
            self.request_count.set(self.request_count.get() - 1);
            data
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use super::structs::{AppState, Beacon, DrandBeacon, InputBufferManager};

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
        let beacon = DrandBeacon::builder().with_round(2).build();
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
            let beacon = DrandBeacon::builder().with_round(1).build();
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
            let beacon = DrandBeacon::builder().with_round(3).build();
            app.keep_newest_beacon(beacon);
            let manager = app.input_buffer_manager.lock().await;
            assert_eq!(3, manager.last_beacon.take().unwrap().round);
        }
    }
}
