//! RecruSearch - A Solana-based platform for research study recruitment and management
//! 
//! This program enables researchers to create and manage studies, while allowing participants
//! to join studies, provide consent, and receive rewards. It includes features for privacy,
//! token-based rewards, and study progress tracking.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use metaplex_core::nft::{Nft, NftMetadata};
use metaplex_core::token::{Token as MetaplexToken, TokenAccount as MetaplexTokenAccount};

pub mod study;
pub mod participant;
pub mod admin;
pub mod privacy;
pub mod researcher;
pub mod wallet;
pub mod consent;

use study::*;
use participant::*;
use admin::*;
use privacy::*;
use researcher::*;
use wallet::*;
use consent::*;

/// Represents a participant in the research platform
/// Stores participant information, wallet details, and study participation status
#[account]
pub struct Participant {
    /// The participant's wallet authority
    pub authority: Pubkey,
    /// Participant's profile information
    pub profile: ParticipantProfile,
    /// Participant's wallet information
    pub wallet: ParticipantWallet,
    /// Hash of the participant's eligibility proof
    pub eligibility_proof: String,
    /// Timestamp when the participant registered
    pub registered_at: i64,
    /// Number of studies the participant is currently active in
    pub active_studies: u32,
    /// Number of studies the participant has completed
    pub completed_studies: u32,
    /// Whether the participant has chosen to remain anonymous
    pub is_anonymous: bool,
    /// Whether the participant has active consent
    pub has_active_consent: bool,
    /// Timestamp when consent was issued
    pub consent_issued_at: i64,
    /// Timestamp when consent was revoked (if applicable)
    pub consent_revoked_at: Option<i64>,
    /// Participant's status
    pub status: ParticipantStatus,
    /// Timestamp when the status was last updated
    pub status_updated_at: i64,
    /// Reason for suspension/ban (if applicable)
    pub status_reason: Option<String>,
    /// Participant's public identifier (anonymous)
    pub public_id: String,
    /// Participant's private identifier (encrypted)
    pub private_id: String,
    /// Last time privacy settings were updated
    pub privacy_updated_at: i64,
    /// Whether the participant is verified
    pub is_verified: bool,
    /// Timestamp when verification was last checked
    pub verification_checked_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ParticipantStatus {
    Active,
    Suspended,
    Banned,
    Frozen,
    Verified,
    Unverified,
}

impl Participant {
    pub fn new(
        authority: Pubkey,
        eligibility_proof: String,
        is_anonymous: bool,
        public_id: String,
        private_id: String,
    ) -> Self {
        Self {
            authority,
            profile: ParticipantProfile::default(),
            wallet: ParticipantWallet::default(),
            eligibility_proof,
            registered_at: Clock::get().unwrap().unix_timestamp,
            active_studies: 0,
            completed_studies: 0,
            is_anonymous,
            has_active_consent: false,
            consent_issued_at: 0,
            consent_revoked_at: None,
            status: ParticipantStatus::Unverified,
            status_updated_at: Clock::get().unwrap().unix_timestamp,
            status_reason: None,
            public_id,
            private_id,
            privacy_updated_at: Clock::get().unwrap().unix_timestamp,
            is_verified: false,
            verification_checked_at: 0,
        }
    }

    pub fn update_status(&mut self, status: ParticipantStatus, reason: Option<String>) -> Result<()> {
        match (self.status, status) {
            (ParticipantStatus::Active, ParticipantStatus::Suspended) => {},
            (ParticipantStatus::Active, ParticipantStatus::Banned) => {},
            (ParticipantStatus::Active, ParticipantStatus::Frozen) => {},
            (ParticipantStatus::Suspended, ParticipantStatus::Active) => {},
            (ParticipantStatus::Frozen, ParticipantStatus::Active) => {},
            (ParticipantStatus::Banned, ParticipantStatus::Active) => {},
            _ => return Err(error!(AdminError::InvalidParticipantAction)),
        }

        self.status = status;
        self.status_reason = reason;
        self.status_updated_at = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }

    pub fn update_privacy_settings(&mut self, is_anonymous: bool) -> Result<()> {
        self.is_anonymous = is_anonymous;
        self.privacy_updated_at = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }
}

/// Represents a participant's wallet for managing tokens and rewards
#[account]
#[derive(Default)]
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
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum TransactionStatus {
    /// Transaction is pending confirmation
    Pending,
    /// Transaction has been completed successfully
    Completed,
    /// Transaction failed
    Failed,
}

/// Represents a research study in the platform
#[account]
pub struct Study {
    /// The researcher's authority
    pub authority: Pubkey,
    /// Title of the study
    pub title: String,
    /// Detailed description of the study
    pub description: String,
    /// Hash of the study's eligibility criteria
    pub criteria_hash: String,
    /// Amount of tokens awarded to participants
    pub reward_amount: u64,
    /// Maximum number of participants allowed
    pub max_participants: u32,
    /// Current number of participants
    pub current_participants: u32,
    /// Study status
    pub status: StudyStatus,
    /// Timestamp when the study was created
    pub created_at: i64,
    /// Timestamp when the study was last updated
    pub updated_at: i64,
    /// Number of participants who completed the study
    pub completed_participants: u32,
    /// Analytics data for the study
    pub analytics: StudyAnalytics,
    /// Category of the study
    pub category: String,
    /// Timestamp when the study was completed
    pub completed_at: Option<i64>,
    /// Timestamp when the study was suspended
    pub suspended_at: Option<i64>,
    /// Reason for suspension
    pub suspension_reason: Option<String>,
    /// Timestamp when the study was rejected
    pub rejected_at: Option<i64>,
    /// Reason for rejection
    pub rejection_reason: Option<String>,
}

impl Study {
    pub fn new(
        authority: Pubkey,
        title: String,
        description: String,
        criteria_hash: String,
        reward_amount: u64,
        max_participants: u32,
        category: String,
    ) -> Self {
        Self {
            authority,
            title,
            description,
            criteria_hash,
            reward_amount,
            max_participants,
            current_participants: 0,
            status: StudyStatus::Draft,
            created_at: Clock::get().unwrap().unix_timestamp,
            updated_at: Clock::get().unwrap().unix_timestamp,
            completed_participants: 0,
            analytics: StudyAnalytics::default(),
            category,
            completed_at: None,
            suspended_at: None,
            suspension_reason: None,
            rejected_at: None,
            rejection_reason: None,
        }
    }

    pub fn update_status(&mut self, status: StudyStatus) -> Result<()> {
        match (self.status, status) {
            (StudyStatus::Draft, StudyStatus::PendingReview) => {},
            (StudyStatus::PendingReview, StudyStatus::Active) => {},
            (StudyStatus::Active, StudyStatus::Inactive) => {},
            (StudyStatus::Active, StudyStatus::Completed) => {},
            (StudyStatus::Active, StudyStatus::Suspended) => {},
            (StudyStatus::Suspended, StudyStatus::Active) => {},
            _ => return Err(error!(AdminError::InvalidStatusTransition)),
        }

        self.status = status;
        self.updated_at = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum StudyStatus {
    Draft,
    PendingReview,
    Active,
    Inactive,
    Completed,
    Suspended,
    Rejected,
}

/// Analytics data for a study
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StudyAnalytics {
    /// Percentage of participants who completed the study
    pub completion_rate: f32,
    /// Total number of participants who joined the study
    pub total_participants: u32,
    /// Average rating given by participants
    pub average_rating: f32,
}

declare_id!("BkXcFAo2TFkXRm9WsKUxikgNYvvR3Pm3yS9xLdqaeJoo");

/// Main program module containing all instruction handlers
#[program]
pub mod recru_search {
    use super::*;

    /// Initializes the program and creates the admin account
    /// This must be called first to set up the program's admin
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let admin = &mut ctx.accounts.admin;
        admin.authority = ctx.accounts.authority.key();
        Ok(())
    }

    /// Initializes the PsyPoints token mint
    /// Creates the token that will be used for study rewards
    pub fn initialize_psypoints(ctx: Context<InitializePsyPoints>) -> Result<()> {
        // Token initialization is handled by the account constraints
        Ok(())
    }

    /// Creates a new token account for a user
    /// Used for managing study rewards and privacy tokens
    pub fn create_token_account(ctx: Context<CreateTokenAccount>) -> Result<()> {
        // Token account creation is handled by the account constraints
        Ok(())
    }

    /// Registers a new researcher in the platform
    /// Requires institution and credentials information
    pub fn register_researcher(
        ctx: Context<RegisterResearcher>,
        institution: String,
        credentials_hash: String,
    ) -> Result<()> {
        let researcher = &mut ctx.accounts.researcher;
        researcher.authority = ctx.accounts.authority.key();
        researcher.institution = institution;
        researcher.credentials_hash = credentials_hash;
        researcher.is_verified = false;
        Ok(())
    }

    /// Creates a new research study
    /// Requires study details and reward information
    pub fn create_study(
        ctx: Context<CreateStudy>,
        title: String,
        description: String,
        criteria_hash: String,
        reward_amount: u64,
        max_participants: u32,
    ) -> Result<()> {
        let study = &mut ctx.accounts.study;
        study.authority = ctx.accounts.researcher.key();
        study.title = title;
        study.description = description;
        study.criteria_hash = criteria_hash;
        study.reward_amount = reward_amount;
        study.max_participants = max_participants;
        study.current_participants = 0;
        study.is_active = true;
        study.created_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Registers a new participant in the platform
    /// Requires eligibility proof for verification
    pub fn register_participant(
        ctx: Context<RegisterParticipant>,
        eligibility_proof: String,
    ) -> Result<()> {
        let participant = &mut ctx.accounts.participant;
        participant.authority = ctx.accounts.authority.key();
        participant.eligibility_proof = eligibility_proof;
        participant.registered_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Allows a participant to join a study
    /// Checks study capacity and participant eligibility
    pub fn join_study(ctx: Context<JoinStudy>) -> Result<()> {
        let study = &mut ctx.accounts.study;
        let participant = &mut ctx.accounts.participant;

        require!(study.is_active, RecruSearchError::StudyInactive);
        require!(
            study.current_participants < study.max_participants,
            RecruSearchError::StudyFull
        );

        study.current_participants = study.current_participants.checked_add(1).unwrap();
        participant.active_studies = participant.active_studies.checked_add(1).unwrap();

        Ok(())
    }

    /// Marks a study as completed for a participant and distributes rewards
    /// Handles token transfers and updates study/participant statistics
    pub fn complete_study(ctx: Context<CompleteStudy>) -> Result<()> {
        let study = &mut ctx.accounts.study;
        let participant = &mut ctx.accounts.participant;

        require!(study.is_active, RecruSearchError::StudyInactive);
        require!(
            ctx.accounts.researcher.key() == study.authority,
            RecruSearchError::UnauthorizedResearcher
        );

        // Transfer reward tokens to participant
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.researcher_token_account.to_account_info(),
                to: ctx.accounts.participant_token_account.to_account_info(),
                authority: ctx.accounts.researcher.to_account_info(),
            },
        );

        token::transfer(transfer_ctx, study.reward_amount)?;

        // Update participant stats
        participant.active_studies = participant.active_studies.checked_sub(1).unwrap();
        participant.completed_studies = participant.completed_studies.checked_add(1).unwrap();

        // Update study stats
        study.completed_participants = study.completed_participants.checked_add(1).unwrap();

        Ok(())
    }

    /// Verifies a researcher's credentials
    /// Can only be called by the admin
    pub fn verify_researcher(ctx: Context<VerifyResearcher>) -> Result<()> {
        let researcher = &mut ctx.accounts.researcher;
        require!(
            ctx.accounts.admin.authority == ctx.accounts.authority.key(),
            RecruSearchError::UnauthorizedAdmin
        );
        researcher.is_verified = true;
        Ok(())
    }

    /// Initializes the consent NFT mint and metadata
    /// Creates the NFT that will be used to track participant consent
    pub fn initialize_consent_nft(ctx: Context<InitializeConsentNFT>) -> Result<()> {
        // Create metadata for the consent NFT
        let cpi_context = CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            create_metadata_accounts_v3::cpi::accounts::CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.authority.to_account_info(),
                payer: ctx.accounts.authority.to_account_info(),
                update_authority: ctx.accounts.authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        create_metadata_accounts_v3(
            cpi_context,
            CONSENT_NFT_NAME.to_string(),
            CONSENT_NFT_SYMBOL.to_string(),
            CONSENT_NFT_URI.to_string(),
            None,
            0,
            true,
            true,
            None,
            None,
            None,
        )?;

        // Create master edition
        let master_edition_cpi_context = CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(),
            create_master_edition_v3::cpi::accounts::CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.authority.to_account_info(),
                mint_authority: ctx.accounts.authority.to_account_info(),
                payer: ctx.accounts.authority.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        create_master_edition_v3(
            master_edition_cpi_context,
            Some(0), // Max supply of 0 means unlimited
        )?;

        Ok(())
    }

    pub fn issue_consent_nft(
        ctx: Context<IssueConsentNFT>,
        consent_version: String,
        consent_hash: String,
    ) -> Result<()> {
        let participant = &mut ctx.accounts.participant;
        let study = &mut ctx.accounts.study;

        // Mint the consent NFT to the participant
        let mint_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        );

        token::mint_to(mint_ctx, 1)?;

        // Update participant's consent status
        participant.has_active_consent = true;
        participant.consent_issued_at = Clock::get()?.unix_timestamp;

        // Update study's consent count
        study.consented_participants = study.consented_participants.checked_add(1).unwrap();

        Ok(())
    }

    pub fn revoke_consent(ctx: Context<RevokeConsent>) -> Result<()> {
        let participant = &mut ctx.accounts.participant;
        let study = &mut ctx.accounts.study;

        // Burn the consent NFT
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.participant.to_account_info(),
            },
        );

        token::burn(burn_ctx, 1)?;

        // Update participant's consent status
        participant.has_active_consent = false;
        participant.consent_revoked_at = Some(Clock::get()?.unix_timestamp);

        // Update study's consent count
        study.consented_participants = study.consented_participants.checked_sub(1).unwrap();

        Ok(())
    }

    pub fn update_participant_profile(
        ctx: Context<UpdateParticipantProfile>,
        interests: Vec<String>,
    ) -> Result<()> {
        let participant = &mut ctx.accounts.participant;
        let wallet = &ctx.accounts.wallet;
        let phantom_address = ctx.accounts.phantom_wallet.key();

        // Verify Phantom wallet ownership
        require!(
            wallet.verify_phantom_ownership(&phantom_address),
            WalletError::InvalidPhantomWallet
        );

        participant.profile.interests = interests;
        participant.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn track_study_progress(
        ctx: Context<TrackStudyProgress>,
        progress: u8,
    ) -> Result<()> {
        require!(progress <= 100, ParticipantError::InvalidProgress);
        
        let participant = &mut ctx.accounts.participant;
        let study = &mut ctx.accounts.study;
        
        // Find the active study
        let active_study = participant.profile.active_studies
            .iter_mut()
            .find(|s| s.study_id == study.key())
            .ok_or(ParticipantError::StudyNotFound)?;
        
        active_study.progress = progress;
        active_study.last_update = Clock::get()?.unix_timestamp;
        
        // Update study analytics if progress is 100%
        if progress == 100 {
            study.analytics.completion_rate = (study.analytics.completion_rate + 1) as f32 / 
                study.analytics.total_participants as f32;
        }
        
        Ok(())
    }

    pub fn submit_study_feedback(
        ctx: Context<SubmitStudyFeedback>,
        rating: u8,
        feedback: Option<String>,
    ) -> Result<()> {
        require!(rating >= 1 && rating <= 5, ParticipantError::InvalidRating);
        if let Some(feedback) = &feedback {
            require!(feedback.len() <= 500, ParticipantError::FeedbackTooLong);
        }
        
        let participant = &mut ctx.accounts.participant;
        let study = &mut ctx.accounts.study;
        
        // Find the completed study
        let completed_study = participant.profile.completed_studies
            .iter_mut()
            .find(|s| s.study_id == study.key())
            .ok_or(ParticipantError::StudyNotFound)?;
        
        completed_study.rating = Some(rating);
        completed_study.feedback = feedback;
        
        // Update study analytics
        study.analytics.average_rating = (study.analytics.average_rating * 
            study.analytics.total_participants as f32 + rating as f32) / 
            (study.analytics.total_participants + 1) as f32;
        
        // Update participant's reputation score
        participant.profile.reputation_score = participant.profile.calculate_reputation_score();
        
        Ok(())
    }

    pub fn update_interests(
        ctx: Context<UpdateInterests>,
        interests: Vec<String>,
    ) -> Result<()> {
        let participant = &mut ctx.accounts.participant;
        participant.profile.interests = interests;
        Ok(())
    }

    pub fn initialize_phantom_wallet(
        ctx: Context<InitializePhantomWallet>,
    ) -> Result<()> {
        let wallet = &mut ctx.accounts.wallet;
        let phantom_address = ctx.accounts.phantom_wallet.key();
        
        // Verify Phantom wallet ownership
        require!(
            ctx.accounts.authority.key() == phantom_address,
            WalletError::InvalidPhantomWallet
        );

        wallet.initialize(
            phantom_address,
            ctx.accounts.main_token_account.key(),
            ctx.accounts.privacy_key_account.key(),
        )?;

        // Initialize participant with anonymous ID
        let participant = &mut ctx.accounts.participant;
        participant.anonymous_id = wallet.anonymous_id.clone();
        participant.phantom_address = phantom_address;
        participant.created_at = Clock::get()?.unix_timestamp;
        participant.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn receive_reward(
        ctx: Context<ReceiveReward>,
        amount: u64,
    ) -> Result<()> {
        let wallet = &mut ctx.accounts.wallet;
        let study = &mut ctx.accounts.study;
        let phantom_address = ctx.accounts.phantom_wallet.key();

        // Verify Phantom wallet ownership
        require!(
            wallet.verify_phantom_ownership(&phantom_address),
            WalletError::InvalidPhantomWallet
        );

        // Transfer tokens from researcher to participant's Phantom wallet
        MetaplexToken::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MetaplexToken::Transfer {
                    from: ctx.accounts.researcher_token_account.to_account_info(),
                    to: ctx.accounts.main_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;

        // Record the reward transaction with anonymous ID
        wallet.add_reward(study.key(), amount)?;
        wallet.update_transaction_status(
            format!("{:x}", u64::from_le_bytes(study.key().to_bytes()[0..8].try_into().unwrap())),
            TransactionStatus::Completed,
        )?;

        Ok(())
    }

    /// Initializes the admin account
    /// This must be called first to set up the program's admin
    pub fn initialize_admin(ctx: Context<InitializeAdmin>) -> Result<()> {
        let admin = &mut ctx.accounts.admin;
        admin.authority = ctx.accounts.payer.key();
        admin.dashboard = AdminDashboard::default();
        admin.verified_researchers = 0;
        admin.pending_verifications = 0;
        admin.last_updated = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }

    /// Verifies a researcher's credentials
    /// Can only be called by the admin
    pub fn verify_researcher(ctx: Context<ReviewResearcherVerification>) -> Result<()> {
        let admin = &mut ctx.accounts.admin;
        let researcher = &mut ctx.accounts.researcher;
        
        require!(admin.authority == ctx.accounts.authority.key(), AdminError::UnauthorizedAccess);
        
        admin.verify_researcher(researcher)?;
        Ok(())
    }

    /// Rejects a researcher's application
    /// Can only be called by the admin
    pub fn reject_researcher(ctx: Context<ReviewResearcherVerification>) -> Result<()> {
        let admin = &mut ctx.accounts.admin;
        let researcher = &mut ctx.accounts.researcher;
        
        require!(admin.authority == ctx.accounts.authority.key(), AdminError::UnauthorizedAccess);
        
        admin.reject_researcher(researcher)?;
        Ok(())
    }

    /// Updates a study's status
    /// Can only be called by the admin
    pub fn update_study_status(ctx: Context<UpdateStudyStatus>, status: StudyStatus) -> Result<()> {
        let admin = &mut ctx.accounts.admin;
        let study = &mut ctx.accounts.study;
        
        require!(admin.authority == ctx.accounts.authority.key(), AdminError::UnauthorizedAccess);
        
        admin.update_study_status(study, status)?;
        Ok(())
    }

    /// Manages participant status
    /// Can only be called by the admin
    pub fn manage_participant(ctx: Context<ManageParticipantStatus>, action: ParticipantAction) -> Result<()> {
        let admin = &mut ctx.accounts.admin;
        let participant = &mut ctx.accounts.participant;
        
        require!(admin.authority == ctx.accounts.authority.key(), AdminError::UnauthorizedAccess);
        
        admin.manage_participant(participant, action)?;
        Ok(())
    }
}

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
pub struct ReceiveReward<'info> {
    #[account(mut)]
    pub wallet: Account<'info, ParticipantWallet>,
    #[account(mut)]
    pub study: Account<'info, Study>,
    #[account(mut)]
    pub researcher_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub main_token_account: Account<'info, TokenAccount>,
    pub phantom_wallet: Signer<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
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

#[error_code]
pub enum RecruSearchError {
    #[msg("Study is inactive")]
    StudyInactive,
    #[msg("Study is full")]
    StudyFull,
    #[msg("Unauthorized researcher")]
    UnauthorizedResearcher,
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
    #[msg("Invalid Phantom wallet")]
    InvalidPhantomWallet,
    #[msg("Invalid progress value")]
    InvalidProgress,
    #[msg("Invalid rating value")]
    InvalidRating,
    #[msg("Feedback too long")]
    FeedbackTooLong,
    #[msg("Study not found")]
    StudyNotFound,
}

#[error_code]
pub enum AdminError {
    #[msg("Unauthorized access")]
    UnauthorizedAccess,
}