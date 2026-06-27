pub enum Action {
    Next,
    Prev,
    MoveNext,
    MovePrev,
    Back,
}

impl Action {
    pub fn ipc_command(&self) -> &'static str {
        match self {
            Self::Next => "workspace next_on_output",
            Self::Prev => "workspace prev_on_output",
            Self::MoveNext => {
                "move container to workspace next_on_output; workspace next_on_output"
            }
            Self::MovePrev => {
                "move container to workspace prev_on_output; workspace prev_on_output"
            }
            Self::Back => "workspace back_and_forth",
        }
    }
}
