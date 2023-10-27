use std::sync::Mutex;

use matchbox_socket::WebRtcSocket;

use crate::reset_socket_instance;

pub trait MutexNetplayExtensions {
    unsafe fn ensure_not_poisoned(&self);
}

impl MutexNetplayExtensions for Mutex<Option<WebRtcSocket>> {
    unsafe fn ensure_not_poisoned(&self) {
        let res = match self.lock() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        };

        if res.is_err() {
            reset_socket_instance();
        }
    }
}
