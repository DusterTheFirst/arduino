//! Utilities for interfacing with the teensy 4's integrated temperature monitor

extern "C" {
    fn tempmonGetTemp() -> f32;
}

/// Get the teensy's temperature in degrees celsius
pub fn get_temp() -> f32 {
    unsafe { tempmonGetTemp() }
}
