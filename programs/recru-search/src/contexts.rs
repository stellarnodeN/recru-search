use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint, Token};
use anchor_spl::metadata::Metadata;
use crate::state::admin::Admin;
use crate::state::researcher::Researcher;
use crate::state::participant::Participant;
use crate::state::study::Study;
use crate::state::consent::Consent;
use crate::error::RecruSearchError;
// For metadata, use UncheckedAccount as a placeholder for Metaplex metadata accounts

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterResearcher<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 100 + 100 + 1)]
    pub researcher: Account<'info, Researcher>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateStudy<'info> {
    #[account(init, payer = researcher, space = 8 + 32 + 100 + 500 + 100 + 8 + 4 + 4 + 1 + 8)]
    pub study: Account<'info, Study>,
    #[account(mut)]
    pub researcher: Account<'info, Researcher>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterParticipant<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 100 + 8 + 4)]
    pub participant: Account<'info, Participant>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinStudy<'info> {
    #[account(mut)]
    pub study: Account<'info, Study>,
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CompleteStudy<'info> {
    #[account(mut)]
    pub study: Account<'info, Study>,
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    #[account(mut)]
    pub researcher: Account<'info, Researcher>,
    #[account(mut)]
    pub researcher_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub participant_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyResearcher<'info> {
    #[account(mut)]
    pub researcher: Account<'info, Researcher>,
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeConsentNFT<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    // Admin account is required to verify admin authority
    pub admin: Account<'info, Admin>,
    
    // Account for the consent NFT (this will be initialized)
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + 32 + 32 + 40 + 40 + 8 + 9 + 1 + 32 + 1 + 8 + 8 + 200, // Appropriate space for Consent
        seeds = [b"consent_nft", admin.key().as_ref()],
        bump
    )]
    pub consent: Account<'info, Consent>,
    
    // Standard accounts needed for NFT operations
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub metadata_program: Program<'info, Metadata>,
}

#[derive(Accounts)]
pub struct InitializePsyPoints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeAdmin<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 8 + 8 + 8)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReviewResearcherVerification<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub researcher: Account<'info, Researcher>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateStudyStatus<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub study: Account<'info, Study>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ManageParticipantStatus<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct TrackStudyProgress<'info> {
    #[account(mut)]
    pub study: Account<'info, Study>,
    // Add more accounts if needed
}

#[derive(Accounts)]
pub struct SubmitStudyFeedback<'info> {
    #[account(mut)]
    pub study: Account<'info, Study>,
    // Add more accounts if needed
}

#[derive(Accounts)]
pub struct UpdateParticipantProfile<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        has_one = authority,
        constraint = !participant.banned @ RecruSearchError::Unauthorized
    )]
    pub participant: Account<'info, Participant>,
}

#[derive(Accounts)]
pub struct UpdateInterests<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        has_one = authority,
        constraint = !participant.banned @ RecruSearchError::Unauthorized
    )]
    pub participant: Account<'info, Participant>,
}

#[derive(Accounts)]
pub struct IssueConsentNFT<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub study: Account<'info, Study>,
    
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    
    #[account(mut)]
    pub consent: Account<'info, Consent>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub participant_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    /// CHECK: Mint authority
    #[account(mut)]
    pub mint_authority: Signer<'info>,
    
    /// CHECK: Payer for transaction
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    
    /// CHECK: Metadata program ID
    pub token_metadata_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RevokeConsent<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub consent: Account<'info, Consent>,
    
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub participant_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializePrivacyManager<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GrantDataAccess<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeDataAccess<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    pub system_program: Program<'info, System>,
}