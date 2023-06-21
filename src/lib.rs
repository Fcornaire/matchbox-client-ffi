use lazy_static::lazy_static;
use matchbox_socket::WebRtcSocket;
use std::sync::{Arc, Mutex};

pub mod ffi;
pub mod peer_msg;
pub mod safe_bytes;

lazy_static! {
    pub static ref SOCKET: Mutex<Option<WebRtcSocket>> = Mutex::new(None);
    pub static ref SHOULD_STOP: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}
