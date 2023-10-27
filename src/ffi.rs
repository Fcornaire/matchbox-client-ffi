use futures::FutureExt;
use futures_timer::Delay;
use matchbox_socket::{PeerId, PeerState, WebRtcSocket};
use tokio::{runtime::Runtime, select};
use uuid::Uuid;

use core::slice;
use std::{mem::forget, os::raw::c_char, time::Duration};

use crate::{
    get_socket_instance, peer_msg::PeerMessage, reset_socket_instance, safe_bytes::SafeBytes,
    SHOULD_STOP,
};

#[no_mangle]
pub unsafe extern "C" fn initialize(room_url: *mut c_char) {
    let mut guard: std::sync::MutexGuard<Option<WebRtcSocket>> =
        get_socket_instance().lock().unwrap();
    let remote_addr = to_string(room_url).unwrap();

    let (socket, future_msg) = WebRtcSocket::new_reliable(&remote_addr);

    *SHOULD_STOP.lock().unwrap() = false;

    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let loop_fut = future_msg.fuse();
            futures::pin_mut!(loop_fut);

            let timeout = Delay::new(Duration::from_millis(10));
            futures::pin_mut!(timeout);

            loop {
                match SHOULD_STOP.try_lock() {
                    Ok(should_stop) => {
                        if *should_stop {
                            break;
                        }
                    }
                    Err(_) => {}
                }

                select! {
                    // Restart this loop every 100ms
                    _ = (&mut timeout).fuse() => {
                        timeout.reset(Duration::from_millis(10));
                    }

                    // Or break if the message loop ends (disconnected, closed, etc.)
                    _ = &mut loop_fut => {
                        break;
                    }
                }
            }
        });
    });

    guard.replace(socket);
}

#[no_mangle]
pub unsafe extern "C" fn disconnect() {
    reset_socket_instance();
}

#[no_mangle]
pub unsafe extern "C" fn poll_message() -> SafeBytes {
    let mut guard = get_socket_instance().lock().unwrap();

    if guard.is_some() {
        let mut socket = guard.take().unwrap();

        // Handle any new peers
        for (peer, state) in socket.update_peers() {
            match state {
                PeerState::Connected => {
                    let packet = "Greetings".as_bytes().to_vec().into_boxed_slice();
                    socket.send(packet, peer);
                }
                PeerState::Disconnected => {}
            }
        }

        let mut messages = vec![];
        // Accept any messages incoming
        for (peer, packet) in socket.receive() {
            let str = String::from_utf8_lossy(&packet);
            let msg = str.as_ref();

            let peer_msg = PeerMessage::new(
                peer,
                String::from_utf8_lossy(&packet).to_string(),
                msg.try_into().unwrap(),
            );

            messages.push(peer_msg);
        }

        let mut serialized = serde_json::to_string(&messages)
            .unwrap()
            .as_bytes()
            .to_vec();

        let safe_bytes = Box::new(SafeBytes::new(serialized.as_mut_ptr(), serialized.len()));

        forget(serialized);

        guard.replace(socket);

        return *safe_bytes;
    }

    SafeBytes::new(std::ptr::null_mut(), 0)
}

#[no_mangle]
pub unsafe extern "C" fn send_message(message: *mut c_char, peer_id: *mut c_char) {
    let mut guard = get_socket_instance().lock().unwrap();

    if guard.is_some() {
        let mut socket = guard.take().unwrap();

        let message = to_string(message).unwrap();
        let peer_id = to_string(peer_id).unwrap();

        let packet = message.as_bytes().to_vec().into_boxed_slice();
        let peer_id: PeerId = PeerId(Uuid::parse_str(&peer_id).unwrap());

        socket.send(packet, peer_id);

        guard.replace(socket);
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_messages(safe_bytes: SafeBytes) {
    let slice = safe_bytes.slice();
    drop(slice);
}

fn to_string(message: *mut c_char) -> Result<String, String> {
    let bytes = unsafe {
        assert!(!message.is_null());

        let len = libc::strlen(message as *const i8) as usize;
        let slice = slice::from_raw_parts(message as *const u8, len);
        slice
    };
    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(String::from(s)),
        Err(e) => Err(format!(
            "Error while converting message string UTF-8: {}",
            e
        )),
    }
}
