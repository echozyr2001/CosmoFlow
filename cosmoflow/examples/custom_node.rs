//! Custom nodes with fallible domain logic.
//!
//! This example uses strongly typed state and a small custom error type. It
//! keeps domain validation in the node implementation instead of relying on a
//! shared key-value store.

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
    use std::error::Error;
    use std::fmt;

    #[derive(Debug, Default)]
    struct AnalyticsState {
        samples: Vec<u32>,
        average: Option<f64>,
        report: Option<String>,
    }

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

    struct AnalyzeSamples;

    impl Node<AnalyticsState> for AnalyzeSamples {
        type Prep = Vec<u32>;
        type Output = f64;
        type Error = ExampleError;

        fn prep(
            &mut self,
            state: &AnalyticsState,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            if state.samples.is_empty() {
                return Err(ExampleError::new("samples cannot be empty"));
            }
            Ok(state.samples.clone())
        }

        fn exec(
            &mut self,
            samples: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            let sum: u32 = samples.iter().sum();
            Ok(sum as f64 / samples.len() as f64)
        }

        fn post(
            &mut self,
            state: &mut AnalyticsState,
            _prep: Self::Prep,
            average: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            state.average = Some(average);
            Ok(Action::new("report"))
        }
    }

    struct WriteReport;

    impl Node<AnalyticsState> for WriteReport {
        type Prep = f64;
        type Output = String;
        type Error = ExampleError;

        fn prep(
            &mut self,
            state: &AnalyticsState,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            state
                .average
                .ok_or_else(|| ExampleError::new("missing average"))
        }

        fn exec(
            &mut self,
            average: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(format!("average score: {:.2}", *average))
        }

        fn post(
            &mut self,
            state: &mut AnalyticsState,
            _prep: Self::Prep,
            report: Self::Output,
            _context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            state.report = Some(report);
            Ok(Action::new("complete"))
        }
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = AnalyticsState {
            samples: vec![91, 88, 95, 82],
            ..AnalyticsState::default()
        };

        let mut flow = FlowBuilder::new()
            .node("analyze", AnalyzeSamples)
            .node("report", WriteReport)
            .route("analyze", "report", "report")
            .build()?;

        let execution = flow.run_recorded(&mut state)?;
        let report = state
            .report
            .as_deref()
            .ok_or_else(|| ExampleError::new("missing report"))?;

        println!("path: {:?}", execution.path);
        println!("{report}");
        Ok(())
    }
}
