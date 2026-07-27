use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Effect handler implementations for all 8 core effects
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectRequest {
    IO {
        op: String,
        fd: u32,
        data: Vec<u8>,
    },
    State {
        op: String,
        cell_id: u32,
        value: u64,
    },
    Async {
        op: String,
        task_id: u32,
    },
    Region {
        op: String,
        region_id: u32,
        size: u32,
    },
    GC {
        op: String,
    },
    Exception {
        error: String,
    },
    FFI {
        func_ptr: u64,
        args: Vec<u64>,
    },
    Concurrency {
        op: String,
        lock_id: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectResponse {
    IO { bytes: usize, status: i32 },
    State { old_value: u64, new_value: u64 },
    Async { task_id: u32, status: String },
    Region { ptr: u64, size: u32 },
    GC { freed_bytes: usize },
    Exception { unwind: bool },
    FFI { result: u64 },
    Concurrency { acquired: bool },
}

pub trait EffectHandler: Send + Sync {
    fn handle(&mut self, request: EffectRequest) -> io::Result<EffectResponse>;
}

pub struct IOHandler;
pub struct StateHandler {
    cells: HashMap<u32, u64>,
}
pub struct AsyncHandler;
pub struct RegionHandler;
pub struct GCHandler;
pub struct ExceptionHandler;
pub struct FFIHandler;
pub struct ConcurrencyHandler {
    locks: HashMap<u32, bool>,
}

impl StateHandler {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }
}

impl ConcurrencyHandler {
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
        }
    }
}

impl EffectHandler for IOHandler {
    fn handle(&mut self, req: EffectRequest) -> io::Result<EffectResponse> {
        match req {
            EffectRequest::IO { .. } => Ok(EffectResponse::IO {
                bytes: 0,
                status: 0,
            }),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Not IO")),
        }
    }
}

impl EffectHandler for StateHandler {
    fn handle(&mut self, req: EffectRequest) -> io::Result<EffectResponse> {
        match req {
            EffectRequest::State { cell_id, value, .. } => {
                let old = self.cells.get(&cell_id).copied().unwrap_or(0);
                self.cells.insert(cell_id, value);
                Ok(EffectResponse::State {
                    old_value: old,
                    new_value: value,
                })
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Not State")),
        }
    }
}

impl EffectHandler for AsyncHandler {
    fn handle(&mut self, req: EffectRequest) -> io::Result<EffectResponse> {
        match req {
            EffectRequest::Async { task_id, .. } => Ok(EffectResponse::Async {
                task_id,
                status: "spawned".to_string(),
            }),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Not Async")),
        }
    }
}

impl EffectHandler for RegionHandler {
    fn handle(&mut self, req: EffectRequest) -> io::Result<EffectResponse> {
        match req {
            EffectRequest::Region {
                region_id, size, ..
            } => Ok(EffectResponse::Region {
                ptr: 0x1000 + (region_id as u64 * 0x1000),
                size,
            }),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Not Region")),
        }
    }
}

impl EffectHandler for GCHandler {
    fn handle(&mut self, req: EffectRequest) -> io::Result<EffectResponse> {
        match req {
            EffectRequest::GC { .. } => Ok(EffectResponse::GC { freed_bytes: 1024 }),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Not GC")),
        }
    }
}

impl EffectHandler for ExceptionHandler {
    fn handle(&mut self, req: EffectRequest) -> io::Result<EffectResponse> {
        match req {
            EffectRequest::Exception { .. } => Ok(EffectResponse::Exception { unwind: true }),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Not Exception")),
        }
    }
}

impl EffectHandler for FFIHandler {
    fn handle(&mut self, req: EffectRequest) -> io::Result<EffectResponse> {
        match req {
            EffectRequest::FFI { .. } => Ok(EffectResponse::FFI { result: 0 }),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Not FFI")),
        }
    }
}

impl EffectHandler for ConcurrencyHandler {
    fn handle(&mut self, req: EffectRequest) -> io::Result<EffectResponse> {
        match req {
            EffectRequest::Concurrency { op, lock_id } => match op.as_str() {
                "lock" => {
                    let acquired = !self.locks.contains_key(&lock_id);
                    if acquired {
                        self.locks.insert(lock_id, true);
                    }
                    Ok(EffectResponse::Concurrency { acquired })
                }
                "unlock" => {
                    self.locks.remove(&lock_id);
                    Ok(EffectResponse::Concurrency { acquired: false })
                }
                _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown op")),
            },
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Not Concurrency",
            )),
        }
    }
}
