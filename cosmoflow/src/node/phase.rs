use std::fmt;

/// Phase of node execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodePhase {
    /// Preparing node input from state.
    Prep,
    /// Executing node logic.
    Exec,
    /// Writing state and returning an action.
    Post,
}

impl fmt::Display for NodePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodePhase::Prep => f.write_str("prep"),
            NodePhase::Exec => f.write_str("exec"),
            NodePhase::Post => f.write_str("post"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_display_is_stable() {
        assert_eq!(NodePhase::Prep.to_string(), "prep");
        assert_eq!(NodePhase::Exec.to_string(), "exec");
        assert_eq!(NodePhase::Post.to_string(), "post");
    }
}
