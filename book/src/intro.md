# Optimism Test Book

_Documentation for the ethereum-optimism/tests repository._

## Introduction

`ethereum-optimsim/tests` is a collection of test fixtures for testing OP Stack execution and derivation.
Alongside test fixtures, the `ethereum-optimism/tests` repository contains CLI tools for generating new
test fixtures written in Rust with the goal of making test fixture generation simple.

It is built and maintained by members of [OP Labs][op-labs] as well as open source contributors,
and is licensed under the MIT License.

`ethereum-optimsim/tests` is comparible to the [ethereum-tests][ethereum-tests] repository for ethereum.
The aim of [ethereum-optimism/tests][tests] is then to provide a set of standard tests for all
OP Stack client and node software to use for testing. In order to run these test fixtures against
various execution and derivation implementations, each instance must implement their own test runner.
For example, similar to how [revm][revm] defines a test runner, [revme][revme], to run the
[ethereum-tests][ethereum-tests] against its ethereum execution implementation.

In this book, we will break down the format of test fixtures, how to approach generating new test
fixtures, and how to implement custom runners. Much of this book is specific to Rust, but is
intentionally portable to other languages over the JSON interface.

## Development Status

**`ethereum-optimism/tests` is currently in active development, and is not yet ready for use in production.**

## Contributing

Contributors are welcome! Please see the [contributing guide][contributing] for more information.

{{#include ./links.md}}
