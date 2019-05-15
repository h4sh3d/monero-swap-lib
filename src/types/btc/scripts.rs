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

use super::PublicKey;
use crate::types::xmr;
use secp256k1::Signature;
use bitcoin::blockdata::script::{Builder, Script};
use bitcoin::blockdata::opcodes::{all, OP_CSV};
use bitcoin_hashes::{Hash, hash160};

pub fn serialize_sig(sig: Signature) -> Vec<u8> {
    let mut sig = Vec::from(sig.serialize_der().as_ref());
    // Add SigHashType::All at DER serialized signature end
    sig.extend_from_slice(&[1]);
    sig
}

pub fn redeem_p2pkh(pk: &PublicKey) -> Script {
    Builder::new()
        .push_opcode(all::OP_DUP)
        .push_opcode(all::OP_HASH160)
        .push_slice(&hash160::Hash::hash(&pk.serialize_compressed()[..])[..])
        .push_opcode(all::OP_EQUALVERIFY)
        .push_opcode(all::OP_CHECKSIG)
        .into_script()
}

#[allow(non_snake_case)]
pub fn create_swaplock(B_a: &PublicKey, B_b: &PublicKey, h_0: &[u8], h_2: &[u8], t_0: u32) -> Script {
    Builder::new()
        .push_opcode(all::OP_IF)
        .push_opcode(all::OP_SHA256)
        .push_slice(h_0)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_opcode(all::OP_SHA256)
        .push_slice(h_2)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_slice(&B_a.serialize_compressed())
        .push_opcode(all::OP_CHECKSIG)
        .push_opcode(all::OP_ELSE)
        .push_int(t_0.into())
        .push_opcode(OP_CSV)
        .push_opcode(all::OP_DROP)
        .push_opcode(all::OP_PUSHNUM_2)
        .push_slice(&B_a.serialize_compressed())
        .push_slice(&B_b.serialize_compressed())
        .push_opcode(all::OP_PUSHNUM_2)
        .push_opcode(all::OP_CHECKMULTISIG)
        .push_opcode(all::OP_ENDIF)
        .into_script()
}

pub fn redeem_swaplock_multisig(swaplock: Script, sig_a: Signature, sig_b: Signature) -> Vec<Vec<u8>> {
    vec![
        vec![], // 0 for multisig
        serialize_sig(sig_a), // Sig_a
        serialize_sig(sig_b), // Sig_b
        vec![], // OP_FALSE
        swaplock.into_bytes(), // swaplock script
    ]
}

pub fn redeem_swaplock_buy(swaplock: Script, sig_a: Signature, share: &xmr::PrivateKey, s: &[u8]) -> Vec<Vec<u8>> {
    vec![
        serialize_sig(sig_a), // Sig_a
        Vec::from(s), // Secret s
        Vec::from(&share.to_bytes()[..]), // x_0 share
        vec![1], // OP_TRUE
        swaplock.into_bytes(), // swaplock script
    ]
}

#[allow(non_snake_case)]
pub fn create_refund(B_a: &PublicKey, B_b: &PublicKey, h_1: &[u8], t_1: u32) -> Script {
    Builder::new()
        .push_opcode(all::OP_IF)
        .push_opcode(all::OP_SHA256)
        .push_slice(h_1)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_slice(&B_b.serialize_compressed())
        .push_opcode(all::OP_CHECKSIG)
        .push_opcode(all::OP_ELSE)
        .push_int(t_1.into())
        .push_opcode(OP_CSV)
        .push_opcode(all::OP_DROP)
        .push_slice(&B_a.serialize_compressed())
        .push_opcode(all::OP_CHECKSIG)
        .push_opcode(all::OP_ENDIF)
        .into_script()
}

pub fn redeem_refund(refund: Script, sig: Signature, share: Option<&xmr::PrivateKey>) -> Vec<Vec<u8>> {
    let sig = serialize_sig(sig);
    match share {
        Some(x_1) => vec![
            sig, // Sig_b
            Vec::from(&x_1.to_bytes()[..]), // x_1 share
            vec![1], // OP_TRUE
            refund.into_bytes(), // refund script
        ],
        None => vec![
            sig, // Sig_a
            vec![], // OP_FALSE
            refund.into_bytes(), // refund script
        ],
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use secp256k1::{Signature, PublicKeyFormat};
    use bitcoin::Script;
    use crate::types::{xmr, btc};
    use super::{create_swaplock, redeem_swaplock_multisig, redeem_swaplock_buy, create_refund, redeem_refund};

    #[test]
    fn swaplock() {
        let B_a = btc::PublicKey::parse_slice(
            &hex::decode("02ea5b20f5e0ff2266a2670a5b96216c11f6760ef796d3ef5c846704c89bdd1099").unwrap(),
            Some(PublicKeyFormat::Compressed)
        ).unwrap();
        let B_b = btc::PublicKey::parse_slice(
            &hex::decode("03580314ac61e993d67dc247aa742a89568f1018efdaa1d29b848aa933563442a8").unwrap(),
            Some(PublicKeyFormat::Compressed)
        ).unwrap();
        let swaplock = create_swaplock(&B_a, &B_b, &[2; 32], &[4; 32], 144);
        assert_eq!(swaplock.as_bytes(), &[99u8, 168, 32, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        136, 168, 32, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 136, 33, 2, 234, 91, 32, 245,
        224, 255, 34, 102, 162, 103, 10, 91, 150, 33, 108, 17, 246, 118, 14,
        247, 150, 211, 239, 92, 132, 103, 4, 200, 155, 221, 16, 153, 172, 103,
        2, 144, 0, 178, 117, 82, 33, 2, 234, 91, 32, 245, 224, 255, 34, 102,
        162, 103, 10, 91, 150, 33, 108, 17, 246, 118, 14, 247, 150, 211, 239,
        92, 132, 103, 4, 200, 155, 221, 16, 153, 33, 3, 88, 3, 20, 172, 97, 233,
        147, 214, 125, 194, 71, 170, 116, 42, 137, 86, 143, 16, 24, 239, 218,
        161, 210, 155, 132, 138, 169, 51, 86, 52, 66, 168, 82, 174, 104][..]);
    }

    #[test]
    fn redeem_swaplock_with_multisig() {
        let swaplock_script = Script::from(vec![0u8; 140]);
        let sig = Signature::parse_der(&[48, 6, 2, 1, 1, 2, 1, 1]).unwrap();
        let redeem = redeem_swaplock_multisig(swaplock_script, sig.clone(), sig);
        assert_eq!(redeem, vec![
                   vec![], // OP_FALSE for multisig
                   vec![48, 6, 2, 1, 1, 2, 1, 1, 1], // Add SIGHASH ALL (0x01) after sig
                   vec![48, 6, 2, 1, 1, 2, 1, 1, 1], // Add SIGHASH ALL (0x01) after sig
                   vec![], // OP_FALSE for IF/ELSE
                   vec![0u8; 140],
        ]);
    }

    #[test]
    fn redeem_swaplock_with_buy() {
        let swaplock_script = Script::from(vec![0u8; 140]);
        let sig = Signature::parse_der(&[48, 6, 2, 1, 1, 2, 1, 1]).unwrap();
        let key: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0,
                             0, 0, 0, 0, 0, 0, 0, 0,
                             0, 0, 0, 0, 0, 0, 0, 0,
                             0, 0, 0, 0, 0, 0, 0, 1];
        let share = xmr::PrivateKey::from_bytes_mod_order(key);
        let redeem = redeem_swaplock_buy(swaplock_script, sig, &share, &[0; 32]);
        assert_eq!(redeem, vec![
                   vec![48, 6, 2, 1, 1, 2, 1, 1, 1], // Add SIGHASH ALL (0x01) after sig
                   vec![0; 32], // Secret
                   vec![0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 1], // Private key share
                   vec![1], // OP_TRUE for IF/ELSE
                   vec![0u8; 140], // Script
        ]);
    }

    #[test]
    fn refund() {
        let B_a = btc::PublicKey::parse_slice(
            &hex::decode("02ea5b20f5e0ff2266a2670a5b96216c11f6760ef796d3ef5c846704c89bdd1099").unwrap(),
            Some(PublicKeyFormat::Compressed)
        ).unwrap();
        let B_b = btc::PublicKey::parse_slice(
            &hex::decode("03580314ac61e993d67dc247aa742a89568f1018efdaa1d29b848aa933563442a8").unwrap(),
            Some(PublicKeyFormat::Compressed)
        ).unwrap();
        let refund = create_refund(&B_a, &B_b, &[0; 32], 144);
        assert_eq!(refund.as_bytes(), &[99u8, 168, 32, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        136, 33, 3, 88, 3, 20, 172, 97, 233, 147, 214, 125, 194, 71, 170, 116,
        42, 137, 86, 143, 16, 24, 239, 218, 161, 210, 155, 132, 138, 169, 51,
        86, 52, 66, 168, 172, 103, 2, 144, 0, 178, 117, 33, 2, 234, 91, 32, 245,
        224, 255, 34, 102, 162, 103, 10, 91, 150, 33, 108, 17, 246, 118, 14,
        247, 150, 211, 239, 92, 132, 103, 4, 200, 155, 221, 16, 153, 172,
        104][..]);
    }

    #[test]
    fn spend_refund() {
        let refund_script = Script::from(vec![0u8; 140]);
        let sig = Signature::parse_der(&[48, 6, 2, 1, 1, 2, 1, 1]).unwrap();
        let redeem = redeem_refund(refund_script, sig, None);
        assert_eq!(redeem, vec![
                   vec![48, 6, 2, 1, 1, 2, 1, 1, 1], // Add SIGHASH ALL (0x01) after sig
                   vec![], // OP_FALSE
                   vec![0u8; 140], // Refund script
        ]);
    }

    #[test]
    fn claim_refund() {
        let refund_script = Script::from(vec![0u8; 140]);
        let sig = Signature::parse_der(&[48, 6, 2, 1, 1, 2, 1, 1]).unwrap();
        let key: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0,
                             0, 0, 0, 0, 0, 0, 0, 0,
                             0, 0, 0, 0, 0, 0, 0, 0,
                             0, 0, 0, 0, 0, 0, 0, 1];
        let share = xmr::PrivateKey::from_bytes_mod_order(key);
        let redeem = redeem_refund(refund_script, sig, Some(&share));
        assert_eq!(redeem, vec![
                   vec![48, 6, 2, 1, 1, 2, 1, 1, 1], // Add SIGHASH ALL (0x01) after sig
                   vec![0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 1], // Pricate key share
                   vec![1], // OP_TRUE for IF/ELSE
                   vec![0u8; 140], // Refund script
        ]);
    }
}
