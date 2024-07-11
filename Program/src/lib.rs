use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token_2022::{
    extension::ExtensionType, instruction as token_2022_instruction, state::Mint,
};

use common::TokenMetadata;

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let mint_account = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let close_authority = next_account_info(accounts_iter)?;
    let payer: &AccountInfo = next_account_info(accounts_iter)?;
    let rent_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token2022_program = next_account_info(accounts_iter)?;

    if !mint_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    spl_token_2022::check_program_account(token2022_program.key)?;

    let token_metadata = TokenMetadata::try_from_slice(instruction_data)?;

    create_mint_account(mint_account, payer, system_program, token2022_program)?;
    initialize_close_authority(
        token2022_program,
        mint_account,
        close_authority,
        system_program,
    )?;

    initialize_mint(
        token2022_program,
        mint_account,
        mint_authority,
        rent_program,
        token_metadata,
    )?;

    Ok(())
}

fn create_mint_account<'a, 'b>(
    mint_account: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
    token2022_program: &'a AccountInfo<'b>,
) -> ProgramResult {
    let space_occupied =
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MintCloseAuthority])?;
    let payable_rent = Rent::get()?.minimum_balance(space_occupied);

    // Mint account
    msg!("Mint Account: {}", mint_account.key);
    let create_mint_account_instr = system_instruction::create_account(
        payer.key,
        mint_account.key,
        payable_rent,
        space_occupied as u64,
        token2022_program.key,
    );
    let create_mint_accounts = [
        mint_account.clone(),
        payer.clone(),
        system_program.clone(),
        token2022_program.clone(),
    ];
    invoke(&create_mint_account_instr, &create_mint_accounts)
}

fn initialize_close_authority<'a, 'b>(
    token2022_program: &'a AccountInfo<'b>,
    mint_account: &'a AccountInfo<'b>,
    close_authority: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
) -> ProgramResult {
    let mint_auth_instruction = token_2022_instruction::initialize_mint_close_authority(
        token2022_program.key,
        mint_account.key,
        Some(close_authority.key),
    )?;
    let mint_auth_accounts = [
        mint_account.clone(),
        close_authority.clone(),
        token2022_program.clone(),
        system_program.clone(),
    ];
    invoke(&mint_auth_instruction, &mint_auth_accounts)
}

fn initialize_mint<'a, 'b>(
    token2022_program: &'a AccountInfo<'b>,
    mint_account: &'a AccountInfo<'b>,
    mint_authority: &'a AccountInfo<'b>,
    rent_program: &'a AccountInfo<'b>,
    token_metadata: TokenMetadata,
) -> ProgramResult {
    let initialize_mint_instruction = token_2022_instruction::initialize_mint(
        token2022_program.key,
        mint_account.key,
        mint_authority.key,
        Some(mint_authority.key),
        token_metadata.decimals,
    )?;
    let initialize_mint_accounts = [
        mint_account.clone(),
        mint_authority.clone(),
        token2022_program.clone(),
        rent_program.clone(),
    ];
    invoke(&initialize_mint_instruction, &initialize_mint_accounts)
}

fn mint_token<'a, 'b>(
    token2022_program: &'a AccountInfo<'b>,
    mint_account: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    mint_authority: &'a AccountInfo<'b>,
    associated_token_account: &'a AccountInfo<'b>,
) -> ProgramResult {
    let mint_to_instruction = token_2022_instruction::mint_to(
        &token2022_program.key,
        &mint_account.key,
        &associated_token_account.key,
        &payer.key,
        &[&payer.key, &mint_account.key],
        10,
    )?;

    let accounts = [
        payer.clone(),
        mint_account.clone(),
        mint_authority.clone(),
        token2022_program.clone(),
        associated_token_account.clone(),
    ];

    invoke(&mint_to_instruction, &accounts)
}
