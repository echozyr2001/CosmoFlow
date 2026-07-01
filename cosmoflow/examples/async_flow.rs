//! Async feature example.
//!
//! This is the only example that implements the async `Node` trait. The other
//! examples stay sync-focused so their core concepts remain easy to read.

#[cfg(not(feature = "async"))]
fn main() {
    println!(
        "Run this example with `cargo run -p cosmoflow --example async_flow --features async`."
    );
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async_example::run().await
}

#[cfg(feature = "async")]
mod async_example {
    use async_trait::async_trait;
    use cosmoflow::action::Action;
    use cosmoflow::flow::FlowBuilder;
    use cosmoflow::node::{Node, NodeContext};
    use std::convert::Infallible;

    #[derive(Default)]
    struct AsyncState {
        events: Vec<&'static str>,
    }

    struct Fetch;
    struct Save;

    #[async_trait]
    impl Node<AsyncState> for Fetch {
        type Prep = ();
        type Output = &'static str;
        type Error = Infallible;

        async fn prep(
            &mut self,
            _state: &AsyncState,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            Ok(())
        }

        async fn exec(
            &mut self,
            _prep: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            // Real async nodes can await I/O here.
            Ok("payload")
        }

        async fn post(
            &mut self,
            state: &mut AsyncState,
            _prep: Self::Prep,
            output: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            state.events.push(output);
            Ok(Action::new("save"))
        }
    }

    #[async_trait]
    impl Node<AsyncState> for Save {
        type Prep = ();
        type Output = ();
        type Error = Infallible;

        async fn prep(
            &mut self,
            _state: &AsyncState,
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
            state: &mut AsyncState,
            _prep: Self::Prep,
            _output: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            state.events.push("saved");
            Ok(Action::new("complete"))
        }
    }

    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = AsyncState::default();
        let mut flow = FlowBuilder::new()
            .node("fetch", Fetch)
            .node("save", Save)
            .route("fetch", "save", "save")
            .build()?;

        let execution = flow.run_recorded(&mut state).await?;

        println!("events: {:?}", state.events);
        println!("path: {:?}", execution.path);
        println!("final action: {}", execution.final_action);
        Ok(())
    }
}
