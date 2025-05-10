//! Wallet management module for the RecruSearch program
//! 
//! This module handles all wallet-related functionality including:
//! - Token account management
//! - Reward distribution
//! - Transaction tracking
//! - Privacy-preserving wallet operations

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use anchor_lang::solana_program::pubkey::Pubkey;
use crate::error::RecruSearchError;

/// Represents a participant's wallet for managing tokens and rewards
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ParticipantWallet {
    /// The wallet's authority
    pub authority: Pubkey,
    /// The associated Phantom wallet address
    pub phantom_address: Pubkey,
    /// Main token account for study rewards
    pub main_token_account: Pubkey,
    /// Privacy key account for anonymous transactions
    pub privacy_key_account: Pubkey,
    /// Anonymous identifier for privacy-preserving operations
    pub anonymous_id: String,
    /// List of rewards received by the participant
    pub rewards: Vec<Reward>,
    /// List of transactions associated with this wallet
    pub transactions: Vec<Transaction>,
    pub total_earned: u64,
    pub is_initialized: bool,
    pub last_activity: i64,
}

/// Represents a reward received for participating in a study
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Reward {
    /// The study ID that generated this reward
    pub study_id: Pubkey,
    /// The amount of tokens received
    pub amount: u64,
    /// Timestamp when the reward was received
    pub timestamp: i64,
}

/// Represents a transaction in the participant's wallet
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: String,
    /// Current status of the transaction
    pub status: TransactionStatus,
    /// Timestamp of the transaction
    pub timestamp: i64,
}

/// Possible states for a transaction
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum TransactionStatus {
    /// Transaction is pending confirmation
    Pending,
    /// Transaction has been completed successfully
    Completed,
    /// Transaction failed
    Failed,
}

/// Context for initializing a participant's wallet
#[derive(Accounts)]
pub struct InitializePhantomWallet<'info> {
    /// The wallet account to be initialized
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 32 + 100 + 1000 + 1000
    )]
    pub wallet: Account<'info, ParticipantWallet>,
    
    /// The participant's main token account
    #[account(mut)]
    pub main_token_account: Account<'info, TokenAccount>,
    
    /// The participant's privacy key account
    #[account(mut)]
    pub privacy_key_account: Account<'info, TokenAccount>,
    
    /// The mint account for the reward tokens
    pub mint: Account<'info, Mint>,
    
    /// The Phantom wallet address
    pub phantom_wallet: Signer<'info>,
    
    /// The authority (participant) signing the transaction
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
    
    /// The associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// The system program
    pub system_program: Program<'info, System>,
}

/// Context for receiving a reward
#[derive(Accounts)]
pub struct ReceiveReward<'info> {
    /// The participant's wallet
    #[account(mut)]
    pub wallet: Account<'info, ParticipantWallet>,
    
    /// The study providing the reward
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    
    /// The researcher's token account
    #[account(mut)]
    pub researcher_token_account: Account<'info, TokenAccount>,
    
    /// The participant's main token account
    #[account(mut)]
    pub main_token_account: Account<'info, TokenAccount>,
    
    /// The Phantom wallet
    pub phantom_wallet: Signer<'info>,
    
    /// The authority (participant) signing the transaction
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
}

impl ParticipantWallet {
    /// Initializes a new participant wallet
    pub fn initialize(
        &mut self,
        phantom_address: Pubkey,
        main_token_account: Pubkey,
        privacy_key_account: Pubkey,
    ) -> Result<()> {
        require!(self.is_initialized == false, RecruSearchError::WalletAlreadyInitialized);
        
        self.phantom_address = phantom_address;
        self.main_token_account = main_token_account;
        self.privacy_key_account = privacy_key_account;
        self.total_earned = 0;
        self.is_initialized = true;
        self.last_activity = Clock::get()?.unix_timestamp;
        
        // Generate anonymous ID using Phantom address and timestamp
        self.anonymous_id = self.generate_anonymous_id()?;
        
        // Initialize empty transaction and reward history
        self.rewards = Vec::new();
        self.transactions = Vec::new();
        
        Ok(())
    }

    pub fn generate_anonymous_id(&self) -> Result<String> {
        // Create a unique anonymous ID by hashing the Phantom address and timestamp
        let mut hasher = anchor_lang::solana_program::hash::hashv(&[
            &self.phantom_address.to_bytes(),
            &self.last_activity.to_le_bytes(),
        ]);
        
        // Convert to a more readable format and add checksum
        let mut id = format!("anon_{:x}", hasher.to_bytes()[0..8].to_vec());
        let checksum = hasher.to_bytes()[0] % 16;
        id.push_str(&format!("_{:x}", checksum));
        
        Ok(id)
    }

    pub fn add_reward(&mut self, study_id: Pubkey, amount: u64) -> Result<()> {
        require!(amount > 0, RecruSearchError::InvalidRewardAmount);
        require!(self.is_initialized, RecruSearchError::WalletNotInitialized);

        let current_time = Clock::get()?.unix_timestamp;
        let transaction = Reward {
            study_id,
            amount,
            timestamp: current_time,
            transaction_id: format!("{:x}", study_id.to_bytes()[0..8].to_vec()),
        };

        self.rewards.push(transaction);
        self.total_earned = self.total_earned.checked_add(amount)
            .ok_or(RecruSearchError::RewardOverflow)?;
        self.last_activity = current_time;
        
        // Update transaction history
        self.transactions.push(Transaction {
            id: format!("reward_{:x}", study_id.to_bytes()[0..8].to_vec()),
            status: TransactionStatus::Completed,
            timestamp: current_time,
        });

        Ok(())
    }

    pub fn update_transaction_status(&mut self, transaction_id: String, status: TransactionStatus) -> Result<()> {
        if let Some(transaction) = self.reward_history.iter_mut()
            .find(|t| t.transaction_id == transaction_id) {
            transaction.status = status;
            self.last_activity = Clock::get()?.unix_timestamp;
        }
        Ok(())
    }

    pub fn get_reward_history(&self, study_id: Option<Pubkey>) -> Vec<&RewardTransaction> {
        self.reward_history.iter()
            .filter(|t| {
                if let Some(id) = study_id {
                    t.study_id == id
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn verify_phantom_ownership(&self, phantom_address: &Pubkey) -> bool {
        self.phantom_address == *phantom_address
    }
}

#[error_code]
pub enum WalletError {
    #[msg("Wallet not initialized")]
    NotInitialized,
    #[msg("Invalid Phantom wallet")]
    InvalidPhantomWallet,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Transaction failed")]
    TransactionFailed,
    #[msg("Invalid transaction ID")]
    InvalidTransactionId,
    #[msg("Privacy key update failed")]
    PrivacyKeyUpdateFailed,
    #[msg("Anonymous ID generation failed")]
    AnonymousIdGenerationFailed,
} 