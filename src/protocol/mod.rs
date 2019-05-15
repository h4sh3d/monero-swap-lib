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

//! Implements the steps for the Bitcoin and Monero side to complete the trade as
//! describe in the draft protocol

use crate::types::Result;
use crate::types::common;

use rand::{Rng, CryptoRng};

/// A protocol define one side of the trade. One protocol is define in Bitcoin and
/// one in Monero
pub trait Protocol {
    type Input;

    /// Output of setup and input of verify_setup
    type Output;

    /// Output of verify_setup
    type Setup;

    /// Initiate a setup with common parameters across nodes
    fn setup<R: Rng + CryptoRng>(params: common::Params, rng: &mut R) -> Result<Self::Output>;
    /// Verify setups
    fn verify_setup(params: &Self::Output, export: &Self::Input) -> Result<Self::Setup>;
}

/// A phase define an action to execute in a protocol
pub trait Phase<T>: Protocol {
    /// Returning value of the current phase
    type Ret;

    /// Execute a phase in a protocl with a given setup and return a result
    fn execute(setup: &Self::Setup, params: &T) -> Result<Self::Ret>;
}

pub mod btc;
pub mod xmr;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::{Protocol, btc::Btc, xmr::Xmr};
    use rand::rngs::OsRng;
    use crate::types::{xmr, btc, common, RelativeLocktime};
    use curve25519_dalek::constants;

    fn setup() -> (OsRng, common::Params) {
        let rng = OsRng::new().expect("OsRng");
        let params = common::Params {
            t_0: RelativeLocktime::Blocks(144),
            t_1: RelativeLocktime::Blocks(144),
        };
        (rng, params)
    }

    #[test]
    fn btc_setup() {
        let (mut rng, params) = setup();
        Btc::setup(params, &mut rng).is_ok();
    }

    #[test]
    fn xmr_setup() {
        let (mut rng, params) = setup();
        Xmr::setup(params, &mut rng).is_ok();
    }

    #[test]
    fn xmr_export_setup_params() {
        let (mut rng, params) = setup();
        let params = Xmr::setup(params, &mut rng).unwrap();
        let _: xmr::ExportedSetupParams = (&params).into();
    }

    #[test]
    fn btc_export_setup_params() {
        let (mut rng, params) = setup();
        let params = Btc::setup(params, &mut rng).unwrap();
        let _: btc::ExportedSetupParams = (&params).into();
    }

    #[test]
    fn xmr_verify_setup() {
        let (mut rng, params) = setup();
        let xmr_params = Xmr::setup(params.clone(), &mut rng).unwrap();
        let btc_params = Btc::setup(params, &mut rng).unwrap();
        let x = &xmr_params.x_0 + &btc_params.x_1;
        let X_0 = &xmr_params.x_0 * &constants::ED25519_BASEPOINT_TABLE;
        let X_1 = &btc_params.x_1 * &constants::ED25519_BASEPOINT_TABLE;
        let X = &X_0 + &X_1;
        let X2 = &x * &constants::ED25519_BASEPOINT_TABLE;
        assert!(X == X2);
        let exported: btc::ExportedSetupParams = (&btc_params).into();
        let setup = Xmr::verify_setup(&xmr_params, &exported).unwrap();
        assert!(setup.X == X);
    }

    #[test]
    fn btc_verify_setup() {
        let (mut rng, params) = setup();
        let xmr_params = Xmr::setup(params.clone(), &mut rng).unwrap();
        let btc_params = Btc::setup(params, &mut rng).unwrap();
        let x = &xmr_params.x_0 + &btc_params.x_1;
        let X_0 = &xmr_params.x_0 * &constants::ED25519_BASEPOINT_TABLE;
        let X_1 = &btc_params.x_1 * &constants::ED25519_BASEPOINT_TABLE;
        let X = &X_0 + &X_1;
        let X2 = &x * &constants::ED25519_BASEPOINT_TABLE;
        assert!(X == X2);
        let exported: xmr::ExportedSetupParams = (&xmr_params).into();
        let setup = Btc::verify_setup(&btc_params, &exported).unwrap();
        assert!(setup.X == X);
    }
}
