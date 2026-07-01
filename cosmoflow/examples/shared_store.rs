//! `MemoryStorage` as an optional key-value state model.
//!
//! Most examples use typed state. Use `SharedStore` when dynamic keys,
//! serialization, or pluggable storage backends are a better fit.

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
    use cosmoflow::shared_store::SharedStore;
    use cosmoflow::shared_store::backends::MemoryStorage;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct ExampleError(String);

    impl ExampleError {
        fn new(message: impl Into<String>) -> Self {
            Self(message.into())
        }
    }

    impl fmt::Display for ExampleError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl Error for ExampleError {}

    struct ReadDynamicValue;

    impl Node<MemoryStorage> for ReadDynamicValue {
        type Prep = String;
        type Output = String;
        type Error = ExampleError;

        fn prep(
            &mut self,
            store: &MemoryStorage,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            store
                .get("user_name")
                .map_err(|error| ExampleError::new(error.to_string()))?
                .ok_or_else(|| ExampleError::new("missing user_name"))
        }

        fn exec(
            &mut self,
            name: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(format!("Hello from MemoryStorage, {name}!"))
        }

        fn post(
            &mut self,
            store: &mut MemoryStorage,
            _prep: Self::Prep,
            output: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            store
                .set("message".to_string(), output)
                .map_err(|error| ExampleError::new(error.to_string()))?;
            Ok(Action::new("complete"))
        }
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut store = MemoryStorage::new();
        store.set("user_name".to_string(), "Ada".to_string())?;

        let mut flow = FlowBuilder::new()
            .node("read_dynamic_value", ReadDynamicValue)
            .build()?;
        flow.run(&mut store)?;

        let message = store
            .get::<String>("message")?
            .ok_or_else(|| ExampleError::new("missing message"))?;

        println!("{message}");
        Ok(())
    }
}
