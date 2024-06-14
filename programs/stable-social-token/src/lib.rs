use anchor_lang::prelude::*;

declare_id!("APP6sVJA6zKhnxVfTdEvkvHN9xGUNYxYZy3nQ6DePEAX");

#[program]
pub mod stable_social_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
