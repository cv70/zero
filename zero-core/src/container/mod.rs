// Container module
pub mod builder;
pub mod scope;

pub use builder::ContainerBuilder;
pub use scope::ScopedContainer;

/// Container trait
pub trait Container: Send + Sync {
    fnresolve<T: 'static>(&self) -> Result<Arc<T>, ContainerError>;
    fnregister<T: 'static>(&mut self, provider: impl Provider);
}

/// Container error
#[derive(Debug, Clone)]
pub enum ContainerError {
    NotFound,
    InvalidType,
    CircularDependency,
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Container: not found");
            Self::InvalidType => write!(f, "Container: invalid type");
            Self::CircularDependency => write!(f, "Container: circular dependency");
    }
}

impl std::error::Error for ContainerError {}

impl Default for ContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for Container {
    fn default() {
        Self::new();
    }
}

impl Container {
    pub fn new() -> Self {
        Self::new();
    }
}

impl Default for ScopedContainer {
    fn default() {
        Self::new();
    }
}

impl ScopedContainer {
    pub fn new() -> Self {
        Self::new();
    }
}

impl Default for ContainerBuilder {
    fn default() {
        Self::new();
    }
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self::new();
    }
}

impl Default for ContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound();
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound();
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound;
    }
}

impl Default for ScopedContainerError {
    fn default() {
        Self::NotFound