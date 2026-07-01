//! Loops as ordinary state-machine routes.
//!
//! CosmoFlow core does not impose a step limit. A loop continues while the
//! current action has a matching route.

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
    struct CounterState {
        count: u32,
    }

    struct Counter {
        limit: u32,
    }

    impl Node<CounterState> for Counter {
        type Prep = u32;
        type Output = u32;
        type Error = Infallible;

        fn prep(
            &mut self,
            state: &CounterState,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            Ok(state.count)
        }

        fn exec(
            &mut self,
            count: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(count + 1)
        }

        fn post(
            &mut self,
            state: &mut CounterState,
            _prep: Self::Prep,
            next_count: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            state.count = next_count;

            if next_count >= self.limit {
                // No route is registered for `done`, so the flow terminates.
                Ok(Action::new("done"))
            } else {
                Ok(Action::new("again"))
            }
        }
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = CounterState::default();
        let mut flow = FlowBuilder::new()
            .node("counter", Counter { limit: 3 })
            .route("counter", "again", "counter")
            .build()?;

        let execution = flow.run_recorded(&mut state)?;

        println!("steps: {}", execution.steps);
        println!("path: {:?}", execution.path);
        println!("count: {}", state.count);
        println!("final action: {}", execution.final_action);
        Ok(())
    }
}
