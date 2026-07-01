# Architecture

CosmoFlow models workflow execution as a state machine graph. The core runtime
is deliberately small: it connects nodes, routes actions, validates graph
structure, and executes one node at a time.

## Design Principles

CosmoFlow core is built around two ideas:

- Every program can be modeled as a state machine.
- The framework core should stay minimal. Runtime policy belongs in user code,
  wrapper nodes, flow composition, or optional extensions.

Because of that, retry, fallback, timeout, agent/tool/LLM integration, memory,
and tracing are not core flow semantics.

## Core Components

### Action

An `Action` is the transition signal returned by a node or flow.

It has two fields:

- `name`: the routing identity.
- `params`: optional JSON values carried with the action.

Flow routing uses only the action name. Parameters are data, not route identity.
`Display` for an action should therefore print only the name.

### Node

A `Node<S>` is the user-defined unit of behavior. It runs once through three
phases:

1. `prep`: read state and prepare input.
2. `exec`: run the main node logic.
3. `post`: write state and return an action.

Phase-aware errors preserve whether a failure happened during prep, exec, or
post. Core node execution does not retry or fallback; those policies can be
modeled above the node, inside the node, or with wrapper nodes.

### Flow

A `Flow<S>` is a state-machine graph plus a single sequential executor.

The builder registers nodes and routes. The first registered node becomes the
default start node unless `.start(id)` is used. Each route maps:

```text
source node + action name -> target node
```

Execution starts at the start node. After each node returns an action, the flow
looks for a matching route from the current node. If none exists, the flow
terminates naturally and returns that final action.

### State

`S` is the runtime state type passed through node and flow execution.

It can be:

- a strong typed application struct;
- an official `SharedStore` backend;
- any other state type that fits the application.

The core model does not require `S: SharedStore`. In async mode, `S: Send + Sync`
is required for future safety, not for storage semantics.

### SharedStore

`SharedStore` is the official dynamic key-value context model. It is useful for
workflow state that needs typed get/set operations, serialization, or pluggable
memory/file/Redis backends.

It remains a supported state model, but it is not the definition of state in the
core API.

## Build-Time Graph Validation

The flow builder validates graph structure before producing a `Flow`.

Build failures include:

- empty flow;
- duplicate node id;
- configured start node missing from the graph;
- route source or target missing;
- duplicate route for the same source node and action name;
- nodes unreachable from the start node.

Cycles are allowed because many state machines are cyclic. `FlowAnalysis`
records reachable nodes, whether the graph is a DAG, and a topological order
when one exists.

## Nested Flows

A built flow can be used as a node in another flow. The parent flow treats the
child flow as one node and sees only its final action.

If the child flow fails, the parent reports that failure as an execution-phase
error on the parent node that contained the child flow.

## Internal Adapter Boundary

Flow internals need to store heterogeneous nodes in one collection. User nodes
can have different `Prep`, `Output`, and `Error` associated types, so the flow
uses an internal adapter layer to erase those differences behind one execution
interface.

This adapter layer is not part of the user mental model. Users implement
`Node<S>` or compose `Flow<S>` values.

## Sync And Async

CosmoFlow has sync and async variants behind the `async` feature.

The public model is the same in both variants:

- nodes still run `prep -> exec -> post`;
- flows still route by action name;
- unmatched routes still terminate naturally;
- graph validation still runs at build time.

The async variant uses async node phases and async flow execution. Its extra
`Send + Sync` bounds are execution-safety bounds introduced by async futures.
