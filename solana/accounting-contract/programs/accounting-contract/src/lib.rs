use anchor_lang::prelude::*;

declare_id!("31weKQJQA9ZFYaVdnjtAXUoEGPW8UUFC5TEvgPJugAub");

#[program]
pub mod accounting_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
