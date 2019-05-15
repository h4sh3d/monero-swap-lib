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

use crate::types::{Result, Error};
use crate::types::btc;
use crate::transactions::{Builder, Validator, Transaction};
use crate::types::constants::FEE_KB_SATOSHI;
use crate::types::RelativeLocktime;
use crate::transactions::btc::funding::FundingTx;
use crate::types::btc::scripts::redeem_swaplock_multisig;

use secp256k1::Signature;
use bitcoin::blockdata::script::Script;
use bitcoin::consensus::encode::serialize_hex;
use bitcoin::util::bip143::SighashComponents;

pub trait Refund: Transaction { }

pub struct RefundTx<'a> {
    pub(crate) tx_hex: Option<String>,
    pub(crate) btx_1: &'a FundingTx,
}

impl<'a> RefundTx<'a> {
    pub fn new(btx_1: &'a FundingTx) -> RefundTx<'a> {
        RefundTx { tx_hex: None, btx_1 }
    }

    pub fn from_hex(hex: String, btx_1: &'a FundingTx) -> RefundTx {
        RefundTx { tx_hex: Some(hex), btx_1 }
    }
}

impl<'a> Transaction for RefundTx<'a> {
    fn to_hex(&self) -> Option<String> {
        self.tx_hex.clone()
    }
}

impl<'a> Refund for RefundTx<'a> { }

pub struct New<'a> {
    pub(crate) refund_script: &'a Script,
    pub(crate) t_0: RelativeLocktime,
}

impl<'a, 'b> Builder<New<'a>> for RefundTx<'b> {
    type Ret = ();

    fn build(&mut self, params: New) -> Result<()> {
        let btx_1d = self.btx_1.to_transaction()?;
        let btx_2_in_amount = btx_1d.output[0].value;
        let btx_2_refund_amount = btx_2_in_amount - FEE_KB_SATOSHI / 2;

        let btx_2 = bitcoin::Transaction {
            version: 2,
            lock_time: 0,
            input: vec![bitcoin::TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: btx_1d.txid(),
                    vout: 0,
                },
                script_sig: bitcoin::Script::new(),
                sequence: params.t_0.as_u32(),
                witness: vec![],
            }],
            output: vec![bitcoin::TxOut {
                value: btx_2_refund_amount,
                script_pubkey: params.refund_script.clone().to_v0_p2wsh(),
            }],
        };

        self.tx_hex = Some(serialize_hex(&btx_2));
        Ok(())
    }
}

pub struct Sign<'a> {
    pub(crate) privkey: &'a btc::PrivateKey,
    pub(crate) swaplock_script: &'a Script,
}

impl<'a, 'b> Builder<Sign<'a>> for RefundTx<'b> {
    type Ret = Signature;

    fn build(&mut self, params: Sign) -> Result<Signature> {
        let btx_1d = self.btx_1.to_transaction()?;
        let btx_2d = self.to_transaction()?;

        let bip143_comp = SighashComponents::new(&btx_2d);
        // Generate Segwit sighash for SIG_ALL
        let sig_hash = bip143_comp.sighash_all(
            &btx_2d.input[0],
            &params.swaplock_script,
            btx_1d.output[0].value
        );
        let msg = secp256k1::Message::parse_slice(&sig_hash[..])?;

        let mut s = secp256k1::sign(&msg, params.privkey)?.0;
        s.normalize_s();
        Ok(s)
    }
}

pub struct Finalize {
    pub(crate) sig_a: Signature,
    pub(crate) sig_b: Signature,
    pub(crate) swaplock_script: Script,
}

impl<'a> Builder<Finalize> for RefundTx<'a> {
    type Ret = String;

    fn build(&mut self, params: Finalize) -> Result<String> {
        let mut btx_2d = self.to_transaction()?;
        btx_2d.input[0].witness = redeem_swaplock_multisig(
            params.swaplock_script,
            params.sig_a,
            params.sig_b,
        );
        let btx_2_signed = serialize_hex(&btx_2d);
        self.tx_hex = Some(btx_2_signed.clone());
        Ok(btx_2_signed)
    }
}

pub struct VerifySigA<'a> {
    pub(crate) pubkey: &'a btc::PublicKey,
    pub(crate) swaplock_script: &'a Script,
}

impl<'a, 'b> Validator<VerifySigA<'b>> for RefundTx<'a> {
    fn validate(&self, params: VerifySigA) -> Result<()> {
        let btx_1d = self.btx_1.to_transaction()?;
        let btx_2d = self.to_transaction()?;
        let sig_a = {
            let sig = &btx_2d.input[0].witness[1];
            Signature::parse_der(&sig[..sig.len()-1])?
        };

        let msg = {
            let bip143_comp = SighashComponents::new(&btx_2d);
            let sig_hash = bip143_comp.sighash_all(
                &btx_2d.input[0],
                params.swaplock_script,
                btx_1d.output[0].value
            );
            secp256k1::Message::parse_slice(&sig_hash[..])?
        };

        match secp256k1::verify(&msg, &sig_a, params.pubkey) {
            true => Ok(()),
            false => Err(Error::InvalidSignature),
        }
    }
}

pub struct VerifySigB<'a> {
    pub(crate) pubkey: &'a btc::PublicKey,
    pub(crate) sig: &'a Signature,
    pub(crate) swaplock_script: &'a Script,
}

impl<'a, 'b> Validator<VerifySigB<'b>> for RefundTx<'a> {
    fn validate(&self, params: VerifySigB) -> Result<()> {
        let btx_1d = self.btx_1.to_transaction()?;
        let btx_2d = self.to_transaction()?;

        let bip143_comp = SighashComponents::new(&btx_2d);
        let sig_hash = bip143_comp.sighash_all(
            &btx_2d.input[0],
            params.swaplock_script,
            btx_1d.output[0].value
        );
        let msg = secp256k1::Message::parse_slice(&sig_hash[..])?;

        match secp256k1::verify(&msg, params.sig, params.pubkey) {
            true => Ok(()),
            false => Err(Error::InvalidSignature),
        }
    }
}
