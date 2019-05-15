Integration tests
===

Integration tests use a Bitcoin node in regtest mode and a Monero node.

## Bitcoin node config

RPC client try to connect to `127.0.0.1` on port `18443` with the login `test`.

```rust
let client = Client::new(
    "http://127.0.0.1:18443".into(),
    Some("test".into()),
    Some("cEl2o3tHHgzYeuu3CiiZ2FjdgSiw9wNeMFzoNbFmx9k=".into()),
);

```

Use this minimal config in `.bitcoin/bitcoin.conf` to have integration tests working:

> THIS SET THE PASSWORD AND OPEN THE NODE! CHANGE THE CONFIG AFTER RUNNING THE TESTS OR USE IT IN A SAFE ENVIRONMENT

```
regtest=1

# Accept command line and JSON-RPC commands.
server=1

[regtest]
rpcbind=0.0.0.0:18443
rpcallowip=0.0.0.0/0

rpcauth=test:dc2230611534670b0b2e2358b1884472$3d19ac78f03e9f3a940448503efce8cebfb3adb69bf2127708ce2b427f6d1262
# Your password:
# cEl2o3tHHgzYeuu3CiiZ2FjdgSiw9wNeMFzoNbFmx9k=
```

## Monero node config

Launch `monerod` and `monero-wallet-rpc` with:

```
monerod --regtest --offline [--fixed-difficulty 1]
```

and

```
monero-wallet-rpc --disable-rpc-login --rpc-bind-port 18083 --wallet-dir /tmp/
```
