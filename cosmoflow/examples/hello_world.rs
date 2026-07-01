//! Basic flow with strongly typed state.
//!
//! This is the smallest useful CosmoFlow program: one node runs once, writes
//! state in `post`, returns an action, and the flow naturally terminates because
//! there is no route for that action.

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

    struct AppState {
        name: String,
        greeting: Option<String>,
    }

    struct Greet;

    impl Node<AppState> for Greet {
        type Prep = String;
        type Output = String;
        type Error = Infallible;

        fn prep(
            &mut self,
            state: &AppState,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            // `prep` reads state and prepares the input for `exec`.
            Ok(state.name.clone())
        }

        fn exec(
            &mut self,
            name: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            // `exec` is the core computation. It does not mutate workflow state.
            Ok(format!("Hello, {name}!"))
        }

        fn post(
            &mut self,
            state: &mut AppState,
            _prep: Self::Prep,
            greeting: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            // `post` commits state changes and returns the transition signal.
            state.greeting = Some(greeting);
            Ok(Action::new("complete"))
        }
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = AppState {
            name: "CosmoFlow".to_string(),
            greeting: None,
        };

        let mut flow = FlowBuilder::new().node("greet", Greet).build()?;
        let execution = flow.run_recorded(&mut state)?;

        println!("path: {:?}", execution.path);
        println!("final action: {}", execution.final_action);
        let greeting = state
            .greeting
            .as_deref()
            .ok_or("expected greeting to be written")?;
        println!("{greeting}");
        Ok(())
    }
}
