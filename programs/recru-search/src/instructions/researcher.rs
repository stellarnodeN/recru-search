use anchor_lang::prelude::*;
use crate::contexts::*;

pub fn register_researcher(ctx: Context<RegisterResearcher>, institution: String, credentials_hash: String) -> Result<()> {
    let researcher = &mut ctx.accounts.researcher;
    researcher.authority = ctx.accounts.authority.key();
    researcher.institution = institution;
    researcher.credentials_hash = credentials_hash;
    researcher.is_verified = false;
    researcher.registered_at = Clock::get()?.unix_timestamp;
    Ok(())
}