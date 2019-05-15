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

use crate::types::Result;
use crate::transactions::{Builder, Transaction};
use crate::transactions::btc::refund::RefundTx;
use crate::transactions::btc::common::{Sign, Finalize};
use crate::transactions::btc::refund::Refund;
use crate::types::btc::scripts::redeem_refund;
use crate::types::constants::FEE_KB_SATOSHI;
use crate::types::{RelativeLocktime};

use secp256k1::Signature;
use bitcoin::util::address::Address;
use bitcoin::util::bip143::SighashComponents;
use bitcoin::consensus::encode::serialize_hex;

#[derive(Debug)]
pub struct ClaimRefundTx {
    pub(crate) tx_hex: Option<String>,
}

impl ClaimRefundTx {
    pub fn new() -> ClaimRefundTx {
        ClaimRefundTx { tx_hex: None }
    }

    pub fn from_hex(hex: String) -> ClaimRefundTx {
        ClaimRefundTx { tx_hex: Some(hex) }
    }
}

impl Transaction for ClaimRefundTx {
    fn to_hex(&self) -> Option<String> {
        self.tx_hex.clone()
    }
}

pub struct New<'a> {
    pub(crate) refund_tx: &'a RefundTx<'a>,
    pub(crate) t_1: RelativeLocktime,
    pub(crate) final_address: Address,
}

impl<'a> Builder<New<'a>> for ClaimRefundTx {
    type Ret = ();

    fn build(&mut self, params: New) -> Result<()> {
        let refund = params.refund_tx.to_transaction()?;
        let out_amount = refund.output[0].value - FEE_KB_SATOSHI / 2;
        let claim_refund = bitcoin::Transaction {
            version: 2,
            lock_time: 0,
            input: vec![bitcoin::TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: refund.txid(),
                    vout: 0,
                },
                script_sig: bitcoin::Script::new(),
                sequence: params.t_1.as_u32(),
                witness: vec![],
            }],
            output: vec![bitcoin::TxOut {
                value: out_amount,
                script_pubkey: params.final_address.script_pubkey(),
            }],
        };

        self.tx_hex = Some(serialize_hex(&claim_refund));
        Ok(())
    }
}

impl<'a, R> Builder<Sign<'a, R>> for ClaimRefundTx where R: Refund {
    type Ret = Signature;

    fn build(&mut self, params: Sign<R>) -> Result<Signature> {
        let spend_refund = self.to_transaction()?;

        let bip143_comp = SighashComponents::new(&spend_refund);
        // Generate Segwit sighash for SIG_ALL
        let sig_hash = bip143_comp.sighash_all(
            &spend_refund.input[0],
            &params.script,
            params.prev_tx.to_transaction()?.output[0].value
        );
        let msg = secp256k1::Message::parse_slice(&sig_hash[..])?;

        let mut s = secp256k1::sign(&msg, params.privkey)?.0;
        s.normalize_s();
        Ok(s)
    }
}

impl<'a> Builder<Finalize<'a>> for ClaimRefundTx {
    type Ret = String;

    fn build(&mut self, params: Finalize<'a>) -> Result<String> {
        let mut spend_refund = self.to_transaction()?;
        spend_refund.input[0].witness = redeem_refund(
            params.script,
            params.sig,
            None,
        );
        let spend_refund = serialize_hex(&spend_refund);
        self.tx_hex = Some(spend_refund.clone());
        Ok(spend_refund)
    }
}
