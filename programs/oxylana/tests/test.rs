use anchor_client::{Client, Cluster, Program};
use anchor_lang::system_program;
use oxylana::RustStation;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature},
    signer::Signer,
};

use std::{error::Error, rc::Rc};

#[test]
fn test_contract() -> Result<(), Box<dyn Error>> {
    // Initialize client and program
    let key: Rc<Keypair> = Rc::new(
        read_keypair_file("../../oxyzEsUj9CV6HsqPCUZqVwrFJJvpd9iCBrPdzTBWLBb.json").unwrap(),
    );
    let client: Client = Client::new(Cluster::Localnet, Rc::clone(&key) as Rc<dyn Signer>);
    let program: Program = client.program(oxylana::ID);

    // Build, sign, and send program instruction
    let rust_station: Pubkey = RustStation::get_pda(&key.pubkey());
    let sig: Signature = program
        .request()
        .accounts(oxylana::accounts::SignDemo {
            user: key.pubkey(),
            rust_station,
            system_program: system_program::ID,
        })
        .args(oxylana::instruction::SignDemo {/* this ix has no args */})
        .payer(Rc::clone(&key) as Rc<dyn Signer>)
        .signer(&*key)
        .send()?;

    println!("demo sig: {sig}");

    // Retrieve and validate state
    let rust_station_account: RustStation = program.account(rust_station)?;
    assert!(rust_station_account.oxidized);

    Ok(())
}
