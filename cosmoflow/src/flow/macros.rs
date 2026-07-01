//! Macros for declarative flow construction.

/// Declarative flow construction.
///
/// This macro expands to [`FlowBuilder`](super::FlowBuilder). It only supports
/// identifier node ids and action names; use `FlowBuilder` directly for string
/// literals, dynamic names, or names that are not valid Rust identifiers.
///
/// # Examples
///
/// ```rust,ignore
/// let flow = cosmoflow::flow::flow! {
///     nodes {
///         load = LoadUser;
///         score = ScoreUser;
///         done = Done;
///     }
///
///     routes {
///         load => next => score;
///         score => complete => done;
///     }
/// }?;
/// ```
///
/// Explicit start node:
///
/// ```rust,ignore
/// let flow = cosmoflow::flow::flow! {
///     start = score;
///
///     nodes {
///         load = LoadUser;
///         score = ScoreUser;
///     }
///
///     routes {
///         score => complete => load;
///     }
/// }?;
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __cosmoflow_flow {
    (
        start = $start:ident;

        nodes {
            $first_id:ident = $first_node:expr;
            $(
                $id:ident = $node:expr;
            )*
        }

        routes {
            $(
                $from:ident => $action:ident => $to:ident;
            )*
        }
    ) => {
        {
            $crate::flow::FlowBuilder::new()
                .node(stringify!($first_id), $first_node)
                $(
                    .node(stringify!($id), $node)
                )*
                .start(stringify!($start))
                $(
                    .route(stringify!($from), stringify!($action), stringify!($to))
                )*
                .build()
        }
    };

    (
        nodes {
            $first_id:ident = $first_node:expr;
            $(
                $id:ident = $node:expr;
            )*
        }

        routes {
            $(
                $from:ident => $action:ident => $to:ident;
            )*
        }
    ) => {
        {
            $crate::flow::FlowBuilder::new()
                .node(stringify!($first_id), $first_node)
                $(
                    .node(stringify!($id), $node)
                )*
                $(
                    .route(stringify!($from), stringify!($action), stringify!($to))
                )*
                .build()
        }
    };
}

pub use crate::__cosmoflow_flow as flow;

#[cfg(all(test, feature = "storage-memory"))]
mod tests {
    use crate::action::Action;
    #[cfg(not(feature = "async"))]
    use crate::flow::FlowBuilder;
    use crate::flow::FlowError;
    use crate::node::{Node, NodeContext, NodeId};
    use crate::shared_store::backends::MemoryStorage;
    use std::convert::Infallible;

    struct StaticNode {
        action: &'static str,
    }

    impl StaticNode {
        fn new(action: &'static str) -> Self {
            Self { action }
        }
    }

    #[cfg(not(feature = "async"))]
    impl Node<MemoryStorage> for StaticNode {
        type Prep = ();
        type Output = ();
        type Error = Infallible;

        fn prep(
            &mut self,
            _state: &MemoryStorage,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            Ok(())
        }

        fn exec(
            &mut self,
            _prep: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(())
        }

        fn post(
            &mut self,
            _state: &mut MemoryStorage,
            _prep: Self::Prep,
            _output: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            Ok(Action::new(self.action))
        }
    }

    #[cfg(feature = "async")]
    #[async_trait::async_trait]
    impl Node<MemoryStorage> for StaticNode {
        type Prep = ();
        type Output = ();
        type Error = Infallible;

        async fn prep(
            &mut self,
            _state: &MemoryStorage,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            Ok(())
        }

        async fn exec(
            &mut self,
            _prep: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(())
        }

        async fn post(
            &mut self,
            _state: &mut MemoryStorage,
            _prep: Self::Prep,
            _output: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            Ok(Action::new(self.action))
        }
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn macro_matches_manual_builder() {
        let mut macro_flow = crate::flow::flow! {
            nodes {
                load = StaticNode::new("next");
                score = StaticNode::new("complete");
            }

            routes {
                load => next => score;
            }
        }
        .unwrap();

        let mut manual_flow = FlowBuilder::new()
            .node("load", StaticNode::new("next"))
            .node("score", StaticNode::new("complete"))
            .route("load", "next", "score")
            .build()
            .unwrap();

        let mut macro_state = MemoryStorage::new();
        let mut manual_state = MemoryStorage::new();
        let macro_execution = macro_flow.run_recorded(&mut macro_state).unwrap();
        let manual_execution = manual_flow.run_recorded(&mut manual_state).unwrap();

        assert_eq!(macro_execution.path, manual_execution.path);
        assert_eq!(macro_execution.final_action, manual_execution.final_action);
        assert_eq!(macro_execution.last_node_id, manual_execution.last_node_id);
        assert_eq!(macro_execution.steps, manual_execution.steps);
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn first_node_is_default_start() {
        let flow = crate::flow::flow! {
            nodes {
                load = StaticNode::new("complete");
                score = StaticNode::new("complete");
            }

            routes {
                load => complete => score;
            }
        }
        .unwrap();

        assert_eq!(flow.start().as_str(), "load");
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn explicit_start_overrides_default_start() {
        let mut flow = crate::flow::flow! {
            start = score;

            nodes {
                load = StaticNode::new("complete");
                score = StaticNode::new("complete");
            }

            routes {
                score => complete => load;
            }
        }
        .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).unwrap();

        assert_eq!(flow.start().as_str(), "score");
        assert_eq!(
            execution.path,
            vec![NodeId::new("score"), NodeId::new("load")]
        );
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn empty_routes_naturally_terminate() {
        let mut flow = crate::flow::flow! {
            nodes {
                done = StaticNode::new("complete");
            }

            routes {}
        }
        .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).unwrap();

        assert_eq!(execution.path, vec![NodeId::new("done")]);
        assert_eq!(execution.final_action, Action::new("complete"));
    }

    #[test]
    fn build_time_graph_errors_are_preserved() {
        let error = crate::flow::flow! {
            nodes {
                start = StaticNode::new("complete");
                orphan = StaticNode::new("complete");
            }

            routes {}
        }
        .unwrap_err();

        assert_eq!(error, FlowError::UnreachableNode(NodeId::new("orphan")));
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn nested_flow_nodes_are_accepted() {
        let inner = crate::flow::flow! {
            nodes {
                inner_start = StaticNode::new("next");
                inner_end = StaticNode::new("inner_done");
            }

            routes {
                inner_start => next => inner_end;
            }
        }
        .unwrap();

        let mut outer = crate::flow::flow! {
            nodes {
                nested = inner;
            }

            routes {}
        }
        .unwrap();
        let mut state = MemoryStorage::new();

        let execution = outer.run_recorded(&mut state).unwrap();

        assert_eq!(execution.path, vec![NodeId::new("nested")]);
        assert_eq!(execution.final_action, Action::new("inner_done"));
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn macro_builds_and_runs_async_flow() {
        let mut flow = crate::flow::flow! {
            nodes {
                load = StaticNode::new("next");
                score = StaticNode::new("complete");
            }

            routes {
                load => next => score;
            }
        }
        .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).await.unwrap();

        assert_eq!(flow.start().as_str(), "load");
        assert_eq!(
            execution.path,
            vec![NodeId::new("load"), NodeId::new("score")]
        );
        assert_eq!(execution.final_action, Action::new("complete"));
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn explicit_start_works_for_async_flow() {
        let mut flow = crate::flow::flow! {
            start = score;

            nodes {
                load = StaticNode::new("complete");
                score = StaticNode::new("complete");
            }

            routes {
                score => complete => load;
            }
        }
        .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).await.unwrap();

        assert_eq!(flow.start().as_str(), "score");
        assert_eq!(
            execution.path,
            vec![NodeId::new("score"), NodeId::new("load")]
        );
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn async_nested_flow_nodes_are_accepted() {
        let inner = crate::flow::flow! {
            nodes {
                inner_start = StaticNode::new("next");
                inner_end = StaticNode::new("inner_done");
            }

            routes {
                inner_start => next => inner_end;
            }
        }
        .unwrap();

        let mut outer = crate::flow::flow! {
            nodes {
                nested = inner;
            }

            routes {}
        }
        .unwrap();
        let mut state = MemoryStorage::new();

        let execution = outer.run_recorded(&mut state).await.unwrap();

        assert_eq!(execution.path, vec![NodeId::new("nested")]);
        assert_eq!(execution.final_action, Action::new("inner_done"));
    }
}
