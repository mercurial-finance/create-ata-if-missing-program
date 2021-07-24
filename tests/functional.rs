use solana_sdk::program_pack::Pack;
use solana_program_test::BanksClient;
use solana_sdk::{signature::Signer, transaction::Transaction,
    instruction::{AccountMeta, Instruction}, pubkey::Pubkey, system_program,
    sysvar
};
use solana_program_test::{processor, tokio, ProgramTest, ProgramTestBanksClientExt};
use assert_matches::assert_matches;
use create_ata_if_missing::process_instruction;

pub async fn get_token_account(
    banks_client: &mut BanksClient,
    token_account_pubkey: Pubkey,
) -> spl_token::state::Account {
    spl_token::state::Account::unpack_from_slice(
        &banks_client
            .get_account(token_account_pubkey)
            .await
            .unwrap()
            .unwrap()
            .data,
    )
    .unwrap()
}

#[tokio::test]
async fn test_process() {
    let program_id = Pubkey::new_unique();
    let pt = ProgramTest::new(
        "create_ata_if_missing",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    let user_address = Pubkey::new_unique();
    // We use the native mint (wSOL) to make testing easy
    let token_mint_address = spl_token::native_mint::id();
    let ata_address = spl_associated_token_account::get_associated_token_address(
        &user_address,
        &token_mint_address
    );

    /*
        [writeable,signer] Funding account (must be a system account)
        [writeable] Associated token account address to be created
        [] Wallet address for the new associated token account
        [] The token mint for the new associated token account
        [] System program
        [] SPL Token program
        [] Rent sysvar 
    */
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(ata_address, false),
            AccountMeta::new_readonly(user_address, false),
            AccountMeta::new_readonly(token_mint_address, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: vec![1],
    };

    // User ATA does not exist
    assert_matches!(
        banks_client
            .process_transaction(
                Transaction::new_signed_with_payer(
                    &[instruction.clone()],
                    Some(&payer.pubkey()),
                    &[&payer],
                    recent_blockhash,
                )
            )
            .await,
        Ok(())
    );

    let ata_account_state =
        get_token_account(&mut banks_client, ata_address)
        .await;

    assert_eq!(user_address, ata_account_state.owner);
    assert_eq!(token_mint_address, ata_account_state.mint);

    // User ATA exists
    // It is essential to get a new blockhash, otherwise the transaction is identical, then dropped
    let new_blockhash = banks_client.get_new_blockhash(&recent_blockhash).await.unwrap().0;
    assert_matches!(
        banks_client
            .process_transaction(
                Transaction::new_signed_with_payer(
                    &[instruction],
                    Some(&payer.pubkey()),
                    &[&payer],
                    new_blockhash,
                )
            )
            .await,
        Ok(())
    );
}