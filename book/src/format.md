# Test Fixture Format

There are two primary types of test fixtures: Derivation and Execution.
Test fixtures are static JSON files that live in the [fixtures][fixtures] directory.

## Execution Test Fixtures

Execution test fixtures live inside the `fixtures/execution/` directory.
Each JSON file in this directory contains the JSON-serialized
[`ExecutionFixture`][exec-fixture] object which is defined in Rust
in the [op-test-vectors][op-test-vectors] crate.

The `ExecutionFixture` holds everything needed to test execution of the OP Stack.
It's composed of the following.
- An `ExecutionEnvironment`, which is used to setup the execution client's environment.
- An initial set of addresses and their states, also called the "pre-state".
- A final set of addresses and their states, also called the "post-state".
- A list of transactions to execute in the environment.
- The result of executing all the transactions.

## Derivation Test Fixtures

// TODO

{{#include ../links.md}}
