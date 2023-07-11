pub mod models {
    use std::{
        borrow::BorrowMut,
        cell::Cell,
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use serde::{Deserialize, Serialize};

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
    }

    pub(crate) struct AppState {
        pub(crate) input_buffer_manager: Arc<Mutex<InputBufferManager>>,
    }

    impl Flag {
        fn new() -> Flag {
            Flag { is_holding: true }
        }

        pub(crate) fn hold_up(&mut self) {
            self.is_holding = true;
        }

        pub(crate) fn release(&mut self) {
            self.is_holding = false;
        }
    }

    impl InputBufferManager {
        pub(crate) fn new() -> InputBufferManager {
            InputBufferManager {
                messages: VecDeque::new(),
                flag_to_hold: Flag::new(),
                request_count: Cell::new(0),
                last_beacon: Cell::new(None),
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
