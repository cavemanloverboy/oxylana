use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod oxylana {
    use super::*;

    pub fn sign_demo(ctx: Context<SignDemo>) -> Result<()> {
        ctx.accounts.rust_station.oxidized = true;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SignDemo<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        space = 9,
        payer = user,
        seeds = [
            user.key().as_ref(),
            b"ferris the crab".as_ref(),
            b"oxylana".as_ref(),
            b"rust enjoyoooor".as_ref(),
        ],
        bump
    )]
    pub rust_station: Account<'info, RustStation>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct RustStation {
    pub oxidized: bool,
}

impl RustStation {
    pub fn get_pda(user: &Pubkey) -> Pubkey {
        let seeds = &[
            user.as_ref(),
            b"ferris the crab".as_ref(),
            b"oxylana".as_ref(),
            b"rust enjoyoooor".as_ref(),
        ];
        Pubkey::find_program_address(seeds, &crate::ID).0
    }
}
