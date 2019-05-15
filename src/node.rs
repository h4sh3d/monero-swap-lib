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

//! Export buyer/seller nodes with wasm-bindgen links

use crate::{Protocol, Phase, Btc, Xmr};
use crate::types::{common, xmr, btc, RelativeLocktime};

use rand::rngs::OsRng;
use bitcoin_hashes::hex::FromHex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/// Seller node wants to swap Monero for Bitcoin
#[wasm_bindgen]
pub struct Seller {
    parameters: Option<xmr::SetupParams>,
    setup: Option<xmr::Setup>,
    setup_is_ready: bool,
}

#[wasm_bindgen]
impl Seller {
    /// Create an empty seller node with no setup
    pub fn new() -> Seller {
        Seller {
            parameters: None,
            setup: None,
            setup_is_ready: false,
        }
    }

    /// Generates the first parameters with randomness
    pub fn generate_params(&mut self, t0: u16, t1: u16) -> Result<(), JsValue> {
        let mut rng = OsRng::new().expect("OsRng");
        let params = common::Params::new(
            RelativeLocktime::Blocks(t0),
            RelativeLocktime::Blocks(t1),
        );
        self.parameters = Some(Xmr::setup(params.clone(), &mut rng)?);
        Ok(())
    }

    /// Export the setup to send it to the other node
    pub fn export_setup(&mut self) -> Result<xmr::ExportedSetupParams, JsValue> {
        match &self.parameters {
            Some(params) => Ok(xmr::ExportedSetupParams::from(params)),
            None => Err("Parameters is missing".into()),
        }
    }

    /// Verify the buyer's exported setup
    pub fn verify_setup(&mut self, buyer_params: &btc::ExportedSetupParams) -> Result<(), JsValue> {
        match &self.parameters {
            Some(params) => {
                let xmr_setup = Xmr::verify_setup(params, buyer_params)?;
                self.setup = Some(xmr_setup);
                self.setup_is_ready = true;
                Ok(())
            },
            None => Err("Parameters is missing".into()),
        }
    }

    /// Check if the setup is complete
    pub fn is_setup_ready(&self) -> bool {
        self.setup_is_ready
    }
}

/// Buyer node wants to swap Bitcoin for Monero
#[wasm_bindgen]
pub struct Buyer {
    parameters: Option<btc::SetupParams>,
    setup: Option<btc::Setup>,
    setup_is_ready: bool,
}

#[wasm_bindgen]
impl Buyer {
    /// Create a new node with no setup
    pub fn new() -> Buyer {
        Buyer {
            parameters: None,
            setup: None,
            setup_is_ready: false,
        }
    }

    /// Generates the first parameters with randomness
    pub fn generate_params(&mut self, t0: u16, t1: u16) -> Result<(), JsValue> {
        let mut rng = OsRng::new().expect("OsRng");
        let params = common::Params::new(
            RelativeLocktime::Blocks(t0),
            RelativeLocktime::Blocks(t1),
        );
        self.parameters = Some(Btc::setup(params.clone(), &mut rng)?);
        Ok(())
    }

    /// Export the setup to send it to the other node
    pub fn export_setup(&mut self) -> Result<btc::ExportedSetupParams, JsValue> {
        match &self.parameters {
            Some(params) => Ok(btc::ExportedSetupParams::from(params)),
            None => Err("Parameters is missing".into()),
        }
    }

    /// Verify the seller's exported setup
    pub fn verify_setup(&mut self, seller_params: &xmr::ExportedSetupParams) -> Result<(), JsValue> {
        match &self.parameters {
            Some(params) => {
                let btc_setup = Btc::verify_setup(params, seller_params)?;
                self.setup = Some(btc_setup);
                self.setup_is_ready = true;
                Ok(())
            },
            None => Err("Parameters is missing".into()),
        }
    }

    /// Check if the setup is complete
    pub fn is_setup_ready(&self) -> bool {
        self.setup_is_ready
    }

    /// Create the first Bitcoin transaction and return it as hex string
    pub fn create_transactions(&mut self, txid: &str, vout: u32, amount: u32) -> Result<String, JsValue> {
        // TODO: amount is u32 for testing, u64 use BigInt in JS, but only supported on Chrome
        // right now
        match &self.setup {
            Some(setup) => {
                let txid = bitcoin_hashes::sha256d::Hash::from_hex(txid)
                    .map_err(|e| format!("{:?}", e))?;
                let init_txs = Btc::execute(setup, &btc::CreateTransactions {
                    utxo: btc::Utxo {
                        txid,
                        vout,
                        amount: amount as u64,
                    },
                })?;
                Ok(init_txs.btx_1)
            },
            None => Err("Setup is missing".into()),
        }
    }
}
