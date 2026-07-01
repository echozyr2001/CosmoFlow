//! Custom nodes using a strongly typed state.
//!
//! `SharedStore` is useful for dynamic key-value state, but the core API also
//! accepts ordinary Rust structs as workflow state.

use cosmoflow::action::Action;
use cosmoflow::flow::FlowBuilder;
use cosmoflow::node::{Node, NodeContext};
use std::error::Error;
use std::fmt;

#[cfg(feature = "async")]
use async_trait::async_trait;

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
        write!(f, "{}", self.0)
    }
}

impl Error for ExampleError {}

struct AnalyzeSamples;

#[cfg(not(feature = "async"))]
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

#[cfg(feature = "async")]
#[async_trait]
impl Node<AnalyticsState> for AnalyzeSamples {
    type Prep = Vec<u32>;
    type Output = f64;
    type Error = ExampleError;

    async fn prep(
        &mut self,
        state: &AnalyticsState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        if state.samples.is_empty() {
            return Err(ExampleError::new("samples cannot be empty"));
        }
        Ok(state.samples.clone())
    }

    async fn exec(
        &mut self,
        samples: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        let sum: u32 = samples.iter().sum();
        Ok(sum as f64 / samples.len() as f64)
    }

    async fn post(
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

#[cfg(not(feature = "async"))]
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

#[cfg(feature = "async")]
#[async_trait]
impl Node<AnalyticsState> for WriteReport {
    type Prep = f64;
    type Output = String;
    type Error = ExampleError;

    async fn prep(
        &mut self,
        state: &AnalyticsState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        state
            .average
            .ok_or_else(|| ExampleError::new("missing average"))
    }

    async fn exec(
        &mut self,
        average: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        Ok(format!("average score: {:.2}", *average))
    }

    async fn post(
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

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn Error>> {
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

    println!("final action: {}", execution.final_action);
    println!("path: {:?}", execution.path);
    println!("{}", state.report.unwrap_or_default());
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut state = AnalyticsState {
        samples: vec![91, 88, 95, 82],
        ..AnalyticsState::default()
    };

    let mut flow = FlowBuilder::new()
        .node("analyze", AnalyzeSamples)
        .node("report", WriteReport)
        .route("analyze", "report", "report")
        .build()?;

    let execution = flow.run_recorded(&mut state).await?;

    println!("final action: {}", execution.final_action);
    println!("path: {:?}", execution.path);
    println!("{}", state.report.unwrap_or_default());
    Ok(())
}
