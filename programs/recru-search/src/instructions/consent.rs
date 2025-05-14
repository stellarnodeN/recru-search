use anchor_lang::prelude::*;
use anchor_spl::metadata::{self, Metadata};
use anchor_spl::token;
// Use proper import path for DataV2 in version 4.1.2
use mpl_token_metadata::types::DataV2;
use crate::error::RecruSearchError;
use crate::contexts::{InitializeConsentNFT, IssueConsentNFT, RevokeConsent};

pub fn initialize_consent_nft(ctx: Context<InitializeConsentNFT>) -> Result<()> {
    let consent = &mut ctx.accounts.consent;
    let admin = &ctx.accounts.admin;
    
    // Verify admin authority
    require!(admin.is_authorized(&ctx.accounts.authority.key()), RecruSearchError::UnauthorizedAdmin);
    
    // Initialize consent NFT
    consent.authority = admin.key();
    consent.mint = ctx.accounts.mint.key();
    // Access bump directly from the ctx.bumps
    consent.bump = ctx.bumps.consent;
    consent.total_issued = 0;
    consent.total_revoked = 0;
    // Initialize empty consent versions
    consent.consent_versions = vec![];
    
    Ok(())
}

pub fn issue_consent_nft(
    ctx: Context<IssueConsentNFT>,
    consent_version: String,
    consent_hash: String,
) -> Result<()> {
    let consent = &mut ctx.accounts.consent;
    let study = &mut ctx.accounts.study;
    let participant = &mut ctx.accounts.participant;
    
    // Verify that the participant is eligible for the study
    require!(participant.profile.is_eligible_for_study(study), RecruSearchError::ParticipantNotEligible);
    
    // Verify that the participant does not already have an active consent
    require!(!participant.has_active_consent, RecruSearchError::DuplicateConsent);

    // Create the consent NFT metadata
    let issued_at = Clock::get()?.unix_timestamp;
    
    // Store consent data in program's own format
    let metadata_uri = format!("https://recrusearch.io/consent/{}/{}/{}", 
                             study.key(), 
                             participant.key(),
                             issued_at);
    
    // Update consent status for participant
    participant.update_consent_status(true)?;
    
    // Increment the study's consent count
    study.increment_consent()?;
    
    // Increment the consent nft issued count
    consent.total_issued += 1;
    
    // First, create the metadata account via CPI to Token Metadata program
    let name = format!("Consent-{}", consent_version);
    let symbol = "CONSENT".to_string();
    let uri = metadata_uri;
    
    // Create metadata account using direct parameters instead of DataV2
    // Create a DataV2 struct that the function expects
    let data = DataV2 {
        name: name,
        symbol: symbol,
        uri: uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };
    
    metadata::create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            metadata::CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.mint_authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
        data,  // Pass the DataV2 struct
        true,  // Update authority is signer
        true,  // Is mutable
        None,  // Collection details
    )?;
    
    // Now mint the NFT to the participant's token account
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.participant_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
        ),
        1, // Mint 1 token as it's an NFT
    )?;

    // Emit consent issuance event
    emit!(ConsentIssued {
        participant: participant.key(),
        study: study.key(),
        version: consent_version,
        hash: consent_hash,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

pub fn revoke_consent(ctx: Context<RevokeConsent>) -> Result<()> {
    let consent = &mut ctx.accounts.consent;
    let participant = &mut ctx.accounts.participant;
    let clock = Clock::get()?;
    
    // Verify participant has active consent
    require!(participant.has_active_consent, RecruSearchError::NoActiveConsent);
    
    // Update participant's consent status
    participant.update_consent_status(false)?;
    
    // Update consent NFT stats
    consent.total_revoked += 1;
    
    // Burn NFT from participant's token account
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.participant_token_account.to_account_info(),
                authority: ctx.accounts.consent.to_account_info(),
            },
        ),
        1, // Burn 1 token
    )?;
    
    // Emit event
    emit!(ConsentRevoked {
        participant: participant.key(),
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

#[event]
pub struct ConsentIssued {
    pub participant: Pubkey,
    pub study: Pubkey,
    pub version: String,
    pub hash: String,
    pub timestamp: i64,
}

#[event]
pub struct ConsentRevoked {
    pub participant: Pubkey,
    pub timestamp: i64,
}