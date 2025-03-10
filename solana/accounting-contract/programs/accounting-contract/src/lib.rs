use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("4d4sEXZXa3BHfEpggwQNGmRniLn71YckNzqzErwhFjwJ");

#[program]
pub mod accounting_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn set_admin(ctx: Context<SetAdmin>, new_admin: Pubkey) -> Result<()> {
        let upgrade_authority = ctx
            .accounts
            .program_account
            .upgrade_authority_address
            .unwrap();
        require!(
            ctx.accounts.caller.key() == upgrade_authority,
            ErrorCode::Unauthorized
        );
        ctx.accounts.admin_account.admin = new_admin;
        Ok(())
    }

    pub fn set_worker(ctx: Context<SetWorker>, new_worker: Pubkey) -> Result<()> {
        require!(
            ctx.accounts.admin_account.admin == ctx.accounts.caller.key(),
            ErrorCode::Unauthorized
        );
        ctx.accounts.worker_account.worker = new_worker;
        Ok(())
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, amount: u64) -> Result<()> {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.depositor.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn create_pay_request(
        ctx: Context<CreatePayRequest>,
        amount: u64,
        nonce: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.worker.worker == ctx.accounts.signer.key(),
            ErrorCode::Unauthorized
        );

        require!(
            ctx.accounts.treasury.balance >= amount,
            ErrorCode::OutOfBalance
        );

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.treasury.to_account_info(),
                    to: ctx.accounts.beneficiary.to_account_info(),
                },
            ),
            amount,
        )?;

        let payout_request = &mut ctx.accounts.pay_out_request;
        payout_request.beneficiary = *ctx.accounts.beneficiary.key;
        payout_request.amount = amount;
        payout_request.nonce = nonce;

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct AdminAccount {
    pub admin: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct WorkerAccount {
    pub worker: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct TreasuryAccount {
    pub balance: u64,
}

#[account]
#[derive(InitSpace)]
pub struct PayoutRequest {
    pub beneficiary: Pubkey,
    pub amount: u64,
    pub nonce: u64,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct CreatePayRequest<'info> {
    #[account(init_if_needed, payer = signer, space = 8 + PayoutRequest::INIT_SPACE, seeds = [beneficiary.to_account_info().key().to_bytes().as_ref(), &nonce.to_be_bytes(), b"payout_request"], bump)]
    pub pay_out_request: Account<'info, PayoutRequest>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(seeds = [b"worker"], bump)]
    pub worker: Account<'info, WorkerAccount>,
    #[account(mut, seeds = [b"treasury"], bump)]
    pub treasury: Account<'info, TreasuryAccount>,
    #[account(mut)]
    pub beneficiary: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositFunds<'info> {
    #[account(init_if_needed, payer = depositor, space = 8 + TreasuryAccount::INIT_SPACE, seeds = [b"treasury"], bump)]
    pub treasury: Account<'info, TreasuryAccount>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetWorker<'info> {
    #[account(init_if_needed, payer = caller, space = 8 + WorkerAccount::INIT_SPACE, seeds = [b"worker"], bump)]
    pub worker_account: Account<'info, WorkerAccount>,
    #[account(mut)]
    pub caller: Signer<'info>,
    #[account(seeds = [b"admin"], bump)]
    pub admin_account: Account<'info, AdminAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(init_if_needed, payer = caller, space = 8 + AdminAccount::INIT_SPACE, seeds = [b"admin"], bump)]
    pub admin_account: Account<'info, AdminAccount>,
    #[account(mut)]
    pub caller: Signer<'info>,
    pub program_account: Account<'info, ProgramData>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Initialize {}

// ✅ Error handling
#[error_code]
pub enum ErrorCode {
    #[msg("Caller is not the Upgrade Authority.")]
    Unauthorized,
    #[msg("Invalid program account.")]
    InvalidProgram,
    #[msg("Could not retrieve upgrade authority.")]
    UpgradeAuthorityNotFound,
    #[msg("The program has run out of Sol tokens")]
    OutOfBalance,
}
