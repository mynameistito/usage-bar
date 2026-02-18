// ANSI Color Constants
pub const COLOR_RESET: &str = "\x1b[0m";
pub const COLOR_CYAN: &str = "\x1b[36m"; // [APP]
pub const COLOR_GREEN: &str = "\x1b[32m"; // [CLAUDE]
pub const COLOR_YELLOW: &str = "\x1b[33m"; // [ZAI]
pub const COLOR_MAGENTA: &str = "\x1b[35m"; // [CRED]
pub const COLOR_BLUE: &str = "\x1b[34m"; // [CACHE]
pub const COLOR_BRIGHT_RED: &str = "\x1b[91m"; // [NET]
pub const COLOR_RED: &str = "\x1b[31m"; // [ERROR]
pub const COLOR_GRAY: &str = "\x1b[90m"; // Timestamps

// ============================================================================
// CATEGORY-SPECIFIC MACROS (Debug builds only)
// ============================================================================

// [APP] - Cyan - Application lifecycle, startup, tray events
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_app {
    ($($arg:tt)*) => {
        println!("{}[APP]{} {}", $crate::COLOR_CYAN, $crate::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_app {
    ($($arg:tt)*) => {};
}

// [CLAUDE] - Green - Claude API calls, OAuth, usage
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_claude {
    ($($arg:tt)*) => {
        println!("{}[CLAUDE]{} {}", $crate::COLOR_GREEN, $crate::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_claude {
    ($($arg:tt)*) => {};
}

// [ZAI] - Yellow - Z.ai API calls, quota, tier
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_zai {
    ($($arg:tt)*) => {
        println!("{}[ZAI]{} {}", $crate::COLOR_YELLOW, $crate::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_zai {
    ($($arg:tt)*) => {};
}

// [CRED] - Magenta - Win32 credential operations
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_cred {
    ($($arg:tt)*) => {
        println!("{}[CRED]{} {}", $crate::COLOR_MAGENTA, $crate::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_cred {
    ($($arg:tt)*) => {};
}

// [CACHE] - Blue - Cache hits/misses, TTL expiry
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_cache {
    ($($arg:tt)*) => {
        println!("{}[CACHE]{} {}", $crate::COLOR_BLUE, $crate::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_cache {
    ($($arg:tt)*) => {};
}

// [NET] - Bright Red - HTTP requests, rate limits
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_net {
    ($($arg:tt)*) => {
        println!("{}[NET]{} {}", $crate::COLOR_BRIGHT_RED, $crate::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_net {
    ($($arg:tt)*) => {};
}

// [AMP] - Bright Cyan - Amp API calls, usage
pub const COLOR_BRIGHT_CYAN: &str = "\x1b[96m";

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_amp {
    ($($arg:tt)*) => {
        println!("{}[AMP]{} {}", $crate::COLOR_BRIGHT_CYAN, $crate::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_amp {
    ($($arg:tt)*) => {};
}

// [ERROR] - Red - Failures, exceptions, retries
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_error {
    ($($arg:tt)*) => {
        println!("{}[ERROR]{} {}", $crate::COLOR_RED, $crate::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_error {
    ($($arg:tt)*) => {};
}

// ============================================================================
// LEGACY MACRO (Deprecated - for backward compatibility)
// ============================================================================

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => { println!($($arg)*); };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}
