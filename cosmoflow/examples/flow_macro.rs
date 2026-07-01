//! Declarative flow construction with `cosmoflow::flow::flow!`.
//!
//! The macro is a thin `FlowBuilder` shorthand. It does not change build-time
//! validation or runtime behavior.

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
    use cosmoflow::flow::flow;
    use cosmoflow::node::{Node, NodeContext};
    use std::convert::Infallible;

    #[derive(Default)]
    struct PipelineState {
        events: Vec<&'static str>,
    }

    struct Load;
    struct Finish;

    macro_rules! impl_static_node {
        ($node:ty, $event:literal, $action:literal) => {
            impl Node<PipelineState> for $node {
                type Prep = ();
                type Output = ();
                type Error = Infallible;

                fn prep(
                    &mut self,
                    _state: &PipelineState,
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
                    state: &mut PipelineState,
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

    impl_static_node!(Load, "load", "next");
    impl_static_node!(Finish, "finish", "complete");

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = PipelineState::default();
        let mut flow = flow! {
            nodes {
                load = Load;
                finish = Finish;
            }

            routes {
                load => next => finish;
            }
        }?;

        let execution = flow.run_recorded(&mut state)?;
        println!("events: {:?}", state.events);
        println!("final action: {}", execution.final_action);
        Ok(())
    }
}
