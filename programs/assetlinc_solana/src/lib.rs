use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;

// Program ID
declare_id!("H73K8LsRbV1jkcJkK3hQM2Z7TC4T1864Gwm3WtF3bAay");

// Constants for fees and periods
const BASIC_FEE: u64 = 10_000_000; // in lamports
const PREMIUM_FEE: u64 = 25_000_000; // in lamports
const ENTERPRISE_FEE: u64 = 100_000_000; // in lamports
const ROYALTY_PERCENTAGE: u8 = 3; // 3%
const MONTHLY_PERIOD: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
const TRIAL_PERIOD: i64 = 14 * 24 * 60 * 60; // 14 days in seconds

#[program]
pub mod subscription_service {
    use super::*;

    // Manage subscription
    pub fn manage_subscription(
        ctx: Context<ManageSubscription>,
        subscription_tier: u8,
    ) -> Result<()> {
        let user_data = &mut ctx.accounts.user_data;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Determine the fee based on subscription tier
        let tier_fee = match subscription_tier {
            0 => BASIC_FEE,    // Basic
            1 => PREMIUM_FEE,  // Premium
            2 => ENTERPRISE_FEE, // Enterprise
            _ => return Err(ErrorCode::InvalidSubscriptionTier.into()),
        };

        // Handle trial period
        let is_new_user = user_data.last_payment == 0;
        let within_trial_period = current_time - user_data.last_payment < TRIAL_PERIOD;

        if is_new_user || within_trial_period {
            user_data.subscription_tier = subscription_tier;
            user_data.last_payment = current_time;
            return Ok(());
        }

        // Require fee payment by transferring lamports
        let user = &ctx.accounts.user;
        let fee_receiver = &ctx.accounts.fee_receiver;

        invoke(
            &system_instruction::transfer(
                user.key,
                fee_receiver.key,
                tier_fee,
            ),
            &[
                user.to_account_info(),
                fee_receiver.to_account_info(),
            ],
        )?;

        // Update subscription details
        user_data.subscription_tier = subscription_tier;
        user_data.last_payment = current_time;
        Ok(())
    }

    // Register an asset as an NFT
    pub fn register_asset_as_nft(
        ctx: Context<RegisterAsset>,
        asset_name: String,
        asset_value: u64,
    ) -> Result<()> {
        let royalty_fee = asset_value * ROYALTY_PERCENTAGE as u64 / 100;

        // Require royalty fee payment by transferring lamports
        let user = &ctx.accounts.user;
        let fee_receiver = &ctx.accounts.fee_receiver;

        invoke(
            &system_instruction::transfer(
                user.key,
                fee_receiver.key,
                royalty_fee,
            ),
            &[
                user.to_account_info(),
                fee_receiver.to_account_info(),
            ],
        )?;

        // Register the NFT
        let nft_account = &mut ctx.accounts.nft_data;
        nft_account.name = asset_name;
        nft_account.value = asset_value;
        nft_account.owner = *user.key;

        Ok(())
    }

    // Check subscription status
    pub fn check_subscription(ctx: Context<CheckSubscription>) -> Result<()> {
        let user_data = &ctx.accounts.user_data;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        if current_time - user_data.last_payment >= MONTHLY_PERIOD {
            return Err(ErrorCode::SubscriptionExpired.into());
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ManageSubscription<'info> {
    #[account(mut)]
    pub user_data: Account<'info, UserData>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: Verified during transfer
    #[account(mut)]
    pub fee_receiver: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterAsset<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer = user, space = 8 + 64)]
    pub nft_data: Account<'info, NftData>,
    /// CHECK: Verified during transfer
    #[account(mut)]
    pub fee_receiver: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckSubscription<'info> {
    pub user_data: Account<'info, UserData>,
}

// Data structures
#[account]
pub struct UserData {
    pub subscription_tier: u8,
    pub last_payment: i64,
}

#[account]
pub struct NftData {
    pub name: String,
    pub value: u64,
    pub owner: Pubkey,
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid subscription tier.")]
    InvalidSubscriptionTier,
    #[msg("Incorrect fee amount.")]
    IncorrectFeeAmount,
    #[msg("Subscription has expired.")]
    SubscriptionExpired,
}
