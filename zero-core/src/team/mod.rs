/// Multi-agent team coordination
///
/// S9-S12: Agent Teams, Protocols, Autonomy, Worktree Isolation
pub mod coordinator;
pub mod protocol;

pub use coordinator::{DefaultTeamCoordinator, TeamCoordinator};
pub use protocol::{MessageType, TeamMessage};
