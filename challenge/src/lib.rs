use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::set_return_data,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};

entrypoint!(process_instruction);

const REDEEM_MEMBERSHIP_REWARD: u8 = 0;
const VERIFY_MEMBERSHIP: u8 = 1;
const GET_REWARD_VAULT_BALANCE: u8 = 2;
const GET_CAMPAIGN_TERMS: u8 = 3;
const BADGE_MINT_SEED: &[u8] = b"badge-mint";
const REWARD_VAULT_SEED: &[u8] = b"prize";
const BADGE_DECIMALS: u8 = 0;
const BADGE_SUPPLY: u64 = 1;

enum MembershipInstruction {
    RedeemReward,
    VerifyMembership,
    GetRewardVaultBalance,
    GetCampaignTerms,
}

impl MembershipInstruction {
    fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input {
            [REDEEM_MEMBERSHIP_REWARD] => Ok(Self::RedeemReward),
            [VERIFY_MEMBERSHIP] => Ok(Self::VerifyMembership),
            [GET_REWARD_VAULT_BALANCE] => Ok(Self::GetRewardVaultBalance),
            [GET_CAMPAIGN_TERMS] => Ok(Self::GetCampaignTerms),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

fn badge_mint_address(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[BADGE_MINT_SEED], program_id).0
}

fn prize_address(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[REWARD_VAULT_SEED], program_id).0
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match MembershipInstruction::unpack(instruction_data)? {
        MembershipInstruction::RedeemReward => redeem_membership_reward(program_id, accounts),
        MembershipInstruction::VerifyMembership => verify_membership(program_id, accounts),
        MembershipInstruction::GetRewardVaultBalance => {
            get_reward_vault_balance(program_id, accounts)
        }
        MembershipInstruction::GetCampaignTerms => {
            let mut terms = [0u8; 9];
            terms[0] = BADGE_DECIMALS;
            terms[1..].copy_from_slice(&BADGE_SUPPLY.to_le_bytes());
            set_return_data(&terms);
            Ok(())
        }
    }
}

fn verify_membership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let mut iter = accounts.iter();
    let badge_mint = next_account_info(&mut iter)?;
    let badge = next_account_info(&mut iter)?;

    validate_membership_mint(program_id, badge_mint)?;
    let token = load_membership_badge(badge, badge_mint.key)?;

    let mut result = [0u8; 41];
    result[0] = 1;
    result[1..33].copy_from_slice(token.owner.as_ref());
    result[33..].copy_from_slice(&token.amount.to_le_bytes());
    set_return_data(&result);
    Ok(())
}

fn get_reward_vault_balance(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let prize = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
    validate_reward_vault_address(program_id, prize)?;
    set_return_data(&prize.lamports().to_le_bytes());
    Ok(())
}

fn redeem_membership_reward(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let mut iter = accounts.iter();
    let player = next_account_info(&mut iter)?;
    let badge_mint = next_account_info(&mut iter)?;
    let borrowed_badge = next_account_info(&mut iter)?;
    let prize = next_account_info(&mut iter)?;

    if !player.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !player.is_writable || !prize.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    validate_membership_mint(program_id, badge_mint)?;
    let token = load_membership_badge(borrowed_badge, badge_mint.key)?;

    // Vulnerability: the decoded token authority is never bound to the player.
    // A production redemption service would require:
    // token.owner == *player.key
    let _unverified_badge_authority = token.owner;

    validate_reward_vault(program_id, prize)?;
    settle_reward(prize, player)
}

fn validate_membership_mint(program_id: &Pubkey, badge_mint: &AccountInfo) -> ProgramResult {
    if badge_mint.key != &badge_mint_address(program_id) || badge_mint.owner != &spl_token::id() {
        return Err(ProgramError::InvalidArgument);
    }
    let mint_data = badge_mint.try_borrow_data()?;
    let mint = Mint::unpack(&mint_data)?;
    if !mint.is_initialized || mint.decimals != BADGE_DECIMALS || mint.supply != BADGE_SUPPLY {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

fn load_membership_badge(
    borrowed_badge: &AccountInfo,
    expected_mint: &Pubkey,
) -> Result<TokenAccount, ProgramError> {
    if borrowed_badge.owner != &spl_token::id() {
        return Err(ProgramError::IllegalOwner);
    }
    let token_data = borrowed_badge.try_borrow_data()?;
    let token = TokenAccount::unpack(&token_data)?;
    if token.state != AccountState::Initialized || token.mint != *expected_mint || token.amount < 1
    {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(token)
}

fn validate_reward_vault(program_id: &Pubkey, prize: &AccountInfo) -> ProgramResult {
    validate_reward_vault_address(program_id, prize)?;
    if prize.lamports() == 0 {
        return Err(ProgramError::InsufficientFunds);
    }
    Ok(())
}

fn validate_reward_vault_address(program_id: &Pubkey, prize: &AccountInfo) -> ProgramResult {
    if prize.key != &prize_address(program_id) || prize.owner != program_id {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

fn settle_reward(prize: &AccountInfo, player: &AccountInfo) -> ProgramResult {
    let amount = prize.lamports();
    **prize.try_borrow_mut_lamports()? = 0;
    **player.try_borrow_mut_lamports()? = player
        .lamports()
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok(())
}
