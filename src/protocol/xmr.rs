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

//! Implements the protocol and phases for the Monero side

use crate::types::{Result, Error};
use crate::types::{btc, xmr, common};
use crate::types::xmr::{VerifyTransactions, VerifiedTransaction, InitiateSwap, Swap, ClaimRefund};
use crate::types::btc::scripts::{create_swaplock, create_refund};
use crate::transactions as tx;
use crate::transactions::btc::funding::FundingTx;
use crate::transactions::btc::refund::RefundTx;
use crate::transactions::{Builder, Validator};
use super::{Protocol, Phase};

use rand::{Rng, CryptoRng};
use bitcoin_hashes::{Hash, sha256};
use curve25519_dalek::constants;

pub struct Xmr;

impl Phase<VerifyTransactions> for Xmr {
    type Ret = VerifiedTransaction;

    #[allow(non_snake_case)]
    fn execute(setup: &xmr::Setup, params: &VerifyTransactions) -> Result<VerifiedTransaction> {
        let swaplock_script = create_swaplock(&setup.B_a, &setup.B_b, &setup.h_0, &setup.h_2, setup.t_0.as_u32());

        let btx_1 = FundingTx::from_hex(params.transactions.btx_1.clone());
        let mut btx_2 = RefundTx::from_hex(params.transactions.btx_2.clone(), &btx_1);

        // TODO: Verify Btx1
        // TODO: verify all utxos are SegWit programs
        // TODO: Verify Btx2

        btx_2.validate(tx::btc::refund::VerifySigB {
            //secp: &secp,
            pubkey: &setup.B_b,
            sig: &params.transactions.sig_b,
            swaplock_script: &swaplock_script,
        })?;

        let sig_a = btx_2.build(tx::btc::refund::Sign {
            privkey: &setup.b_a,
            swaplock_script: &swaplock_script,
        })?;

        let sig_b = params.transactions.sig_b.clone();
        let btx_2_signed = btx_2.build(tx::btc::refund::Finalize {
            sig_a,
            sig_b,
            swaplock_script,
        })?;

        Ok(VerifiedTransaction {
            btx_2_signed,
        })
    }
}

impl Phase<InitiateSwap> for Xmr {
    type Ret = ();

    fn execute(_: &xmr::Setup, _: &InitiateSwap) -> Result<()> {
        // TODO: Create Xtx
        // TODO: Sign Xtx
        // TODO: Broadcast Xtx
        Ok(())
    }
}

impl Phase<Swap> for Xmr {
    type Ret = String;

    fn execute(setup: &xmr::Setup, params: &Swap) -> Result<String> {
        let swaplock_script = create_swaplock(&setup.B_a, &setup.B_b, &setup.h_0, &setup.h_2, setup.t_0.as_u32());

        let mut buy = tx::btc::buy::BuyTx::new();
        buy.build(tx::btc::common::New {
            prev_tx: &params.funding,
            final_address: params.address.clone(),
        })?;

        let sig = buy.build(tx::btc::common::Sign {
            privkey: &setup.b_a,
            script: &swaplock_script,
            prev_tx: &params.funding,
        })?;

        let buy_hex = buy.build(tx::btc::common::Finalize {
            sig,
            script: swaplock_script,
            privkey: Some(&setup.x_0),
            secret: Some(params.s),
        })?;

        Ok(buy_hex)
    }
}

impl<'a> Phase<ClaimRefund<'a>> for Xmr {
    type Ret = String;

    fn execute(setup: &xmr::Setup, params: &ClaimRefund) -> Result<String> {
        let refund_script = create_refund(&setup.B_a, &setup.B_b, &setup.h_1, setup.t_1.as_u32());

        let mut claim_refund = tx::btc::claim_refund::ClaimRefundTx::new();
        claim_refund.build(tx::btc::claim_refund::New {
            refund_tx: params.btx_2_signed,
            t_1: setup.t_1,
            final_address: params.address.clone(),
        })?;

        let sig = claim_refund.build(tx::btc::common::Sign {
            privkey: &setup.b_a,
            script: &refund_script,
            prev_tx: params.btx_2_signed,
        })?;

        let claim_refund_hex = claim_refund.build(tx::btc::common::Finalize {
            sig,
            privkey: None,
            script: refund_script,
            secret: None,
        })?;

        Ok(claim_refund_hex)
    }
}

impl Protocol for Xmr {
    type Output = xmr::SetupParams;
    type Input = btc::ExportedSetupParams;
    type Setup = xmr::Setup;

    #[allow(non_snake_case)]
    fn setup<R: Rng + CryptoRng>(params: common::Params, rng: &mut R) -> Result<xmr::SetupParams> {
        let common::Params {
            t_0,
            t_1,
        } = params;

        let a_0 = xmr::PrivateKey::random(rng);
        let x_0 = xmr::PrivateKey::random(rng);

        let mut bytes = [0u8; 32];
        rng.try_fill(&mut bytes)?;
        let b_a = btc::PrivateKey::parse(&bytes)?;
        let B_a = btc::PublicKey::from_secret_key(&b_a);

        let mut h_0 = [0u8; 32];
        let hash = sha256::Hash::hash(&x_0.to_bytes());
        h_0.copy_from_slice(&hash[..]);

        Ok(xmr::SetupParams {
            a_0,
            x_0,
            b_a,
            B_a,
            h_0,
            t_0,
            t_1,
        })
    }

    #[allow(non_snake_case)]
    fn verify_setup(params: &xmr::SetupParams, export: &btc::ExportedSetupParams) -> Result<xmr::Setup> {
        let xmr::SetupParams {
            a_0,
            x_0,
            b_a,
            B_a,
            h_0,
            t_0,
            t_1,
        } = params;

        let btc::ExportedSetupParams {
            a_1,
            X_1,
            B_b,
            h_1,
            h_2,
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
        let X_0 = x_0 * &constants::ED25519_BASEPOINT_TABLE;
        let X = X_0 + X_1;

        Ok(xmr::Setup {
            a,
            x_0: *x_0,
            X,
            b_a: b_a.clone(),
            B_a: B_a.clone(),
            B_b: B_b.clone(),
            h_0: *h_0,
            h_1: *h_1,
            h_2: *h_2,
            t_0: *t_0,
            t_1: *t_1,
        })
    }
}
