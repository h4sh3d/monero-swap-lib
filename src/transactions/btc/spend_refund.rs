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
use crate::types::constants::FEE_KB_SATOSHI;
use crate::types::btc::scripts::redeem_refund;
use crate::transactions::btc::refund::Refund;
use crate::transactions::btc::common::{New, Sign, Finalize};

use secp256k1::Signature;
use bitcoin::consensus::encode::serialize_hex;
use bitcoin::util::bip143::SighashComponents;

#[derive(Debug)]
pub struct SpendRefundTx {
    pub(crate) tx_hex: Option<String>,
}

impl SpendRefundTx {
    pub fn new() -> SpendRefundTx {
        SpendRefundTx { tx_hex: None }
    }

    pub fn from_hex(hex: String) -> SpendRefundTx {
        SpendRefundTx { tx_hex: Some(hex) }
    }
}

impl Transaction for SpendRefundTx {
    fn to_hex(&self) -> Option<String> {
        self.tx_hex.clone()
    }
}

impl<'a, T> Builder<New<'a, T>> for SpendRefundTx where T: Refund {
    type Ret = ();

    fn build(&mut self, params: New<T>) -> Result<()> {
        let refund = params.prev_tx.to_transaction()?;
        let out_amount = refund.output[0].value - FEE_KB_SATOSHI / 2;
        let spend_refund = bitcoin::Transaction {
            version: 2,
            lock_time: 0,
            input: vec![bitcoin::TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: refund.txid(),
                    vout: 0,
                },
                script_sig: bitcoin::Script::new(),
                sequence: std::u32::MAX,
                witness: vec![],
            }],
            output: vec![bitcoin::TxOut {
                value: out_amount,
                script_pubkey: params.final_address.script_pubkey(),
            }],
        };

        self.tx_hex = Some(serialize_hex(&spend_refund));
        Ok(())
    }
}

impl<'a, R> Builder<Sign<'a, R>> for SpendRefundTx where R: Refund {
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

impl<'a> Builder<Finalize<'a>> for SpendRefundTx {
    type Ret = String;

    fn build(&mut self, params: Finalize<'a>) -> Result<String> {
        let mut spend_refund = self.to_transaction()?;
        spend_refund.input[0].witness = redeem_refund(
            params.script,
            params.sig,
            params.privkey,
        );
        let spend_refund = serialize_hex(&spend_refund);
        self.tx_hex = Some(spend_refund.clone());
        Ok(spend_refund)
    }
}
