use anchor_lang::prelude::*;

#[account]
pub struct Researcher {
    pub authority: Pubkey,
    pub institution: String,
    pub credentials_hash: String,
    pub is_verified: bool,
    pub registered_at: i64,
    pub studies_created: u32,
    pub active_studies: u32,
    pub total_participants: u32,
    pub reputation_score: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ResearcherProfile {
    pub institution: String,
    pub credentials: Vec<Credential>,
    pub is_verified: bool,
    pub created_studies: Vec<Pubkey>,
    pub active_studies: Vec<Pubkey>,
    pub completed_studies: Vec<Pubkey>,
    pub reputation_score: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Credential {
    pub credential_type: String,
    pub issuer: String,
    pub issue_date: i64,
    pub expiry_date: Option<i64>,
    pub verification_status: VerificationStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
}

impl Researcher {
    pub fn create(
        &mut self,
        authority: Pubkey,
        institution: String,
        credentials_hash: String,
    ) -> Result<()> {
        require!(institution.len() > 0, crate::error::RecruSearchError::InvalidInstitutionName);
        require!(credentials_hash.len() > 0, crate::error::RecruSearchError::InvalidCredentials);
        self.authority = authority;
        self.institution = institution;
        self.credentials_hash = credentials_hash;
        self.is_verified = false;
        self.registered_at = Clock::get()?.unix_timestamp;
        self.studies_created = 0;
        self.active_studies = 0;
        self.total_participants = 0;
        self.reputation_score = 0;
        Ok(())
    }
    pub fn increment_studies_created(&mut self) -> Result<()> {
        self.studies_created = self.studies_created.checked_add(1)
            .ok_or(crate::error::RecruSearchError::InvalidStudyParameters)?;
        Ok(())
    }
    pub fn update_active_studies(&mut self, delta: i32) -> Result<()> {
        require!(self.is_verified, crate::error::RecruSearchError::NotVerified);
        if delta > 0 {
            self.active_studies = self.active_studies.checked_add(delta as u32)
                .ok_or(crate::error::RecruSearchError::InvalidStudyParameters)?;
        } else {
            self.active_studies = self.active_studies.checked_sub((-delta) as u32)
                .ok_or(crate::error::RecruSearchError::InvalidStudyParameters)?;
        }
        self.update_reputation_score(
            self.reputation_score + (delta.abs() as u32 * 10)
        )?;
        Ok(())
    }
    pub fn update_total_participants(&mut self, delta: i32) -> Result<()> {
        if delta > 0 {
            self.total_participants = self.total_participants.checked_add(delta as u32)
                .ok_or(crate::error::RecruSearchError::InvalidStudyParameters)?;
        } else {
            self.total_participants = self.total_participants.checked_sub((-delta) as u32)
                .ok_or(crate::error::RecruSearchError::InvalidStudyParameters)?;
        }
        Ok(())
    }
    pub fn update_reputation_score(&mut self, score: u32) -> Result<()> {
        self.reputation_score = score;
        Ok(())
    }
}

impl ResearcherProfile {
    pub fn can_create_study(&self) -> bool {
        self.is_verified && self.reputation_score >= 50
    }
    pub fn update_reputation(&mut self, study: &crate::state::study::Study) {
        let base_score = 10;
        let completion_bonus = (study.analytics.completion_rate * 20.0) as u32;
        let rating_bonus = (study.analytics.average_rating * 10.0) as u32;
        self.reputation_score += base_score + completion_bonus + rating_bonus;
    }
    pub fn verify_credentials(&mut self) -> bool {
        let valid_credentials = self.credentials.iter()
            .filter(|cred| {
                if let Some(expiry) = cred.expiry_date {
                    let now = Clock::get().unwrap().unix_timestamp;
                    now <= expiry
                } else {
                    true
                }
            })
            .count();
        valid_credentials > 0
    }
}

#[error_code]
pub enum ResearcherError {
    #[msg("Researcher not verified")]
    NotVerified,
    #[msg("Maximum studies limit reached")]
    MaxStudiesReached,
    #[msg("Invalid institution information")]
    InvalidInstitution,
    #[msg("Invalid credentials")]
    InvalidCredentials,
    #[msg("Researcher already registered")]
    AlreadyRegistered,
} 