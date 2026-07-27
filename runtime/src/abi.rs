//! Effect ABI: Runtime Interface
//!
//! Defines the binary interface between:
//! - Native code and managed handlers
//! - Managed code and runtime services
//! - WASM sandbox and native bridge
//!
//! This is the contract that ensures compatibility across execution modes.

use serde::{Deserialize, Serialize};

/// Effect request format (binary ABI).
#[derive(Debug, Serialize, Deserialize)]
pub struct EffectRequestAbi {
    pub request_id: u32,
    pub effect_kind: u8,
    pub payload_offset: u32,
    pub payload_size: u32,
}

/// Effect response format (binary ABI).
#[derive(Debug, Serialize, Deserialize)]
pub struct EffectResponseAbi {
    pub request_id: u32,
    pub status: i32,
    pub result_offset: u32,
    pub result_size: u32,
}

/// Effect kind codes (matching runtime enums).
pub mod effect_kinds {
    pub const IO: u8 = 0;
    pub const STATE: u8 = 1;
    pub const ASYNC: u8 = 2;
    pub const REGION: u8 = 3;
    pub const GC: u8 = 4;
    pub const EXCEPTION: u8 = 5;
    pub const FFI: u8 = 6;
    pub const CONCURRENCY: u8 = 7;
}

/// Region operation subcodes.
pub mod region_ops {
    pub const ENTER: u8 = 0;
    pub const EXIT: u8 = 1;
    pub const ALLOCATE: u8 = 2;
    pub const DEALLOCATE: u8 = 3;
}

/// IO operation subcodes.
pub mod io_ops {
    pub const READ: u8 = 0;
    pub const WRITE: u8 = 1;
    pub const OPEN: u8 = 2;
    pub const CLOSE: u8 = 3;
}

// TODO: Implement ABI marshalling (native <-> managed)
// TODO: Implement ABI versioning
// TODO: Implement ABI validation at runtime
// TODO: Implement ABI bridge for WASM sandboxing
