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

//! Implements the protocol and phases for the Bitcoin side

use crate::types::{Result, Error};
use crate::types::{btc, xmr, common};
use crate::types::btc::{CreateTransactions, InitialTransactions, LockFunds, VerifyXmrLock, ReleaseXmr, SpendRefund};
use crate::types::btc::scripts::{create_swaplock, create_refund};
use crate::transactions as tx;
use crate::transactions::Transaction;
use crate::transactions::btc::funding::FundingTx;
use crate::transactions::btc::refund::RefundTx;
use crate::transactions::{Builder, Validator};
use super::{Protocol, Phase};

use rand::{Rng, CryptoRng};
use bitcoin_hashes::{Hash, sha256};
use curve25519_dalek::constants;

pub struct Btc;

impl Phase<CreateTransactions> for Btc {
    type Ret = InitialTransactions;

    #[allow(non_snake_case)]
    fn execute(setup: &btc::Setup, params: &CreateTransactions) -> Result<InitialTransactions> {
        let swaplock_script = create_swaplock(&setup.B_a, &setup.B_b, &setup.h_0, &setup.h_2, setup.t_0.as_u32());
        let refund_script = create_refund(&setup.B_a, &setup.B_b, &setup.h_1, setup.t_0.as_u32());

        let mut btx_1 = FundingTx::new();
        btx_1.build(tx::btc::funding::New {
            utxo: &params.utxo,
            swaplock_script: &swaplock_script,
        })?;

        let mut btx_2 = RefundTx::new(&btx_1);
        btx_2.build(tx::btc::refund::New {
            refund_script: &refund_script,
            t_0: setup.t_0,
        })?;

        let sig_b = btx_2.build(tx::btc::refund::Sign {
            privkey: &setup.b_b,
            swaplock_script: &swaplock_script,
        })?;

        Ok(InitialTransactions {
            btx_1: btx_1.to_hex()?,
            btx_2: btx_2.to_hex()?,
            sig_b,
        })
    }
}

impl<'a> Phase<LockFunds<'a>> for Btc {
    type Ret = String;

    #[allow(non_snake_case)]
    fn execute(setup: &btc::Setup, params: &LockFunds) -> Result<String> {
        let swaplock_script = create_swaplock(&setup.B_a, &setup.B_b, &setup.h_0, &setup.h_2, setup.t_0.as_u32());
        let pubkey = btc::PublicKey::from_secret_key(&params.input.privkey);

        let mut btx_1 = FundingTx::from_hex(params.btx_1.clone());
        let btx_2 = RefundTx::from_hex(params.btx_2_signed.clone(), &btx_1);

        btx_2.validate(tx::btc::refund::VerifySigA {
            pubkey: &setup.B_a,
            swaplock_script: &swaplock_script,
        })?;

        let sig = btx_1.build(tx::btc::funding::Sign {
            pubkey: &pubkey,
            input: &params.input,
        })?;

        Ok(btx_1.build(tx::btc::funding::Finalize {
            sig,
            pubkey,
        })?)
    }
}

impl<'a> Phase<SpendRefund<'a>> for Btc {
    type Ret = String;

    fn execute(setup: &btc::Setup, params: &SpendRefund) -> Result<String> {
        let refund_script = create_refund(&setup.B_a, &setup.B_b, &setup.h_1, setup.t_1.as_u32());

        let mut spend_refund = tx::btc::spend_refund::SpendRefundTx::new();
        spend_refund.build(tx::btc::common::New {
            prev_tx: params.btx_2_signed,
            final_address: params.address.clone(),
        })?;

        let sig = spend_refund.build(tx::btc::common::Sign {
            privkey: &setup.b_b,
            script: &refund_script,
            prev_tx: params.btx_2_signed,
        })?;

        let spend_refund_hex = spend_refund.build(tx::btc::common::Finalize {
            sig,
            privkey: Some(&setup.x_1),
            script: refund_script,
            secret: None,
        })?;

        Ok(spend_refund_hex)
    }
}

impl Phase<VerifyXmrLock> for Btc {
    type Ret = [u8; 32];

    fn execute(_: &btc::Setup, _: &VerifyXmrLock) -> Result<[u8; 32]> {
        // TODO: Verify Xtx w/ A, X
        // TODO: Send s
        Ok([0u8; 32])
    }
}

impl Phase<ReleaseXmr> for Btc {
    type Ret = ();

    fn execute(_: &btc::Setup, _: &ReleaseXmr) -> Result<()> {
        // TODO: Compute x
        // TODO: Spend Xtx
        Ok(())
    }
}

impl Protocol for Btc {
    type Output = btc::SetupParams;
    type Input = xmr::ExportedSetupParams;
    type Setup = btc::Setup;

    #[allow(non_snake_case)]
    fn setup<R: Rng + CryptoRng>(params: common::Params, rng: &mut R) -> Result<btc::SetupParams> {
        let common::Params {
            t_0,
            t_1,
        } = params;

        let a_1 = xmr::PrivateKey::random(rng);
        let x_1 = xmr::PrivateKey::random(rng);

        let mut bytes = [0u8; 32];
        rng.try_fill(&mut bytes)?;
        let b_b = btc::PrivateKey::parse(&bytes)?;
        let B_b = btc::PublicKey::from_secret_key(&b_b);

        let mut s = [0u8; 32];
        rng.try_fill(&mut s[..])?;

        let mut h_1 = [0u8; 32];
        let hash = sha256::Hash::hash(&x_1.to_bytes());
        h_1.copy_from_slice(&hash[..]);

        let mut h_2 = [0u8; 32];
        let hash = sha256::Hash::hash(&s);
        h_2.copy_from_slice(&hash[..]);

        Ok(btc::SetupParams {
            a_1,
            x_1,
            b_b,
            B_b,
            s,
            h_1,
            h_2,
            t_0,
            t_1,
        })
    }

    #[allow(non_snake_case)]
    fn verify_setup(params: &btc::SetupParams, export: &xmr::ExportedSetupParams) -> Result<btc::Setup> {
        let btc::SetupParams {
            a_1,
            x_1,
            b_b,
            B_b,
            s,
            h_1,
            h_2,
            t_0,
            t_1,
        } = params;

        let xmr::ExportedSetupParams {
            a_0,
            X_0,
            B_a,
            h_0,
            ..
        } = export;

        match t_0 == &export.t_0 {
            false => return Err(Error::MissmatchCommonParameters),
            true => (),
        };

        match t_1 == &export.t_1 {
            false => return Err(Error::MissmatchCommonParameters),
            true => (),
        };

        let a = a_0 + a_1;
        let X_1 = x_1 * &constants::ED25519_BASEPOINT_TABLE;
        let X = X_0 + X_1;

        Ok(btc::Setup {
            a,
            x_1: *x_1,
            X,
            B_a: B_a.clone(),
            b_b: b_b.clone(),
            B_b: B_b.clone(),
            s: *s,
            h_0: *h_0,
            h_1: *h_1,
            h_2: *h_2,
            t_0: *t_0,
            t_1: *t_1,
        })
    }
}
