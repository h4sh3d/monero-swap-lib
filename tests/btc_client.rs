extern crate monero_swap_lib as lib;
extern crate curve25519_dalek;
extern crate bitcoincore_rpc;
extern crate rand;
extern crate hex;
extern crate bitcoin_bech32;

use rand::rngs::OsRng;
use bitcoincore_rpc::Client;
use bitcoin::util::address::Address;
use bitcoin::util::base58;
use bitcoin::network::constants::Network;
use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::json::AddressType;

use lib::{Protocol, Phase, Btc, Xmr};
use lib::types::{common, xmr, btc, RelativeLocktime};
use lib::transactions::btc::{refund, funding};

const T_0: u16 = 10;
const T_1: u16 = 10;

fn setup() -> (xmr::Setup, btc::Setup, Client, String) {
    let mut rng = OsRng::new().expect("OsRng");
    let params = common::Params::new(
        RelativeLocktime::Blocks(T_0),
        RelativeLocktime::Blocks(T_1),
    );
    let xmr_params = Xmr::setup(params.clone(), &mut rng).unwrap();
    let btc_params = Btc::setup(params, &mut rng).unwrap();
    let btc_exported = btc::ExportedSetupParams::from(&btc_params);
    let xmr_exported = xmr::ExportedSetupParams::from(&xmr_params);
    let btc_setup = Btc::verify_setup(&btc_params, &xmr_exported).unwrap();
    let xmr_setup = Xmr::verify_setup(&xmr_params, &btc_exported).unwrap();

    let client = Client::new(
        "http://127.0.0.1:18443".into(),
        Some("test".into()),
        Some("cEl2o3tHHgzYeuu3CiiZ2FjdgSiw9wNeMFzoNbFmx9k=".into()),
    );
    let address = client.get_new_address(None, Some(AddressType::Bech32)).unwrap();
    (xmr_setup, btc_setup, client, address)
}

fn verified_txs(params: &(xmr::Setup, btc::Setup, Client, String)) -> (btc::InitialTransactions, xmr::VerifiedTransaction, Address, btc::PrivateKey) {
    let (xmr_setup, btc_setup, client, address) = params;

    let _ = client.send_to_address(&address, 1.0f64, None, None, None);
    let witness_program = bitcoin_bech32::WitnessProgram::from_address(&address).unwrap();
    let address = Address {
        payload: bitcoin::util::address::Payload::WitnessProgram(witness_program),
        network: Network::Regtest,
    };

    let mut utxos: Vec<lib::types::btc::Utxo> = client.list_unspent(
            None,
            None,
            Some(vec![&address]),
            None,
            None
        ).unwrap()
        .iter()
        .map(|u| lib::types::btc::Utxo {
            txid: u.txid,
            vout: u.vout,
            amount: u.amount.into_inner() as u64,
        })
        .collect();

    let init_txs = Btc::execute(&btc_setup, &btc::CreateTransactions {
        utxo: utxos.remove(0),
    }).unwrap();

    let verify_txs = xmr::VerifyTransactions {
        transactions: init_txs.clone(),
    };
    let verified_txs = Xmr::execute(&xmr_setup, &verify_txs).unwrap();
    let privkey_str = client.dump_priv_key(&address).unwrap();
    let bytes = base58::from_check(&privkey_str).unwrap();
    let privkey = btc::PrivateKey::parse_slice(&bytes[1..33]).unwrap();
    (init_txs, verified_txs, address, privkey)
}

fn setup_btx1() -> (Client, btc::Setup, xmr::Setup, String, String) {
    let setup  = setup();
    let (init_txs, verified_txs, address, privkey) = verified_txs(&setup);
    let (xmr_setup, btc_setup, client, _) = setup;

    let lock_funds = btc::LockFunds {
        input: lib::types::btc::Input {
            amount: 100_000_000,
            address: &address,
            privkey: &privkey,
        },
        btx_1: init_txs.btx_1.clone(),
        btx_2_signed: verified_txs.btx_2_signed,
    };
    let tx = Btc::execute(&btc_setup, &lock_funds).unwrap();
    client.send_raw_transaction(&tx).unwrap();

    let btx_1 = init_txs.btx_1.clone();
    let btx_2 = lock_funds.btx_2_signed.clone();
    (client, btc_setup, xmr_setup, btx_1, btx_2)
}

#[test]
fn broadcast_btx1() {
    let _ = setup_btx1();
}

#[test]
fn broadcast_btx2_before_locktime() {
    let (client, _, _, _, btx_2_signed) = setup_btx1();

    // Generate less than Timelock 0 blocks
    let _ = client.generate(T_0 as u64 - 1, None);

    // Should throw an Error non-BIP68 final
    assert_eq!(true, client.send_raw_transaction(&btx_2_signed).is_err());
}

#[test]
fn broadcast_btx2_after_locktime() {
    let (client, _, _, _, btx_2_signed) = setup_btx1();

    // Generate equal to Timelock 0 blocks
    let _ = client.generate(T_0 as u64, None);

    // Should not throw an error
    assert_eq!(false, client.send_raw_transaction(&btx_2_signed).is_err());
}

#[test]
fn broadcast_btx2_before_locktime_xmr() {
    let (client, _, _, _, btx_2_signed) = setup_btx1();

    // Generate less than Timelock 0 blocks
    let _ = client.generate(T_0 as u64 - 1, None);

    // Should throw an Error non-BIP68 final
    assert_eq!(true, client.send_raw_transaction(&btx_2_signed).is_err());
}

#[test]
fn broadcast_btx2_after_locktime_xmr() {
    let (client, _, _, _, btx_2_signed) = setup_btx1();

    // Generate equal to Timelock 0 blocks
    let _ = client.generate(T_0 as u64, None);

    // Should not throw an error
    assert_eq!(false, client.send_raw_transaction(&btx_2_signed).is_err());
}

fn lock_funds_and_start_refund() -> (xmr::Setup, btc::Setup, Client, String, String) {
    let (client, btc_setup, xmr_setup, btx_1, btx_2) = setup_btx1();

    // Generate equal to Timelock 0 blocks
    let _ = client.generate(T_0 as u64, None);

    // Start refund process
    client.send_raw_transaction(&btx_2).unwrap();

    (xmr_setup, btc_setup, client, btx_1, btx_2)
}

#[test]
fn spend_refund() {
    let (_, btc_setup, client, btx_1, btx_2) = lock_funds_and_start_refund();

    let btx_1 = funding::FundingTx::from_hex(btx_1);
    let btx_2 = refund::RefundTx::from_hex(btx_2, &btx_1);

    let final_address = client.get_new_address(None, Some(AddressType::Bech32)).unwrap();
    let witness_program = bitcoin_bech32::WitnessProgram::from_address(&final_address).unwrap();
    let final_address = Address {
        payload: bitcoin::util::address::Payload::WitnessProgram(witness_program),
        network: Network::Regtest,
    };

    let spend_refund = btc::SpendRefund {
        btx_2_signed: &btx_2,
        address: final_address,
    };
    let tx = Btc::execute(&btc_setup, &spend_refund).unwrap();
    assert_eq!(false, client.send_raw_transaction(&tx).is_err());
}

#[test]
fn claim_refund_before_locktime() {
    let (xmr_setup, _, client, btx_1, btx_2) = lock_funds_and_start_refund();

    // Generate less than Timelock 1 blocks
    let _ = client.generate(T_1 as u64 - 1, None);

    let btx_1 = funding::FundingTx::from_hex(btx_1);
    let btx_2 = refund::RefundTx::from_hex(btx_2, &btx_1);

    let final_address = client.get_new_address(None, Some(AddressType::Bech32)).unwrap();
    let witness_program = bitcoin_bech32::WitnessProgram::from_address(&final_address).unwrap();
    let final_address = Address {
        payload: bitcoin::util::address::Payload::WitnessProgram(witness_program),
        network: Network::Regtest,
    };

    let claim_refund = xmr::ClaimRefund {
        btx_2_signed: &btx_2,
        address: final_address,
    };
    let tx = Xmr::execute(&xmr_setup, &claim_refund).unwrap();
    assert_eq!(true, client.send_raw_transaction(&tx).is_err());
}

#[test]
fn claim_refund_after_locktime() {
    let (xmr_setup, _, client, btx_1, btx_2) = lock_funds_and_start_refund();

    // Generate equal to Timelock 0 blocks
    let _ = client.generate(T_1 as u64, None);

    let btx_1 = funding::FundingTx::from_hex(btx_1);
    let btx_2 = refund::RefundTx::from_hex(btx_2, &btx_1);

    let final_address = client.get_new_address(None, Some(AddressType::Bech32)).unwrap();
    let witness_program = bitcoin_bech32::WitnessProgram::from_address(&final_address).unwrap();
    let final_address = Address {
        payload: bitcoin::util::address::Payload::WitnessProgram(witness_program),
        network: Network::Regtest,
    };

    let claim_refund = xmr::ClaimRefund {
        btx_2_signed: &btx_2,
        address: final_address,
    };
    let tx = Xmr::execute(&xmr_setup, &claim_refund).unwrap();
    assert_eq!(false, client.send_raw_transaction(&tx).is_err());
}

#[test]
fn buy_bitcoin() {
    let (client, btc_setup, xmr_setup, btx_1, _) = setup_btx1();
    let btx_1 = funding::FundingTx::from_hex(btx_1);

    let address = client.get_new_address(None, Some(AddressType::Bech32)).unwrap();
    let witness_program = bitcoin_bech32::WitnessProgram::from_address(&address).unwrap();
    let address = Address {
        payload: bitcoin::util::address::Payload::WitnessProgram(witness_program),
        network: Network::Regtest,
    };

    let tx = Xmr::execute(&xmr_setup, &xmr::Swap {
        funding: btx_1,
        address,
        s: btc_setup.get_s(),
    }).unwrap();
    client.send_raw_transaction(&tx).unwrap();
}
