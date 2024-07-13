use std::{env, str::FromStr};

use common::MintOperation;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let program_id = Pubkey::from_str(args[1].as_str()).expect("Invalid Public Key");
    println!("PROGRAM_ID: {}", program_id);

    let mint_authority = Keypair::new();
    let mint_account = Keypair::new();
    let destination = Keypair::new();

    println!("Mint Authority: {}", mint_authority.pubkey());
    println!("Mint Account: {}", mint_account.pubkey());
    println!("Destination: {}", destination.pubkey());
    println!("Token2022 Program: {}", spl_token_2022::id());

    let mint_op = MintOperation::InitializeMint;
    let data = borsh::to_vec(&mint_op).unwrap();

    let init_instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(mint_authority.pubkey(), true),
            AccountMeta::new(mint_account.pubkey(), true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data,
    };

    let commitment = CommitmentConfig::finalized();

    let client = RpcClient::new_with_commitment("http://localhost:8899".to_string(), commitment);

    check_request_airdrop(&client, &mint_authority.pubkey(), 1);

    let message = Message::new(&[init_instruction], Some(&mint_authority.pubkey()));
    let mut tx = Transaction::new_unsigned(message);
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    tx.sign(&[&mint_authority, &mint_account], recent_blockhash);

    client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::finalized(),
        )
        .unwrap();

    check_request_airdrop(&client, &destination.pubkey(), 5);

    let signature = transfer_sol_get_sig(&destination, &client, &mint_authority);
    let received = get_received(&client, &signature);

    if received == 1 {
        mint_to_destination(
            &mint_authority,
            &mint_account,
            &destination,
            &client,
            program_id,
        );
    } else {
        panic!("Incorrect Amount");
    }
}

fn transfer_sol_get_sig(
    destination_keypair: &Keypair,
    client: &RpcClient,
    mint_authority: &Keypair,
) -> Signature {
    let transfer_instruction = solana_program::system_instruction::transfer(
        &destination_keypair.pubkey(),
        &mint_authority.pubkey(),
        LAMPORTS_PER_SOL,
    );
    let message = Message::new(&[transfer_instruction], Some(&destination_keypair.pubkey()));
    let mut tx = Transaction::new_unsigned(message);
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    tx.sign(&[destination_keypair], recent_blockhash);
    client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::finalized(),
        )
        .unwrap()
}

fn mint_to_destination(
    mint_authority: &Keypair,
    mint_account: &Keypair,
    destination: &Keypair,
    client: &RpcClient,
    program_id: Pubkey,
) {
    let signature = transfer_sol_get_sig(&destination, &client, &mint_authority);

    let received = get_received(&client, &signature);
    if received == 1 {
        let mint_op = MintOperation::MintTo(5);
        let data = borsh::to_vec(&mint_op).unwrap();

        let create_ata_instruction = create_associated_token_account(
            &destination.pubkey(),
            &destination.pubkey(),
            &mint_account.pubkey(),
            &spl_token_2022::id(),
        );
        let ata = get_associated_token_address_with_program_id(
            &destination.pubkey(),
            &mint_account.pubkey(),
            &spl_token_2022::id(),
        );

        let mint_to_instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(mint_authority.pubkey(), true),
                AccountMeta::new(mint_account.pubkey(), true),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
                AccountMeta::new_readonly(spl_token_2022::id(), false),
                AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
                AccountMeta::new(ata, false),
            ],
            data,
        };

        let message = Message::new(
            &[create_ata_instruction, mint_to_instruction],
            Some(&mint_authority.pubkey()),
        );
        let mut tx = Transaction::new_unsigned(message);
        let recent_blockhash = client.get_latest_blockhash().unwrap();
        tx.sign(
            &[&mint_authority, &mint_account, &destination],
            recent_blockhash,
        );
        client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::finalized(),
            )
            .unwrap();

        println!("Created ATA: {}", &ata);
        println!("Finished Minting To {} ATA", &ata);
    } else {
        panic!("Incorrect Ammount");
    }
}

fn check_request_airdrop(client: &RpcClient, account: &Pubkey, amount: u64) {
    if client.get_balance(&account).unwrap().eq(&0) {
        client
            .request_airdrop(&account, LAMPORTS_PER_SOL * amount)
            .unwrap();

        loop {
            if 1u64.gt(&client.get_balance(&account).unwrap()) {
                println!("Airdrop for {} has not reflected ...", account);
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else {
                println!("\nAirdrop for {} Authority has reflected!\n", account);

                break;
            }
        }
    }
}

fn get_received(client: &RpcClient, signature: &Signature) -> u64 {
    let get_tx = client
        .get_transaction(signature, UiTransactionEncoding::Binary)
        .unwrap();

    let tx_meta = get_tx.transaction.meta.unwrap();
    let (pre_balance, post_balance) = (tx_meta.pre_balances[1], tx_meta.post_balances[1]);

    (post_balance - pre_balance) / LAMPORTS_PER_SOL
}
