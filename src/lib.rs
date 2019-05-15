// Monero Swap Rust Library
// Written in 2019 by
//   h4sh3d <h4sh3d@truelevel.io>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//

#![feature(try_trait)]

extern crate rand;
extern crate hex;
extern crate curve25519_dalek;
extern crate secp256k1;
extern crate bitcoin;
extern crate bitcoin_hashes;
extern crate wasm_bindgen;

pub mod node;
pub mod types;
pub mod transactions;
pub mod protocol;

pub use crate::protocol::{Protocol, Phase};
pub use crate::protocol::btc::Btc;
pub use crate::protocol::xmr::Xmr;
pub use crate::node::Buyer;
pub use crate::node::Seller;
