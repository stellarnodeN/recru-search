use anchor_lang::prelude::*;
use crate::contexts::*;

pub fn register_participant(ctx: Context<RegisterParticipant>, eligibility_proof: String) -> Result<()> {
    let participant = &mut ctx.accounts.participant;
    participant.create(ctx.accounts.authority.key(), eligibility_proof)?;
    Ok(())
}

pub fn update_participant_profile(ctx: Context<UpdateParticipantProfile>, interests: Vec<String>) -> Result<()> {
    let participant = &mut ctx.accounts.participant;
    participant.profile.interests = interests;
    Ok(())
}

pub fn update_interests(ctx: Context<UpdateInterests>, interests: Vec<String>) -> Result<()> {
    let participant = &mut ctx.accounts.participant;
    participant.profile.interests = interests;
    Ok(())
}