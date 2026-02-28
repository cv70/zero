/// Memory module for global shared memory
pub mod memory;
pub mod search;
pub mod backend;

pub use crate::memory::backend::FilesystemMemory;
pub use crate::memory::memory::{GlobalSharedMemory, MemoryEntry};
