//! Centralized WASM debug logging for the kinematics demo.
//!
//! Provides `log_always` (always-on, for init and first-frames) and `log` (only when
//! debug mode is enabled via `?debug=1` or `set_debug_kinematics(true)`).

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static DEBUG_KINEMATICS: RefCell<bool> = RefCell::new(false);
}

#[cfg(target_arch = "wasm32")]
static FRAME_COUNTER: AtomicU32 = AtomicU32::new(0);

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CURRENT_FRAME: RefCell<u32> = RefCell::new(0);
}

/// Increment frame counter and return new value. Call at start of each step_ik.
#[must_use]
pub fn next_frame() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        let n = FRAME_COUNTER.fetch_add(1, Ordering::Relaxed);
        CURRENT_FRAME.with(|c| *c.borrow_mut() = n);
        n
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

/// Current frame number (set by next_frame). Used by sync/step_ik for throttling.
#[allow(dead_code)]
#[must_use]
pub fn current_frame() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        CURRENT_FRAME.with(|c| *c.borrow())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

/// Set whether verbose debug logging is enabled. Call from JS or after parsing URL.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn set_debug_kinematics(enabled: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        DEBUG_KINEMATICS.with(|c| *c.borrow_mut() = enabled);
    }
    let _ = enabled;
}

/// Return whether debug logging is enabled.
#[must_use]
pub fn get_debug_kinematics() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        DEBUG_KINEMATICS.with(|c| *c.borrow())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Log a message to the console. Only logs when `target_arch = "wasm32"` and debug mode is enabled.
pub fn log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if DEBUG_KINEMATICS.with(|c| *c.borrow()) {
            let _ = web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(msg));
        }
    }
    let _ = msg;
}

/// Log a message to the console. Always logs when `target_arch = "wasm32"` (for init and first-frames).
/// On native, logs to stderr.
pub fn log_always(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(msg));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("{}", msg);
    }
}

/// Parse URL search for `debug=1` or `verbose=1` and enable debug if found.
/// Call at init after engine is ready.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn maybe_enable_debug_from_url() {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return;
        };
        let location = window.location();
        let loc: String = match location.search() {
            Ok(s) => s,
            Err(_) => return,
        };
        if loc.contains("debug=1") || loc.contains("verbose=1") {
            set_debug_kinematics(true);
            log_always("[kinematics] debug enabled from URL");
        }
    }
}
