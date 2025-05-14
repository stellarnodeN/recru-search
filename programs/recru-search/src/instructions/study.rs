use anchor_lang::prelude::*;
use crate::contexts::*;
use crate::state::study::StudyType;
use crate::state::admin::StudyStatus;

pub fn join_study(ctx: Context<JoinStudy>) -> Result<()> {
    let study = &mut ctx.accounts.study;
    let participant = &mut ctx.accounts.participant;
    study.can_accept_participants()?;
    study.add_participant()?;
    participant.increment_active_studies()?;
    Ok(())
}

pub fn create_study(
    ctx: Context<CreateStudy>,
    title: String,
    description: String,
    criteria_hash: String,
    reward_amount: u64,
    max_participants: u32,
) -> Result<()> {
    let study = &mut ctx.accounts.study;
    let researcher = &ctx.accounts.researcher;
    study.create(
        researcher.key(),
        title,
        description,
        criteria_hash,
        reward_amount,
        max_participants,
        StudyType::Survey, // Or pass as argument if needed
    )?;
    Ok(())
}

pub fn complete_study(ctx: Context<CompleteStudy>) -> Result<()> {
    let study = &mut ctx.accounts.study;
    let participant = &mut ctx.accounts.participant;
    let researcher = &mut ctx.accounts.researcher;
    study.complete_participant()?;
    participant.increment_completed_studies()?;
    researcher.update_total_participants(1)?;
    Ok(())
}

pub fn track_study_progress(ctx: Context<TrackStudyProgress>, progress: u8) -> Result<()> {
    let study = &mut ctx.accounts.study;
    study.update_progress(progress)?;
    Ok(())
}

pub fn submit_study_feedback(
    ctx: Context<SubmitStudyFeedback>,
    rating: u8,
    feedback: Option<String>,
) -> Result<()> {
    let study = &mut ctx.accounts.study;
    study.submit_feedback(rating, feedback)?;
    Ok(())
}

pub fn update_study_status(ctx: Context<UpdateStudyStatus>, status: StudyStatus) -> Result<()> {
    let study = &mut ctx.accounts.study;
    study.status = status;
    Ok(())
}