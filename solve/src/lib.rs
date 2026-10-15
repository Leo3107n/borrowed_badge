use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    // TODO: present the server's borrowed token account as if it belonged to
    // the player, then invoke the challenge to drain the prize.
    Ok(())
}
