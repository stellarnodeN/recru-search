use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AdminDashboard {
    pub total_studies: u32,
    pub active_studies: u32,
    pub total_participants: u32,
    pub total_researchers: u32,
    pub total_rewards_distributed: u64,
    pub study_categories: Vec<CategoryStats>,
    pub platform_metrics: PlatformMetrics,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CategoryStats {
    pub category: String,
    pub study_count: u32,
    pub participant_count: u32,
    pub average_rating: f32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlatformMetrics {
    pub average_completion_rate: f32,
    pub average_study_duration: i64,
    pub participant_retention_rate: f32,
    pub researcher_satisfaction_rate: f32,
}

#[account]
pub struct Admin {
    pub authority: Pubkey,
    pub dashboard: AdminDashboard,
    pub verified_researchers: u32,
    pub pending_verifications: u32,
    pub last_updated: i64,
    pub study_status: StudyStatus,
    pub participant_action: ParticipantAction,
}

#[derive(Accounts)]
pub struct InitializeAdmin<'info> {
    #[account(init, payer = payer, space = 8 + 1000)]
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
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateStudyStatus<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub study: Account<'info, Study>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ManageParticipantStatus<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    pub authority: Signer<'info>,
}

impl Admin {
    pub fn new(authority: Pubkey) -> Self {
        Self {
            authority,
            dashboard: AdminDashboard {
                total_studies: 0,
                active_studies: 0,
                total_participants: 0,
                total_researchers: 0,
                total_rewards_distributed: 0,
                study_categories: Vec::new(),
                platform_metrics: PlatformMetrics {
                    average_completion_rate: 0.0,
                    average_study_duration: 0,
                    participant_retention_rate: 0.0,
                    researcher_satisfaction_rate: 0.0,
                },
            },
            verified_researchers: 0,
            pending_verifications: 0,
            last_updated: Clock::get().unwrap().unix_timestamp,
        }
    }

    pub fn verify_researcher(&mut self, researcher: &Account<Researcher>) -> Result<()> {
        require!(researcher.credentials_verified == false, AdminError::AlreadyVerified);
        
        researcher.credentials_verified = true;
        self.verified_researchers += 1;
        self.pending_verifications -= 1;
        self.last_updated = Clock::get().unwrap().unix_timestamp;
        
        Ok(())
    }

    pub fn reject_researcher(&mut self, researcher: &Account<Researcher>) -> Result<()> {
        require!(researcher.credentials_verified == false, AdminError::AlreadyVerified);
        
        researcher.credentials_verified = false;
        self.pending_verifications -= 1;
        self.last_updated = Clock::get().unwrap().unix_timestamp;
        
        Ok(())
    }

    pub fn update_study_status(&mut self, study: &Account<Study>, status: StudyStatus) -> Result<()> {
        study.status = status;
        
        match status {
            StudyStatus::Active => self.dashboard.active_studies += 1,
            StudyStatus::Inactive => self.dashboard.active_studies -= 1,
            _ => {}
        }
        
        self.last_updated = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }

    pub fn manage_participant(&mut self, participant: &Account<Participant>, action: ParticipantAction) -> Result<()> {
        match action {
            ParticipantAction::Suspend => participant.is_suspended = true,
            ParticipantAction::Unsuspend => participant.is_suspended = false,
            ParticipantAction::Ban => participant.is_banned = true,
        }
        
        self.last_updated = Clock::get().unwrap().unix_timestamp;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ParticipantAction {
    Suspend,
    Unsuspend,
    Ban,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum StudyStatus {
    Active,
    Inactive,
    Completed,
    Suspended,
}

#[derive(Accounts)]
pub struct UpdateAdminDashboard<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyResearcher<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub researcher: Account<'info, Researcher>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReviewStudy<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub study: Account<'info, Study>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ManageParticipant<'info> {
    #[account(mut)]
    pub admin: Account<'info, Admin>,
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    pub authority: Signer<'info>,
}

impl AdminDashboard {
    pub fn update_metrics(&mut self, study: &super::Study) {
        // Update category stats
        if let Some(category_stats) = self.study_categories
            .iter_mut()
            .find(|stats| stats.category == study.category) {
            category_stats.study_count += 1;
            category_stats.participant_count += study.current_participants;
            category_stats.average_rating = (category_stats.average_rating * 
                category_stats.study_count as f32 + study.analytics.average_rating) / 
                (category_stats.study_count + 1) as f32;
        } else {
            self.study_categories.push(CategoryStats {
                category: study.category.clone(),
                study_count: 1,
                participant_count: study.current_participants,
                average_rating: study.analytics.average_rating,
            });
        }

        // Update platform metrics
        self.platform_metrics.average_completion_rate = 
            (self.platform_metrics.average_completion_rate * self.total_studies as f32 + 
            study.analytics.completion_rate) / (self.total_studies + 1) as f32;
        
        self.platform_metrics.average_study_duration = 
            (self.platform_metrics.average_study_duration * self.total_studies as i64 + 
            study.duration_days as i64) / (self.total_studies + 1) as i64;
    }

    pub fn calculate_participant_retention(&self, participant: &super::Participant) -> f32 {
        let completed_studies = participant.profile.completed_studies.len() as f32;
        let total_studies = (participant.profile.completed_studies.len() + 
            participant.profile.active_studies.len()) as f32;
        
        if total_studies == 0.0 {
            0.0
        } else {
            completed_studies / total_studies
        }
    }
}

#[error_code]
pub enum AdminError {
    #[msg("Unauthorized access")]
    UnauthorizedAccess,
    #[msg("Already verified")]
    AlreadyVerified,
    #[msg("Invalid action")]
    InvalidAction,
    #[msg("Study not found")]
    StudyNotFound,
    #[msg("Participant not found")]
    ParticipantNotFound,
}