use anchor_lang::prelude::*;
use crate::contexts::{InitializeAdmin, ReviewResearcherVerification, UpdateStudyStatus, ManageParticipantStatus};
use crate::state::admin::{StudyStatus, ParticipantAction};

pub fn initialize_admin(ctx: Context<InitializeAdmin>) -> Result<()> {
    let admin = &mut ctx.accounts.admin;
    admin.authority = ctx.accounts.payer.key();
    // Initialize dashboard and other fields as needed (already handled by Admin::new if used)
    Ok(())
}

pub fn verify_researcher(ctx: Context<ReviewResearcherVerification>) -> Result<()> {
    let admin = &ctx.accounts.admin;
    let researcher = &mut ctx.accounts.researcher;
    require!(admin.authority == ctx.accounts.authority.key(), crate::error::RecruSearchError::UnauthorizedAdmin);
    researcher.is_verified = true;
    Ok(())
}

pub fn reject_researcher(ctx: Context<ReviewResearcherVerification>) -> Result<()> {
    let admin = &ctx.accounts.admin;
    let researcher = &mut ctx.accounts.researcher;
    require!(admin.authority == ctx.accounts.authority.key(), crate::error::RecruSearchError::UnauthorizedAdmin);
    researcher.is_verified = false;
    // Optionally set a rejected flag or remove from pending list
    Ok(())
}

pub fn update_study_status(ctx: Context<UpdateStudyStatus>, status: StudyStatus) -> Result<()> {
    let admin = &ctx.accounts.admin;
    let study = &mut ctx.accounts.study;
    require!(admin.authority == ctx.accounts.authority.key(), crate::error::RecruSearchError::UnauthorizedAdmin);
    study.status = status;
    Ok(())
}

pub fn manage_participant(ctx: Context<ManageParticipantStatus>, action: ParticipantAction) -> Result<()> {
    let admin = &ctx.accounts.admin;
    let participant = &mut ctx.accounts.participant;
    require!(admin.authority == ctx.accounts.authority.key(), crate::error::RecruSearchError::UnauthorizedAdmin);
    match action {
        ParticipantAction::Suspend => participant.suspended = true,
        ParticipantAction::Unsuspend => participant.suspended = false,
        ParticipantAction::Ban => participant.banned = true,
    }
    Ok(())
}