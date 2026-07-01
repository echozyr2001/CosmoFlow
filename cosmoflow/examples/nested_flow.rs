//! Nested flows.
//!
//! A built `Flow<S>` can be added to another `FlowBuilder` as a node. The
//! parent flow records only the parent node id; the child flow keeps its own
//! internal path private.

#[cfg(feature = "async")]
fn main() {
    println!("Run `cargo run -p cosmoflow --example async_flow --features async` for async usage.");
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    sync_example::run()
}

#[cfg(not(feature = "async"))]
mod sync_example {
    use cosmoflow::action::Action;
    use cosmoflow::flow::FlowBuilder;
    use cosmoflow::node::{Node, NodeContext};
    use std::convert::Infallible;

    #[derive(Default)]
    struct AppState {
        events: Vec<&'static str>,
    }

    struct StartChild;
    struct FinishChild;

    macro_rules! impl_child_node {
        ($node:ty, $event:literal, $action:literal) => {
            impl Node<AppState> for $node {
                type Prep = ();
                type Output = ();
                type Error = Infallible;

                fn prep(
                    &mut self,
                    _state: &AppState,
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
                    state: &mut AppState,
                    _prep: Self::Prep,
                    _output: Self::Output,
                    _context: &NodeContext,
                ) -> Result<Action, Self::Error> {
                    state.events.push($event);
                    Ok(Action::new($action))
                }
            }
        };
    }

    impl_child_node!(StartChild, "child:start", "next");
    impl_child_node!(FinishChild, "child:finish", "complete");

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let child_flow = FlowBuilder::new()
            .node("start", StartChild)
            .node("finish", FinishChild)
            .route("start", "next", "finish")
            .build()?;

        let mut parent_flow = FlowBuilder::new().node("child", child_flow).build()?;
        let mut state = AppState::default();
        let execution = parent_flow.run_recorded(&mut state)?;

        println!("events: {:?}", state.events);
        println!("parent path: {:?}", execution.path);
        println!("final action: {}", execution.final_action);
        Ok(())
    }
}
