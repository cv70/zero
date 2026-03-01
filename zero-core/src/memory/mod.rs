/// Memory module for global shared memory
pub mod backend;
pub mod memory;

pub use crate::memory::backend::FilesystemMemory;
pub use crate::memory::memory::{GlobalSharedMemory, MemoryEntry};
