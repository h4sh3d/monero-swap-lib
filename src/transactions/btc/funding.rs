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
use crate::types::constants::FEE_KB_SATOSHI;
use crate::types::btc::{Utxo, Input, scripts, PublicKey};
use crate::transactions::{Builder, Transaction};

use secp256k1::Signature;
use bitcoin::util::bip143::SighashComponents;
use bitcoin::blockdata::script::Script;
use bitcoin::consensus::encode::serialize_hex;

pub trait Funding: Transaction { }

pub struct FundingTx {
    pub(crate) tx_hex: Option<String>,
}

impl FundingTx {
    pub fn new() -> FundingTx {
        FundingTx { tx_hex: None }
    }

    pub fn from_hex(hex: String) -> FundingTx {
        FundingTx { tx_hex: Some(hex) }
    }
}

impl Transaction for FundingTx {
    fn to_hex(&self) -> Option<String> {
        self.tx_hex.clone()
    }
}

impl Funding for FundingTx { }

pub struct New<'a> {
    pub(crate) utxo: &'a Utxo,
    pub(crate) swaplock_script: &'a Script,
}

impl<'a> Builder<New<'a>> for FundingTx {
    type Ret = ();

    fn build(&mut self, params: New) -> Result<()> {
        let in_amount: u64 = params.utxo.amount;
        // TODO: estimate vsize of btx_1
        let fee = FEE_KB_SATOSHI;
        let out_amount = in_amount - fee;

        // TODO: verify all utxos are SegWit programs
        let btx_1 = bitcoin::Transaction {
            version: 2,
            lock_time: 0,
            input: vec![bitcoin::TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: params.utxo.txid,
                    vout: params.utxo.vout,
                },
                script_sig: bitcoin::Script::new(),
                sequence: std::u32::MAX,
                witness: vec![],
            }],
            output: vec![bitcoin::TxOut {
                value: out_amount,
                script_pubkey: params.swaplock_script.clone().to_v0_p2wsh(),
            }],
        };

        self.tx_hex = Some(serialize_hex(&btx_1));

        Ok(())
    }
}

pub struct Sign<'a> {
    pub(crate) pubkey: &'a PublicKey,
    pub(crate) input: &'a Input<'a>,
}

impl<'a> Builder<Sign<'a>> for FundingTx {
    type Ret = Signature;

    fn build(&mut self, params: Sign<'a>) -> Result<Signature> {
        let btx_1d = self.to_transaction()?;
        let bip143 = SighashComponents::new(&btx_1d);
        // Generate Segwit sighash for SIG_ALL
        let sig_hash = bip143.sighash_all(
            &btx_1d.input[0],
            &scripts::redeem_p2pkh(params.pubkey),
            params.input.amount,
        );
        let msg = secp256k1::Message::parse_slice(&sig_hash[..])?;

        let mut s = secp256k1::sign(&msg, params.input.privkey)?.0;
        s.normalize_s();
        Ok(s)
    }
}

pub struct Finalize {
    pub(crate) sig: Signature,
    pub(crate) pubkey: PublicKey,
}

impl Builder<Finalize> for FundingTx {
    type Ret = String;

    fn build(&mut self, params: Finalize) -> Result<String> {
        let mut btx_1d = self.to_transaction()?;

        btx_1d.input[0].witness = vec![
            scripts::serialize_sig(params.sig),
            params.pubkey.serialize_compressed().to_vec(),
        ];

        let tx = serialize_hex(&btx_1d);
        self.tx_hex = Some(tx.clone());
        Ok(tx)
    }
}
