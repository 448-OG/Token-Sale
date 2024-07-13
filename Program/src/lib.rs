use borsh::BorshDeserialize;
use common::MintOperation;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token_2022::{
    extension::{
        default_account_state::instruction::initialize_default_account_state, ExtensionType,
    },
    instruction::{mint_to, thaw_account},
    state::{AccountState, Mint},
};

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_info_iter = &mut accounts.iter();

    let mint_authority = next_account_info(accounts_info_iter)?;
    let mint_account = next_account_info(accounts_info_iter)?;
    let system_program = next_account_info(accounts_info_iter)?;
    let token2022_program = next_account_info(accounts_info_iter)?;
    let rent_program = next_account_info(accounts_info_iter)?;

    msg!("Mint Authority: {}", mint_authority.key);
    msg!("Mint Account: {}", mint_account.key);
    assert!(solana_program::system_program::id().eq(&system_program.key));
    assert!(spl_token_2022::id().eq(&token2022_program.key));

    let mint_op = MintOperation::try_from_slice(instruction_data)?;

    match mint_op {
        MintOperation::InitializeMint => init_mit(
            mint_account,
            mint_authority,
            token2022_program,
            rent_program,
        ),
        MintOperation::MintTo(no_of_tokens) => {
            let ata = next_account_info(accounts_info_iter)?;
            msg!("Destination ATA: {}", ata.key);
            mint_to_ata(
                ata,
                mint_account,
                token2022_program,
                mint_authority,
                no_of_tokens,
            )
        }
    }
}

fn init_mit<'a, 'b>(
    mint_account: &'a AccountInfo<'b>,
    mint_authority: &'a AccountInfo<'b>,
    token2022_program: &'a AccountInfo<'b>,
    rent_program: &'a AccountInfo<'b>,
) -> ProgramResult {
    let extension = ExtensionType::DefaultAccountState;
    let extension_mint_len = ExtensionType::try_calculate_account_len::<Mint>(&[extension])?;

    let rent = Rent::get()?.minimum_balance(extension_mint_len);
    let decimals = 0u8;

    let create_mint_instruction = system_instruction::create_account(
        &mint_authority.key,
        &mint_account.key,
        rent,
        extension_mint_len as u64,
        &token2022_program.key,
    );

    let default_state_instruction = initialize_default_account_state(
        &token2022_program.key,
        &mint_account.key,
        &AccountState::Frozen,
    )?;

    let initialize_mint_instruction = spl_token_2022::instruction::initialize_mint(
        &token2022_program.key,
        &mint_account.key,
        &mint_authority.key,
        Some(&mint_authority.key),
        decimals,
    )?;

    invoke(
        &create_mint_instruction,
        &[
            mint_authority.clone(),
            mint_account.clone(),
            token2022_program.clone(),
            rent_program.clone(),
        ],
    )?;

    invoke(
        &default_state_instruction,
        &[mint_account.clone(), token2022_program.clone()],
    )?;

    invoke(
        &initialize_mint_instruction,
        &[
            mint_authority.clone(),
            mint_account.clone(),
            token2022_program.clone(),
            rent_program.clone(),
        ],
    )?;

    msg!("Mint with `Frozen` default state initialized successfully");

    Ok(())
}

fn mint_to_ata<'a, 'b>(
    ata: &'a AccountInfo<'b>,
    mint_account: &'a AccountInfo<'b>,
    token2022_program: &'a AccountInfo<'b>,
    mint_authority: &'a AccountInfo<'b>,
    no_of_tokens: u64,
) -> ProgramResult {
    msg!("ATA: {}", ata.key,);
    let thaw_to_instruction = thaw_account(
        &token2022_program.key,
        &ata.key,
        &mint_account.key,
        &mint_authority.key,
        &[&mint_authority.key, &mint_account.key],
    )?;

    let mint_to_instruction = mint_to(
        &token2022_program.key,
        &mint_account.key,
        &ata.key,
        &mint_authority.key,
        &[&mint_authority.key, &mint_account.key],
        no_of_tokens,
    )?;

    invoke(
        &thaw_to_instruction,
        &[
            token2022_program.clone(),
            ata.clone(),
            mint_account.clone(),
            mint_authority.clone(),
        ],
    )?;

    invoke(
        &mint_to_instruction,
        &[
            token2022_program.clone(),
            ata.clone(),
            mint_account.clone(),
            mint_authority.clone(),
        ],
    )?;

    msg!("Minted {} tokens to {}!", no_of_tokens, ata.key);

    Ok(())
}
