/*
This example is a simple iOS-style calculator. This particular example can run any platform - Web, Mobile, Desktop.
This calculator version uses React-style state management. All state is held as individual use_states.
*/

use std::str::FromStr;

use anchor_lang::{
    prelude::AccountMeta,
    solana_program::{instruction::Instruction, message::Message},
    system_program, AccountDeserialize, InstructionData,
};

use dioxus::prelude::*;
use gloo_utils::format::JsValueSerdeExt;
use oxylana::RustStation;
use solana_client_wasm::solana_sdk::{pubkey::Pubkey, transaction::Transaction};

mod phantom;
use phantom::{solana, ConnectResponse};

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus_web::launch(web_app);
}

fn web_app(cx: Scope) -> Element {
    let user_state = use_state(&cx, || Option::<Pubkey>::None);
    let account_state = use_state(&cx, || Option::<RustStation>::None);
    let client_state = use_state(&cx, || {
        solana_client_wasm::WasmClient::new("http://localhost:8899")
    });

    // Get user state
    let connected = user_state.is_some();
    let text: &str = if connected {
        "Disconnect"
    } else {
        "Connect Wallet"
    };

    // Get account state
    let initialized = account_state.is_some();
    if let Some(user) = **user_state {
        let rust_station_pda = RustStation::get_pda(&user);
        let client = client_state.to_owned();
        let account = account_state.to_owned();
        if !initialized {
            cx.spawn(async move {
                if let Ok(rust_station_data) = client.get_account_data(&rust_station_pda).await {
                    log::info!("obtained data");
                    let rust_station =
                        RustStation::try_deserialize(&mut &rust_station_data[..]).unwrap();
                    log::info!("deserialized data");
                    account.set(Some(rust_station));
                }
            });
        }
    }

    cx.render(rsx! {
        button {
            onclick: move |_| {
                // This is a second move, so we need another copy
                // It is a copy of an Rc so it's pretty cheap
                let user_state = user_state.to_owned();
                cx.spawn(connect_or_disconnect(user_state))
            },
            text
        }
        if initialized {
            log::info!("attempting to display oxidized");
            rsx! {
                button {
                    "Oxidized"
                }
            }
        } else if connected {
            rsx! {
            button {
                    onclick: move |_| {
                        // This is a second move, so we need another copy
                        // It is a copy of an Rc so it's pretty cheap
                        let user_state = user_state.to_owned();
                        let account_state = account_state.to_owned();
                        cx.spawn(sign_and_send(user_state, account_state))
                    },
                    "Oxidize"
                }
            }
        }
    })
}

async fn connect_or_disconnect(account_state: UseState<Option<Pubkey>>) {
    let connected = account_state.is_some();
    log::info!("connected state is {connected}");
    if connected {
        let response = solana.disconnect().await;
        log::info!("disconnected: {:?}", response);
        account_state.set(None);
    } else {
        let response = solana.connect().await;
        log::info!("is connected: {:?}", solana.is_connected());
        if solana.is_connected() {
            log::info!("connected");
            let response: ConnectResponse = response.into_serde().unwrap();
            log::info!("user connected: {}", response.public_key);
            account_state.set(Some(
                Pubkey::from_str(&response.public_key)
                    .expect("phantom should only return valid pubkeys"),
            ));
        }
    }
}

/// This is only called when a RustStation is not initialized.
async fn sign_and_send(
    user_state: UseState<Option<Pubkey>>,
    account_state: UseState<Option<RustStation>>,
) {
    let client = solana_client_wasm::WasmClient::new("http://localhost:8899");

    if let Some(user) = *user_state {
        if account_state.is_none() {
            // First try to see if initialized and if so return early
            let rust_station_pda = RustStation::get_pda(&user);
            if let Ok(rust_station_data) = client.get_account_data(&rust_station_pda).await {
                let rust_station =
                    RustStation::try_deserialize(&mut &rust_station_data[..]).unwrap();
                account_state.set(Some(rust_station));
                return;
            }

            // Get instruction data
            let ix_data: Vec<u8> = oxylana::instruction::SignDemo {}.data();

            // Build instruction with correct progra, accounts, ix data
            let ix: Instruction = Instruction {
                program_id: oxylana::ID,
                accounts: vec![
                    AccountMeta::new_readonly(user, true),
                    AccountMeta::new(rust_station_pda, false),
                    AccountMeta::new_readonly(system_program::ID, false),
                    AccountMeta::new_readonly(oxylana::ID, false),
                ],
                data: ix_data,
            };

            // Build message with ix, signer, and latest blockhash
            let message = Message::new_with_blockhash(
                &[ix],
                Some(&user),
                &client.get_latest_blockhash().await.unwrap(),
            );

            // Build and encode transaction
            let tx = Transaction::new_unsigned(message);
            let message = bs58::encode(tx.message_data()).into_string();

            // Build json string expected by phantom
            // Refer to phantom docs for more information
            let json_string = format!(
                r#"{{
                "method": "signAndSendTransaction",
                "params": {{
                    "message": "{message}"
                }}
            }}"#
            );

            // Convert to JSON java script type and send to phantom
            let params = js_sys::JSON::parse(&json_string).unwrap();
            std::panic::set_hook(Box::new(|_| {}));
            solana.request(params).await;
        }
    }
}
