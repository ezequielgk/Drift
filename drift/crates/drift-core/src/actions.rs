// drift never modifies workspace layout or split direction.
// Sway manages tiling internally; we only move focus and containers.
pub enum Action {
    Next,
    Prev,
    MoveNext,
    MovePrev,
    Back,
}

impl Action {
    /// Build the IPC command string given the current workspace number.
    /// `current` is the focused workspace number from GET_WORKSPACES.
    pub fn ipc_command_for(&self, current: u32) -> String {
        match self {
            Self::Next => format!("workspace number {}", current + 1),
            Self::Prev => format!("workspace number {}", current.max(2) - 1),
            Self::MoveNext => {
                format!(
                    "move container to workspace number {0}; workspace number {0}",
                    current + 1
                )
            }
            Self::MovePrev => {
                format!(
                    "move container to workspace number {0}; workspace number {0}",
                    current.max(2) - 1
                )
            }
            Self::Back => "workspace back_and_forth".to_string(),
        }
    }
}
