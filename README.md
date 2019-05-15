[![Build Status](https://travis-ci.org/h4sh3d/monero-swap-lib.svg?branch=master)](https://travis-ci.org/h4sh3d/monero-swap-lib) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# XMR-BTC Atomic Swap Rust library

> This code has not been reviewed or audited and is not complete.

This library aims to implement all the heavy parts required in the draft proposal [Bitcoin & Monero Cross-chain Atomic Swap](https://github.com/h4sh3d/xmr-btc-atomic-swap).

Supports (or should support)

* create and validate setups paramters
* create Bitcoin transactions
* discover and validate Monero outputs

TODO

* zero-knowledge proofs in the setup phase
* monero integration with `monero-rs` libraries
* full validation for Bitcoin transactions

## Run tests

Tests need a bitcoin node in regtest mode (see README in `./tests`) only.

```
./run_tests.sh
```
