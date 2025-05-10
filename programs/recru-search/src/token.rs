use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint};
use metaplex_core::token::{Token as MetaplexToken, TokenAccount as MetaplexTokenAccount};

pub const PSYPOINTS_DECIMALS: u8 = 9;
pub const PSYPOINTS_NAME: &str = "PsyPoints";
pub const PSYPOINTS_SYMBOL: &str = "PSY";

#[derive(Accounts)]
pub struct InitializePsyPoints<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = PSYPOINTS_DECIMALS,
        mint::authority = authority,
    )]
    pub mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, MetaplexToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = authority,
    )]
    pub token_account: Account<'info, MetaplexTokenAccount>,
    
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
} 