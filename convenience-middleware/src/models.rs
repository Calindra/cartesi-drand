pub mod models {
    use std::{borrow::BorrowMut, cell::Cell, collections::VecDeque, sync::Arc};

    use serde::{Deserialize, Serialize};
    use tokio::sync::Mutex;

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
        pub(crate) metadata: String,
    }

    pub(crate) struct InputBufferManager {
        pub(crate) messages: VecDeque<Item>,
        pub(crate) flag_to_hold: Flag,
        pub(crate) request_count: Cell<usize>,
        pub(crate) last_beacon: Cell<Option<Beacon>>,
        pub(crate) pending_beacon_timestamp: Cell<u64>,
        pub(crate) randomness_salt: Cell<u64>,
    }

    pub(crate) struct AppState {
        pub(crate) input_buffer_manager: Arc<Mutex<InputBufferManager>>,
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
            }
        }
    }

    impl InputBufferManager {
        pub(crate) fn new() -> Arc<Mutex<InputBufferManager>> {
            let buffer = InputBufferManager::default();

            Arc::new(Mutex::new(buffer))
        }

        pub(crate) fn set_pending_beacon_timestamp(&mut self, timestamp: u64) {
            let current = self.pending_beacon_timestamp.take();
            // mantendo o mais recente para economizar transacoes
            if current == 0 || current < timestamp {
                println!("pending beacon timestamp {} changed", timestamp);
                self.pending_beacon_timestamp.set(timestamp);
            } else {
                println!("pending beacon timestamp {} still the same", current);
                self.pending_beacon_timestamp.set(current);
            }
        }

        pub(crate) fn consume_input(&mut self) -> Option<Item> {
            println!("Consuming input");
            let buffer = self.messages.borrow_mut();

            if buffer.is_empty() || self.flag_to_hold.is_holding {
                return None;
            }

            let data = buffer.pop_front();
            self.request_count.set(self.request_count.get() - 1);
            data
        }

        pub(crate) fn await_beacon(&mut self) {
            println!("Awaiting beacon");

            self.flag_to_hold.hold_up();
        }
    }
}
