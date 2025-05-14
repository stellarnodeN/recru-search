use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::wallet::{ParticipantWallet, get_wallet_size};
use crate::error::RecruSearchError;

/// Context for initializing a Phantom wallet
#[derive(Accounts)]
pub struct InitializePhantomWallet<'info> {
    /// The participant initializing the wallet
    #[account(mut)]
    pub participant: Signer<'info>,

    /// The wallet account to be initialized
    #[account(
        init,
        payer = payer,
        space = get_wallet_size(),
        seeds = [b"wallet", participant.key().as_ref()],
        bump
    )]
    pub wallet: Account<'info, ParticipantWallet>,

    /// The Phantom wallet public key
    /// CHECK: This is the public key of the Phantom wallet
    pub phantom_public_key: AccountInfo<'info>,

    /// Payer for the initialization
    #[account(mut)]
    pub payer: Signer<'info>,


    /// System program
    pub system_program: Program<'info, System>,
}

/// Context for receiving rewards
#[derive(Accounts)]
pub struct ReceiveReward<'info> {
    /// The participant receiving the reward
    #[account(mut)]
    pub participant: Signer<'info>,

    /// The participant's wallet account
    #[account(
        mut,
        seeds = [b"wallet", participant.key().as_ref()],
        bump = wallet.bump,
    )]
    pub wallet: Account<'info, ParticipantWallet>,

    /// The token account receiving the reward
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    /// The token mint for the reward
    pub token_mint: Account<'info, token::Mint>,

    /// The token program
    pub token_program: Program<'info, Token>,
}

/// Initializes a wallet for the participant
pub fn initialize_phantom_wallet(ctx: Context<InitializePhantomWallet>) -> Result<()> {
    let wallet = &mut ctx.accounts.wallet;
    // Initialize the wallet using the ParticipantWallet's new method
    let wallet_data = ParticipantWallet::new(
        ctx.accounts.participant.key(),
        ctx.accounts.phantom_public_key.key(),
        ctx.bumps.wallet,
    )?;
    
    // Copy fields from the created wallet data to the account
    wallet.participant = wallet_data.participant;
    wallet.phantom_public_key = wallet_data.phantom_public_key;
    wallet.bump = wallet_data.bump;
    wallet.is_active = wallet_data.is_active;
    wallet.total_rewards = wallet_data.total_rewards;
    wallet.last_activity = wallet_data.last_activity;
    wallet.last_reward_at = wallet_data.last_reward_at;
    wallet.metadata_uri = wallet_data.metadata_uri;
    
    // Emit event
    emit!(WalletInitialized {
        participant: ctx.accounts.participant.key(),
        wallet: wallet.key(),
        phantom_public_key: ctx.accounts.phantom_public_key.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// Processes a reward for a participant
pub fn receive_reward(ctx: Context<ReceiveReward>, amount: u64) -> Result<()> {
    // Verify the token account is for the correct mint and owned by the participant
    require!(
        ctx.accounts.recipient_token_account.mint == ctx.accounts.token_mint.key(),
        RecruSearchError::InvalidTokenMint
    );
    
    require!(
        ctx.accounts.recipient_token_account.owner == ctx.accounts.participant.key(),
        RecruSearchError::Unauthorized
    );

    // Add reward to wallet using the ParticipantWallet's method
    let wallet = &mut ctx.accounts.wallet;
    wallet.add_reward(amount)?;
    
    // In a real implementation, you would transfer tokens here
    // For example:
    // token::transfer(
    //     CpiContext::new(
    //         ctx.accounts.token_program.to_account_info(),
    //         Transfer {
    //             from: source_token_account.to_account_info(),
    //             to: ctx.accounts.recipient_token_account.to_account_info(),
    //             authority: authority_info,
    //         },
    //     ),
    //     amount,
    // )?;
    
    // Update the wallet's last activity
    wallet.update_activity()?;
    
    // Emit event
    emit!(RewardReceived {
        participant: ctx.accounts.participant.key(),
        wallet: wallet.key(),
        amount,
        token_mint: ctx.accounts.token_mint.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// Event emitted when a wallet is initialized
#[event]
pub struct WalletInitialized {
    pub participant: Pubkey,
    pub wallet: Pubkey,
    pub phantom_public_key: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when a reward is received
#[event]
pub struct RewardReceived {
    pub participant: Pubkey,
    pub wallet: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
    pub timestamp: i64,
}

/// Context for updating wallet metadata
#[derive(Accounts)]
pub struct UpdateWalletMetadata<'info> {
    /// The wallet owner
    #[account(mut)]
    pub participant: Signer<'info>,
    
    /// The wallet account to update
    #[account(
        mut,
        has_one = participant @ RecruSearchError::Unauthorized,
        seeds = [b"wallet", participant.key().as_ref()],
        bump = wallet.bump,
    )]
    pub wallet: Account<'info, ParticipantWallet>,
}

/// Updates the metadata URI for a wallet
pub fn update_wallet_metadata(
    ctx: Context<UpdateWalletMetadata>,
    metadata_uri: Option<String>,
) -> Result<()> {
    let wallet = &mut ctx.accounts.wallet;
    
    // Update the metadata URI using the ParticipantWallet's method
    wallet.update_metadata_uri(metadata_uri)?;
    
    // Emit event
    emit!(WalletMetadataUpdated {
        participant: ctx.accounts.participant.key(),
        wallet: wallet.key(),
        metadata_uri: wallet.metadata_uri.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// Event emitted when wallet metadata is updated
#[event]
pub struct WalletMetadataUpdated {
    pub participant: Pubkey,
    pub wallet: Pubkey,
    pub metadata_uri: Option<String>,
    pub timestamp: i64,
}