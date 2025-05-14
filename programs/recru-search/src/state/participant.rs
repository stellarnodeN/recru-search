use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ParticipantProfile {
    pub age_group: String,
    pub gender: String,
    pub region: String,
    pub interests: Vec<String>,
    pub is_anonymous: bool,
    pub completed_studies: Vec<CompletedStudy>,
    pub active_studies: Vec<ActiveStudy>,
    pub reward_history: Vec<RewardRecord>,
    pub reputation_score: u32,
}

#[account]
pub struct Participant {
    pub authority: Pubkey,
    pub profile: ParticipantProfile,
    pub eligibility_proof: String,
    pub registered_at: i64,
    pub suspended: bool,
    pub banned: bool,
    pub active_studies: u32,
    pub completed_studies: u32,
    pub has_active_consent: bool,
    pub consent_issued_at: i64,
    pub consent_revoked_at: Option<i64>,
    pub wallet: Option<Pubkey>,
    pub reputation_score: u32,
    pub last_activity: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompletedStudy {
    pub study_id: Pubkey,
    pub completion_date: i64,
    pub reward_amount: u64,
    pub rating: Option<u8>,
    pub feedback: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ActiveStudy {
    pub study_id: Pubkey,
    pub start_date: i64,
    pub progress: u8,
    pub last_update: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardRecord {
    pub study_id: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub transaction_id: String,
}

impl Participant {
    pub fn create(
        &mut self,
        authority: Pubkey,
        eligibility_proof: String,
    ) -> Result<()> {
        self.authority = authority;
        self.eligibility_proof = eligibility_proof;
        self.registered_at = Clock::get()?.unix_timestamp;
        self.active_studies = 0;
        self.completed_studies = 0;
        self.has_active_consent = false;
        self.consent_issued_at = 0;
        self.consent_revoked_at = None;
        Ok(())
    }
    pub fn update_profile(&mut self, profile: ParticipantProfile) -> Result<()> {
        self.profile = profile;
        Ok(())
    }
    pub fn increment_active_studies(&mut self) -> Result<()> {
        self.active_studies = self.active_studies.checked_add(1)
            .ok_or(crate::error::RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }
    pub fn decrement_active_studies(&mut self) -> Result<()> {
        self.active_studies = self.active_studies.checked_sub(1)
            .ok_or(crate::error::RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }
    pub fn increment_completed_studies(&mut self) -> Result<()> {
        self.completed_studies = self.completed_studies.checked_add(1)
            .ok_or(crate::error::RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }
    pub fn update_consent_status(&mut self, has_consent: bool) -> Result<()> {
        self.has_active_consent = has_consent;
        if has_consent {
            self.consent_issued_at = Clock::get()?.unix_timestamp;
            self.consent_revoked_at = None;
        } else {
            self.consent_revoked_at = Some(Clock::get()?.unix_timestamp);
        }
        Ok(())
    }
}

impl ParticipantProfile {
    pub fn calculate_reputation_score(&self) -> u32 {
        let base_score = self.completed_studies.len() as u32 * 10;
        let rating_bonus: u32 = self.completed_studies
            .iter()
            .filter_map(|study| study.rating)
            .map(|rating| rating as u32)
            .sum();
        base_score + rating_bonus
    }
    pub fn is_eligible_for_study(&self, _study: &crate::state::study::Study) -> bool {
        // Basic eligibility criteria
        let criteria = [
            // Age group check
            true, // Replace with actual age group check when implemented
            // Reputation score check - assuming a minimum reputation score requirement
            // This is a placeholder as the Study struct doesn't have min_reputation_score
            self.reputation_score >= 10, // Use a default minimum score of 10
        ];
        criteria.iter().all(|&check| check == true)
    }
}

#[error_code]
pub enum ParticipantError {
    #[msg("Invalid progress update")]
    InvalidProgress,
    #[msg("Study not found in active studies")]
    StudyNotFound,
    #[msg("Invalid rating value")]
    InvalidRating,
    #[msg("Feedback too long")]
    FeedbackTooLong,
    #[msg("Maximum active studies reached")]
    MaxActiveStudiesReached,
    #[msg("Insufficient reputation score")]
    InsufficientReputation,
} 