use super::{NodeContext, NodeError, NodePhase};
use crate::action::Action;
use crate::shared_store::SharedStore;
use async_trait::async_trait;

/// Node trait for the v2 asynchronous single-run execution model.
#[async_trait]
pub trait Node<S: SharedStore>: Send + Sync {
    /// Result type produced by the preparation phase.
    type Prep: Send + Sync + 'static;
    /// Result type produced by the execution phase.
    type Output: Send + Sync + 'static;
    /// Error type returned by node phases.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Read state and prepare input for execution.
    async fn prep(&mut self, state: &S, context: &NodeContext) -> Result<Self::Prep, Self::Error>;

    /// Execute node logic once.
    async fn exec(
        &mut self,
        prep: &Self::Prep,
        context: &NodeContext,
    ) -> Result<Self::Output, Self::Error>;

    /// Write state and return the next action.
    async fn post(
        &mut self,
        state: &mut S,
        prep: Self::Prep,
        output: Self::Output,
        context: &NodeContext,
    ) -> Result<Action, Self::Error>;

    /// Return a human-readable node name for diagnostics.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Run a node once through `prep -> exec -> post`.
    async fn run(&mut self, state: &mut S, context: NodeContext) -> Result<Action, NodeError> {
        let prep = self.prep(state, &context).await.map_err(|error| {
            NodeError::new(NodePhase::Prep, context.node_id.clone(), error.to_string())
        })?;

        let output = self.exec(&prep, &context).await.map_err(|error| {
            NodeError::new(NodePhase::Exec, context.node_id.clone(), error.to_string())
        })?;

        self.post(state, prep, output, &context)
            .await
            .map_err(|error| {
                NodeError::new(NodePhase::Post, context.node_id.clone(), error.to_string())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SharedStore;
    use crate::shared_store::backends::MemoryStorage;
    use std::error::Error;
    use std::fmt;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct TestError(&'static str);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl Error for TestError {}

    #[derive(Clone)]
    struct Calls(Arc<Mutex<Vec<&'static str>>>);

    impl Calls {
        fn new() -> Self {
            Self(Arc::new(Mutex::new(Vec::new())))
        }

        fn push(&self, call: &'static str) {
            self.0.lock().unwrap().push(call);
        }

        fn snapshot(&self) -> Vec<&'static str> {
            self.0.lock().unwrap().clone()
        }
    }

    struct RecordingNode {
        calls: Calls,
        fail_phase: Option<NodePhase>,
        exec_calls: usize,
    }

    impl RecordingNode {
        fn new(calls: Calls) -> Self {
            Self {
                calls,
                fail_phase: None,
                exec_calls: 0,
            }
        }

        fn failing(calls: Calls, phase: NodePhase) -> Self {
            Self {
                calls,
                fail_phase: Some(phase),
                exec_calls: 0,
            }
        }
    }

    #[async_trait]
    impl Node<MemoryStorage> for RecordingNode {
        type Prep = String;
        type Output = String;
        type Error = TestError;

        async fn prep(
            &mut self,
            _state: &MemoryStorage,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            self.calls.push("prep");
            if self.fail_phase == Some(NodePhase::Prep) {
                return Err(TestError("prep failed"));
            }
            Ok("prepared".to_string())
        }

        async fn exec(
            &mut self,
            prep: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            self.calls.push("exec");
            self.exec_calls += 1;
            assert_eq!(prep, "prepared");
            if self.fail_phase == Some(NodePhase::Exec) {
                return Err(TestError("exec failed"));
            }
            Ok("output".to_string())
        }

        async fn post(
            &mut self,
            state: &mut MemoryStorage,
            prep: Self::Prep,
            output: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            self.calls.push("post");
            assert_eq!(prep, "prepared");
            assert_eq!(output, "output");
            if self.fail_phase == Some(NodePhase::Post) {
                return Err(TestError("post failed"));
            }
            state
                .set("async_result".to_string(), output)
                .map_err(|_| TestError("storage failed"))?;
            Ok(Action::simple("complete"))
        }
    }

    #[tokio::test]
    async fn successful_run_executes_prep_exec_post_once() {
        let calls = Calls::new();
        let mut node = RecordingNode::new(calls.clone());
        let mut state = MemoryStorage::new();
        let context = NodeContext::new("async_node");

        let action = node.run(&mut state, context).await.unwrap();
        let stored: Option<String> = state.get("async_result").unwrap();

        assert_eq!(action, Action::simple("complete"));
        assert_eq!(stored, Some("output".to_string()));
        assert_eq!(calls.snapshot(), vec!["prep", "exec", "post"]);
        assert_eq!(node.exec_calls, 1);
    }

    #[tokio::test]
    async fn prep_failure_stops_before_exec_and_post() {
        let calls = Calls::new();
        let mut node = RecordingNode::failing(calls.clone(), NodePhase::Prep);
        let mut state = MemoryStorage::new();

        let error = node
            .run(&mut state, NodeContext::new("async_node"))
            .await
            .unwrap_err();

        assert_eq!(error.phase, NodePhase::Prep);
        assert_eq!(error.node_id.as_str(), "async_node");
        assert_eq!(error.message, "prep failed");
        assert_eq!(calls.snapshot(), vec!["prep"]);
    }

    #[tokio::test]
    async fn exec_failure_is_not_retried_and_skips_post() {
        let calls = Calls::new();
        let mut node = RecordingNode::failing(calls.clone(), NodePhase::Exec);
        let mut state = MemoryStorage::new();

        let error = node
            .run(&mut state, NodeContext::new("async_node"))
            .await
            .unwrap_err();

        assert_eq!(error.phase, NodePhase::Exec);
        assert_eq!(error.message, "exec failed");
        assert_eq!(calls.snapshot(), vec!["prep", "exec"]);
        assert_eq!(node.exec_calls, 1);
    }

    #[tokio::test]
    async fn post_failure_reports_post_phase() {
        let calls = Calls::new();
        let mut node = RecordingNode::failing(calls.clone(), NodePhase::Post);
        let mut state = MemoryStorage::new();

        let error = node
            .run(&mut state, NodeContext::new("async_node"))
            .await
            .unwrap_err();

        assert_eq!(error.phase, NodePhase::Post);
        assert_eq!(error.message, "post failed");
        assert_eq!(calls.snapshot(), vec!["prep", "exec", "post"]);
    }
}
