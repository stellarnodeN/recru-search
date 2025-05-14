use anchor_lang::prelude::*;

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

impl Admin {
    pub fn is_authorized(&self, authority: &Pubkey) -> bool {
        &self.authority == authority
    }
    
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
            study_status: StudyStatus::Inactive,
            participant_action: ParticipantAction::Unsuspend,
        }
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

impl AdminDashboard {
    pub fn update_metrics(&mut self, study: &crate::state::study::Study) {
        if let Some(category_stats) = self.study_categories
            .iter_mut()
            .find(|stats| stats.category == study.title) {
            category_stats.study_count += 1;
            category_stats.participant_count += study.current_participants;
            category_stats.average_rating = (category_stats.average_rating * 
                category_stats.study_count as f32 + study.analytics.average_rating) / 
                (category_stats.study_count + 1) as f32;
        } else {
            self.study_categories.push(CategoryStats {
                category: study.title.clone(),
                study_count: 1,
                participant_count: study.current_participants,
                average_rating: study.analytics.average_rating,
            });
        }
        self.platform_metrics.average_completion_rate = 
            (self.platform_metrics.average_completion_rate * self.total_studies as f32 + 
            study.analytics.completion_rate) / (self.total_studies + 1) as f32;
        self.platform_metrics.average_study_duration = 
            (self.platform_metrics.average_study_duration * self.total_studies as i64 + 
            0) / (self.total_studies + 1) as i64;
    }
    pub fn calculate_participant_retention(&self, participant: &crate::state::participant::Participant) -> f32 {
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