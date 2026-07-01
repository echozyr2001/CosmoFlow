use super::{FlowError, Route};
use crate::action::ActionName;
use crate::node::NodeId;
use std::collections::{HashMap, HashSet, VecDeque};

/// Static graph analysis for a flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowAnalysis {
    /// Nodes reachable from the configured start node, in discovery order.
    pub reachable_nodes: Vec<NodeId>,
    /// Whether the full flow graph is a DAG.
    pub is_dag: bool,
    /// Topological order when the full flow graph is a DAG.
    pub topological_order: Option<Vec<NodeId>>,
}

pub(crate) fn validate_graph(
    node_order: &[NodeId],
    routes: &[Route],
    start: Option<&NodeId>,
    duplicate_nodes: &[NodeId],
) -> Result<FlowAnalysis, FlowError> {
    if node_order.is_empty() {
        return Err(FlowError::EmptyFlow);
    }

    if let Some(node_id) = duplicate_nodes.first() {
        return Err(FlowError::DuplicateNode(node_id.clone()));
    }

    let start = start.expect("non-empty flow must have a start node");
    let nodes = node_set(node_order);
    if !nodes.contains(start) {
        return Err(FlowError::MissingStart(start.clone()));
    }

    validate_routes(routes, &nodes)?;

    let analysis = analyze_graph(node_order, routes, start);
    for node_id in node_order {
        if !analysis.reachable_nodes.contains(node_id) {
            return Err(FlowError::UnreachableNode(node_id.clone()));
        }
    }

    Ok(analysis)
}

fn validate_routes(routes: &[Route], nodes: &HashSet<NodeId>) -> Result<(), FlowError> {
    let mut route_keys = HashSet::<(NodeId, ActionName)>::new();

    for route in routes {
        if !nodes.contains(&route.from) {
            return Err(FlowError::MissingRouteSource(route.from.clone()));
        }
        if !nodes.contains(&route.to) {
            return Err(FlowError::MissingRouteTarget(route.to.clone()));
        }

        let route_key = (route.from.clone(), route.action.clone());
        if !route_keys.insert(route_key) {
            return Err(FlowError::DuplicateRoute {
                from: route.from.clone(),
                action: route.action.clone(),
            });
        }
    }

    Ok(())
}

fn analyze_graph(node_order: &[NodeId], routes: &[Route], start: &NodeId) -> FlowAnalysis {
    let adjacency = adjacency(routes);
    let reachable_nodes = reachable(node_order, &adjacency, start);
    let topological_order = topological_order(node_order, routes);
    let is_dag = topological_order.is_some();

    FlowAnalysis {
        reachable_nodes,
        is_dag,
        topological_order,
    }
}

fn node_set(node_order: &[NodeId]) -> HashSet<NodeId> {
    node_order.iter().cloned().collect()
}

fn adjacency(routes: &[Route]) -> HashMap<NodeId, Vec<NodeId>> {
    let mut adjacency = HashMap::<NodeId, Vec<NodeId>>::new();
    for route in routes {
        adjacency
            .entry(route.from.clone())
            .or_default()
            .push(route.to.clone());
    }
    adjacency
}

fn reachable(
    node_order: &[NodeId],
    adjacency: &HashMap<NodeId, Vec<NodeId>>,
    start: &NodeId,
) -> Vec<NodeId> {
    let node_positions = node_order
        .iter()
        .enumerate()
        .map(|(index, node_id)| (node_id.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut seen = HashSet::<NodeId>::new();
    let mut queue = VecDeque::from([start.clone()]);
    let mut reachable = Vec::new();

    while let Some(node_id) = queue.pop_front() {
        if !seen.insert(node_id.clone()) {
            continue;
        }
        reachable.push(node_id.clone());

        if let Some(targets) = adjacency.get(&node_id) {
            let mut targets = targets.clone();
            targets.sort_by_key(|target| {
                *node_positions
                    .get(target)
                    .expect("route targets are validated before reachability analysis")
            });
            for target in targets {
                queue.push_back(target);
            }
        }
    }

    reachable
}

fn topological_order(node_order: &[NodeId], routes: &[Route]) -> Option<Vec<NodeId>> {
    let mut indegree = node_order
        .iter()
        .cloned()
        .map(|node_id| (node_id, 0usize))
        .collect::<HashMap<_, _>>();
    let mut adjacency = HashMap::<NodeId, Vec<NodeId>>::new();

    for route in routes {
        if let Some(count) = indegree.get_mut(&route.to) {
            *count += 1;
        }
        adjacency
            .entry(route.from.clone())
            .or_default()
            .push(route.to.clone());
    }

    let mut ready = node_order
        .iter()
        .filter(|node_id| indegree.get(*node_id).copied() == Some(0))
        .cloned()
        .collect::<VecDeque<_>>();
    let mut sorted = Vec::new();

    while let Some(node_id) = ready.pop_front() {
        sorted.push(node_id.clone());
        if let Some(targets) = adjacency.get(&node_id) {
            for target in targets {
                let count = indegree
                    .get_mut(target)
                    .expect("route targets are validated before analysis");
                *count -= 1;
                if *count == 0 {
                    ready.push_back(target.clone());
                }
            }
        }
    }

    if sorted.len() == node_order.len() {
        Some(sorted)
    } else {
        None
    }
}
