//! Error types for the RecruSearch program
//! 
//! This module defines all possible error conditions that can occur during program execution.
//! Each error includes a descriptive message to help with debugging and user feedback.

use anchor_lang::prelude::*;

/// Custom error types for the RecruSearch program
#[error_code]
pub enum RecruSearchError {
    /// Attempted to interact with a study that is not currently active
    #[msg("Study is not active")]
    StudyInactive,

    /// Attempted to join a study that has reached its maximum participant capacity
    #[msg("Study has reached maximum participant capacity")]
    StudyFull,

    /// Attempted to perform a researcher-only action without proper authorization
    #[msg("Unauthorized researcher access")]
    UnauthorizedResearcher,

    /// Attempted to perform an admin-only action without proper authorization
    #[msg("Unauthorized admin access")]
    UnauthorizedAdmin,

    /// Attempted to transfer or use an invalid token amount
    #[msg("Invalid token amount")]
    InvalidTokenAmount,

    /// Participant does not meet the eligibility criteria for a study
    #[msg("Participant not eligible for study")]
    ParticipantNotEligible,

    /// Attempted to interact with a study that does not exist
    #[msg("Study not found")]
    StudyNotFound,

    /// Attempted to perform an action requiring verified researcher status
    #[msg("Researcher not verified")]
    ResearcherNotVerified,

    /// Attempted to perform an action with invalid consent status
    #[msg("Invalid consent status")]
    InvalidConsentStatus,

    /// Token transfer operation failed
    #[msg("Token transfer failed")]
    TokenTransferFailed,

    /// Attempted to initialize an account that is already initialized
    #[msg("Account already initialized")]
    AlreadyInitialized,

    /// Participant is in an invalid state for the requested operation
    #[msg("Invalid participant status")]
    InvalidParticipantStatus,

    /// Study completion requirements have not been met
    #[msg("Study completion criteria not met")]
    StudyCompletionCriteriaNotMet,

    /// Specified reward amount is invalid
    #[msg("Invalid reward amount")]
    InvalidRewardAmount,

    /// Attempted to set a maximum participant count that exceeds allowed limits
    #[msg("Maximum participants limit exceeded")]
    MaxParticipantsExceeded,

    /// Study parameters provided are invalid
    #[msg("Invalid study parameters")]
    InvalidStudyParameters,

    /// Attempted to register a participant that is already registered
    #[msg("Participant already registered")]
    ParticipantAlreadyRegistered,

    /// Attempted to register a researcher that is already registered
    #[msg("Researcher already registered")]
    ResearcherAlreadyRegistered,

    /// Invalid progress value provided
    #[msg("Invalid progress value")]
    InvalidProgress,

    /// Invalid rating value provided
    #[msg("Invalid rating value")]
    InvalidRating,

    /// Feedback text is too long
    #[msg("Feedback too long")]
    FeedbackTooLong,
} 