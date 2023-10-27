use exts::MutexNetplayExtensions;
use matchbox_socket::WebRtcSocket;
use once_cell::sync::{Lazy, OnceCell};
use std::sync::{Arc, Mutex};

pub mod exts;
pub mod ffi;
pub mod peer_msg;
pub mod safe_bytes;

static mut SOCKET: OnceCell<Mutex<Option<WebRtcSocket>>> = OnceCell::new();
static SHOULD_STOP: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

unsafe fn get_socket_instance() -> &'static Mutex<Option<WebRtcSocket>> {
    let mutex = SOCKET.get_or_init(|| Mutex::new(None));

    mutex.ensure_not_poisoned();

    mutex
}

unsafe fn reset_socket_instance() {
    SOCKET.take();

    match SOCKET.set(Mutex::new(None)) {
        Ok(_) => {}
        Err(_) => {}
    }
}
