//! RecruSearch - A Solana-based platform for research study recruitment and management

use anchor_lang::prelude::*;

pub mod contexts;
pub mod error;
pub mod instructions;
pub mod state;


use crate::contexts::*;
use crate::instructions::*;
//use crate::contexts::{InitializePrivacyManager, GrantDataAccess, RevokeDataAccess};
use crate::state::admin::{StudyStatus, ParticipantAction};

declare_id!("BkXcFAo2TFkXRm9WsKUxikgNYvvR3Pm3yS9xLdqaeJoo");

#[program]
pub mod recru_search {
    use super::*;
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn initialize_psypoints(_ctx: Context<InitializePsyPoints>) -> Result<()> {
        Ok(())
    }

    pub fn create_token_account(_ctx: Context<CreateTokenAccount>) -> Result<()> {
        Ok(())
    }

    pub fn register_researcher(
        ctx: Context<RegisterResearcher>,
        institution: String,
        credentials_hash: String,
    ) -> Result<()> {
        instructions::researcher::register_researcher(ctx, institution, credentials_hash)
    }

    pub fn create_study(
        ctx: Context<CreateStudy>,
        title: String,
        description: String,
        criteria_hash: String,
        reward_amount: u64,
        max_participants: u32,
    ) -> Result<()> {
        instructions::study::create_study(ctx, title, description, criteria_hash, reward_amount, max_participants)
    }

    pub fn register_participant(
        ctx: Context<RegisterParticipant>,
        eligibility_proof: String,
    ) -> Result<()> {
        instructions::participant::register_participant(ctx, eligibility_proof)
    }

    pub fn join_study(ctx: Context<JoinStudy>) -> Result<()> {
        instructions::study::join_study(ctx)
    }

    pub fn complete_study(ctx: Context<CompleteStudy>) -> Result<()> {
        instructions::study::complete_study(ctx)
    }

    pub fn verify_researcher(ctx: Context<ReviewResearcherVerification>) -> Result<()> {
        instructions::admin::verify_researcher(ctx)
    }

    pub fn initialize_consent_nft(ctx: Context<InitializeConsentNFT>) -> Result<()> {
        instructions::consent::initialize_consent_nft(ctx)
    }

    pub fn issue_consent_nft(
        ctx: Context<IssueConsentNFT>,
        consent_version: String,
        consent_hash: String,
    ) -> Result<()> {
        instructions::consent::issue_consent_nft(ctx, consent_version, consent_hash)
    }

    pub fn revoke_consent(ctx: Context<RevokeConsent>) -> Result<()> {
        instructions::consent::revoke_consent(ctx)
    }

    pub fn update_participant_profile(
        ctx: Context<UpdateParticipantProfile>,
        interests: Vec<String>,
    ) -> Result<()> {
        instructions::participant::update_participant_profile(ctx, interests)
    }

    pub fn track_study_progress(
        ctx: Context<TrackStudyProgress>,
        progress: u8,
    ) -> Result<()> {
        instructions::study::track_study_progress(ctx, progress)
    }

    pub fn submit_study_feedback(
        ctx: Context<SubmitStudyFeedback>,
        rating: u8,
        feedback: Option<String>,
    ) -> Result<()> {
        instructions::study::submit_study_feedback(ctx, rating, feedback)
    }

    pub fn update_interests(
        ctx: Context<UpdateInterests>,
        interests: Vec<String>,
    ) -> Result<()> {
        instructions::participant::update_interests(ctx, interests)
    }

    pub fn initialize_phantom_wallet(ctx: Context<InitializePhantomWallet>) -> Result<()> {
        instructions::wallet::initialize_phantom_wallet(ctx)
    }

    pub fn receive_reward(ctx: Context<ReceiveReward>, amount: u64) -> Result<()> {
        instructions::wallet::receive_reward(ctx, amount)
    }

    pub fn update_wallet_metadata(
        ctx: Context<UpdateWalletMetadata>,
        metadata_uri: Option<String>,
    ) -> Result<()> {
        instructions::wallet::update_wallet_metadata(ctx, metadata_uri)
    }

    pub fn initialize_admin(ctx: Context<InitializeAdmin>) -> Result<()> {
        instructions::admin::initialize_admin(ctx)
    }

    pub fn reject_researcher(ctx: Context<ReviewResearcherVerification>) -> Result<()> {
        instructions::admin::reject_researcher(ctx)
    }

    pub fn update_study_status(
        ctx: Context<UpdateStudyStatus>,
        status: StudyStatus,
    ) -> Result<()> {
        instructions::admin::update_study_status(ctx, status)
    }

    pub fn manage_participant(
        ctx: Context<ManageParticipantStatus>,
        action: ParticipantAction,
    ) -> Result<()> {
        instructions::admin::manage_participant(ctx, action)
    }

    // Privacy manager functions
    pub fn initialize_privacy_manager(ctx: Context<InitializePrivacyManager>) -> Result<()> {
        instructions::privacy::initialize_privacy_manager(ctx)
    }
    pub fn grant_data_access(ctx: Context<GrantDataAccess>) -> Result<()> {
        instructions::privacy::grant_data_access(ctx)
    }
    pub fn revoke_data_access(ctx: Context<RevokeDataAccess>) -> Result<()> {
        instructions::privacy::revoke_data_access(ctx)
    }
}
