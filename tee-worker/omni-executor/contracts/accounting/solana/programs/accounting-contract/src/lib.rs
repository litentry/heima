#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use anchor_lang::system_program;
use serde::{Deserialize, Serialize};

declare_id!("EJgprV3h3RECRWtsMnee6FUD8CT5AUk3nMFXb7PA4xYP");

#[program]
pub mod accounting_contract {
	use super::*;

	pub fn set_admin(ctx: Context<SetAdmin>, new_admin: Pubkey) -> Result<()> {
		let upgrade_authority = match ctx.accounts.program_account.upgrade_authority_address {
			Some(address) => address,
			None => return Err(ErrorCode::UpgradeAuthorityNotFound.into()),
		};

		#[cfg(not(feature = "test-skip-auth"))]
		require!(ctx.accounts.signer.key() == upgrade_authority, ErrorCode::Unauthorized);

		let admin = &mut ctx.accounts.admin_account.admin;
		*admin = new_admin;
		Ok(())
	}

	pub fn set_worker(ctx: Context<SetWorker>, new_worker: Pubkey) -> Result<()> {
		require!(
			ctx.accounts.admin_account.admin == ctx.accounts.signer.key(),
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
					from: ctx.accounts.signer.to_account_info(),
					to: ctx.accounts.treasury.to_account_info(),
				},
			),
			amount,
		)?;
		Ok(())
	}

	pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
		require!(
			ctx.accounts.admin_account.admin == ctx.accounts.signer.key(),
			ErrorCode::Unauthorized
		);

		require!(
			ctx.accounts.treasury.to_account_info().lamports() >= amount,
			ErrorCode::OutOfBalance
		);

		**ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? -= amount;
		**ctx.accounts.beneficiary.to_account_info().try_borrow_mut_lamports()? += amount;

		Ok(())
	}

	pub fn create_pay_request(
		ctx: Context<CreatePayRequest>,
		nonce: u64,
		amount: u64,
	) -> Result<()> {
		require!(ctx.accounts.worker.worker == ctx.accounts.signer.key(), ErrorCode::Unauthorized);

		require!(
			ctx.accounts.treasury.to_account_info().lamports() >= amount,
			ErrorCode::OutOfBalance
		);

		require!(ctx.accounts.account_nonce.nonce + 1 == nonce, ErrorCode::InvalidNonce);

		**ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? -= amount;
		**ctx.accounts.beneficiary.to_account_info().try_borrow_mut_lamports()? += amount;

		let payout_request = &mut ctx.accounts.pay_out_request;
		payout_request.beneficiary = *ctx.accounts.beneficiary.key;
		payout_request.amount = amount;
		payout_request.nonce = nonce;

		let nonce = &mut ctx.accounts.account_nonce.nonce;
		*nonce += 1;

		Ok(())
	}
}

#[account]
#[derive(InitSpace, Serialize, Deserialize, Debug)]
pub struct AdminAccount {
	pub admin: Pubkey,
}

#[account]
#[derive(InitSpace, Serialize, Deserialize, Debug)]
pub struct WorkerAccount {
	pub worker: Pubkey,
}

#[account]
#[derive(InitSpace, Serialize, Deserialize, Debug)]
pub struct TreasuryAccount {}

#[account]
#[derive(InitSpace, Serialize, Deserialize, Debug)]
pub struct PayoutRequest {
	pub beneficiary: Pubkey,
	pub nonce: u64,
	pub amount: u64,
}

#[account]
#[derive(InitSpace, Serialize, Deserialize, Debug, bincode::Encode, bincode::Decode)]
pub struct Nonce {
	pub nonce: u64,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct CreatePayRequest<'info> {
	#[account(init, payer = signer, space = 8 + PayoutRequest::INIT_SPACE, seeds = [beneficiary.to_account_info().key().to_bytes().as_ref(), nonce.to_le_bytes().as_ref(), b"payout_request"], bump)]
	pub pay_out_request: Account<'info, PayoutRequest>,
	#[account(init_if_needed, payer = signer, space = 8 + Nonce::INIT_SPACE, seeds = [beneficiary.to_account_info().key().to_bytes().as_ref(), b"nonce"], bump)]
	pub account_nonce: Account<'info, Nonce>,
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
	#[account(init_if_needed, payer = signer, space = 8 + TreasuryAccount::INIT_SPACE, seeds = [b"treasury"], bump)]
	pub treasury: Account<'info, TreasuryAccount>,
	#[account(mut)]
	pub signer: Signer<'info>,
	pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
	#[account(mut)]
	pub signer: Signer<'info>,
	#[account(mut, seeds = [b"admin"], bump)]
	pub admin_account: Account<'info, AdminAccount>,
	#[account(mut, seeds = [b"treasury"], bump)]
	pub treasury: Account<'info, TreasuryAccount>,
	pub beneficiary: SystemAccount<'info>,
	pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetWorker<'info> {
	#[account(init_if_needed, payer = signer, space = 8 + WorkerAccount::INIT_SPACE, seeds = [b"worker"], bump)]
	pub worker_account: Account<'info, WorkerAccount>,
	#[account(mut)]
	pub signer: Signer<'info>,
	#[account(seeds = [b"admin"], bump)]
	pub admin_account: Account<'info, AdminAccount>,
	pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
	#[account(init_if_needed, payer = signer, space = 8 + AdminAccount::INIT_SPACE, seeds = [b"admin"], bump)]
	pub admin_account: Account<'info, AdminAccount>,
	#[account(mut)]
	pub signer: Signer<'info>,
	pub program_account: Account<'info, ProgramData>,
	pub system_program: Program<'info, System>,
}

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
	#[msg("The nonce is invalid")]
	InvalidNonce,
}
