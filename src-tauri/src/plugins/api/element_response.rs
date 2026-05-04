use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static ELEMENT_REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

static ELEMENT_RESPONSE_STORE: OnceLock<Mutex<HashMap<u64, std::sync::mpsc::Sender<String>>>> =
    OnceLock::new();

fn get_element_response_store() -> &'static Mutex<HashMap<u64, std::sync::mpsc::Sender<String>>> {
    ELEMENT_RESPONSE_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn element_response_create() -> (u64, std::sync::mpsc::Receiver<String>) {
    let id = ELEMENT_REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = std::sync::mpsc::channel();
    let mut store = get_element_response_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.insert(id, tx);
    (id, rx)
}

pub fn element_response_resolve(request_id: u64, data: String) {
    let mut store = get_element_response_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(tx) = store.remove(&request_id) {
        let _ = tx.send(data);
    }
}
