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
use crate::types::constants::FEE_KB_SATOSHI;
use crate::types::btc::scripts::redeem_swaplock_buy;
use crate::transactions::{Builder, Transaction};
use crate::transactions::btc::funding::Funding;
use crate::transactions::btc::common::{New, Sign, Finalize};

use secp256k1::Signature;
use bitcoin::consensus::encode::serialize_hex;
use bitcoin::util::bip143::SighashComponents;

#[derive(Debug)]
pub struct BuyTx {
    pub(crate) tx_hex: Option<String>,
}

impl BuyTx {
    pub fn new() -> BuyTx {
        BuyTx { tx_hex: None }
    }

    pub fn from_hex(hex: String) -> BuyTx {
        BuyTx { tx_hex: Some(hex) }
    }
}

impl Transaction for BuyTx {
    fn to_hex(&self) -> Option<String> {
        self.tx_hex.clone()
    }
}

impl<'a, T> Builder<New<'a, T>> for BuyTx where T: Funding {
    type Ret = ();

    fn build(&mut self, params: New<T>) -> Result<()> {
        let funding = params.prev_tx.to_transaction()?;
        let out_amount = funding.output[0].value - FEE_KB_SATOSHI / 2;
        let buy_tx = bitcoin::Transaction {
            version: 2,
            lock_time: 0,
            input: vec![bitcoin::TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: funding.txid(),
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


        self.tx_hex = Some(serialize_hex(&buy_tx));
        Ok(())
    }
}

impl<'a, F> Builder<Sign<'a, F>> for BuyTx where F: Funding {
    type Ret = Signature;

    fn build(&mut self, params: Sign<F>) -> Result<Signature> {
        let buy = self.to_transaction()?;

        let bip143_comp = SighashComponents::new(&buy);
        let sig_hash = bip143_comp.sighash_all(
            &buy.input[0],
            &params.script,
            params.prev_tx.to_transaction()?.output[0].value
        );
        let msg = secp256k1::Message::parse_slice(&sig_hash[..])?;

        let mut s = secp256k1::sign(&msg, params.privkey)?.0;
        s.normalize_s();
        Ok(s)
    }
}

impl<'a> Builder<Finalize<'a>> for BuyTx {
    type Ret = String;

    fn build(&mut self, params: Finalize<'a>) -> Result<String> {
        let mut buy = self.to_transaction()?;
        buy.input[0].witness = redeem_swaplock_buy(
            params.script,
            params.sig,
            params.privkey.ok_or(Error::MissingValue)?,
            &params.secret.ok_or(Error::MissingValue)?,
        );
        let buy = serialize_hex(&buy);
        self.tx_hex = Some(buy.clone());
        Ok(buy)
    }
}
