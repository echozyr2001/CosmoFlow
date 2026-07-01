//! Action parameters.
//!
//! Routing uses only `Action::name()`. Parameters are carried with the action
//! for the caller; they do not affect route matching.

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
    use serde_json::json;
    use std::convert::Infallible;

    #[derive(Default)]
    struct ScoreState {
        score: u32,
    }

    struct Score;

    impl Node<ScoreState> for Score {
        type Prep = u32;
        type Output = u32;
        type Error = Infallible;

        fn prep(
            &mut self,
            state: &ScoreState,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            Ok(state.score)
        }

        fn exec(
            &mut self,
            score: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(score + 7)
        }

        fn post(
            &mut self,
            state: &mut ScoreState,
            _prep: Self::Prep,
            output: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            state.score = output;
            Ok(Action::with_param("scored", "score", json!(output)))
        }
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = ScoreState { score: 35 };
        let mut flow = FlowBuilder::new().node("score", Score).build()?;
        let action = flow.run(&mut state)?;
        let score = action
            .get_param("score")
            .ok_or("expected score action param")?;

        println!("route name: {}", action.as_str());
        println!("score param: {score}");
        Ok(())
    }
}
