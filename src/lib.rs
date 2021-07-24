use solana_program::{
    program::invoke_signed,
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
    program_pack::Pack, account_info::next_account_info, program_error::ProgramError,
    msg
};
use spl_token;
use spl_associated_token_account;

entrypoint!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let funding_account = next_account_info(account_iter)?;
    let ata_account = next_account_info(account_iter)?;
    let wallet_account = next_account_info(account_iter)?;
    let spl_token_mint_account= next_account_info(account_iter)?;
    let system_program_account = next_account_info(account_iter)?;
    let token_program_account = next_account_info(account_iter)?;
    let rent_sysvar_account= next_account_info(account_iter)?;
    let ata_program_account = next_account_info(account_iter)?;
    
    if *ata_account.owner == spl_token::id() {
        let ata_state = spl_token::state::Account::unpack(&ata_account.data.borrow())?;
        // This can only happen if the owner transfered ownership to someone else but let's check anyway
        if ata_state.owner != *wallet_account.key {
            return Err(ProgramError::Custom(123));
        }
        msg!("ATA already exists");
    }
    else {
        /*
            [writeable,signer] Funding account (must be a system account)
            [writeable] Associated token account address to be created
            [] Wallet address for the new associated token account
            [] The token mint for the new associated token account
            [] System program
            [] SPL Token program
            [] Rent sysvar 
        */
        invoke_signed(
            &spl_associated_token_account::create_associated_token_account(
                &funding_account.key,
                &wallet_account.key,
                &spl_token_mint_account.key
            ),
            &[
                funding_account.clone(),
                ata_account.clone(),
                wallet_account.clone(),
                spl_token_mint_account.clone(),
                system_program_account.clone(),
                token_program_account.clone(),
                rent_sysvar_account.clone(),
                ata_program_account.clone(),
            ],
            &[]
        )?;
    }

    Ok(())
}