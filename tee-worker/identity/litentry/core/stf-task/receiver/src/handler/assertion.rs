// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

#![allow(clippy::result_large_err)]

use crate::{EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use ita_sgx_runtime::Hash;
use ita_stf::{Getter, TrustedCallSigned};
use itp_stf_executor::traits::StfEnclaveSigning;
use itp_top_pool_author::traits::AuthorApi;
use itp_utils::stringify::account_id_to_string;
use lc_credentials::credential_schema;
use lc_data_providers::DataProviderConfig;
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use litentry_primitives::{
	AmountHoldingTimeType, Assertion, AssertionBuildRequest, ErrorDetail, ErrorString, Identity,
	ParameterString, VCMPError,
};
use log::*;
use sp_core::{ed25519::Pair as Ed25519Pair, Pair, H160};
use std::{
	format,
	iter::once,
	string::{String, ToString},
	sync::Arc,
	vec::Vec,
};

fn build_holding_time(
	req: &AssertionBuildRequest,
	htype: AmountHoldingTimeType,
	min_balance: ParameterString,
	data_provider_config: &DataProviderConfig,
) -> Result<lc_credentials::Credential, VCMPError> {
	lc_assertion_build::holding_time::build(req, htype, min_balance, data_provider_config)
}

pub fn create_credential_str<
	A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter>,
	S: StfEnclaveSigning<TrustedCallSigned>,
	O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem>,
>(
	req: &AssertionBuildRequest,
	enclave_signer: Arc<S>,
	enclave_account: Arc<Ed25519Pair>,
	ocall_api: Arc<O>,
	data_provider_config: Arc<DataProviderConfig>,
	assertion_repository: Arc<AR>,
) -> Result<(Vec<u8>, Option<Vec<u8>>), VCMPError> {
	let mut vc_logs: Option<Vec<String>> = None;
	let mut credential = match req.assertion.clone() {
		Assertion::A1 => {
			#[cfg(test)]
			{
				std::thread::sleep(core::time::Duration::from_secs(5));
			}
			lc_assertion_build::a1::build(req)
		},
		Assertion::A2(guild_id) =>
			lc_assertion_build::a2::build(req, guild_id, &data_provider_config),

		Assertion::A3(guild_id, channel_id, role_id) =>
			lc_assertion_build::a3::build(req, guild_id, channel_id, role_id, &data_provider_config),

		Assertion::A4(min_balance) =>
			build_holding_time(req, AmountHoldingTimeType::LIT, min_balance, &data_provider_config),

		Assertion::A6 => lc_assertion_build::a6::build(req, &data_provider_config),

		Assertion::A7(min_balance) =>
			build_holding_time(req, AmountHoldingTimeType::DOT, min_balance, &data_provider_config),

		// no need to pass `networks` again because it's the same as the `get_supported_web3networks`
		Assertion::A8(_networks) => lc_assertion_build::a8::build(req, &data_provider_config),

		Assertion::A10(min_balance) =>
			build_holding_time(req, AmountHoldingTimeType::WBTC, min_balance, &data_provider_config),

		Assertion::A11(min_balance) =>
			build_holding_time(req, AmountHoldingTimeType::ETH, min_balance, &data_provider_config),

		Assertion::A13(owner) => lc_assertion_build::a13::build(req, ocall_api, &owner),

		Assertion::A14 => lc_assertion_build::a14::build(req, &data_provider_config),

		Assertion::Achainable(param) =>
			lc_assertion_build::achainable::build(req, param, &data_provider_config),

		Assertion::A20 => lc_assertion_build::a20::build(req, &data_provider_config),

		Assertion::OneBlock(course_type) =>
			lc_assertion_build::oneblock::course::build(req, course_type, &data_provider_config),

		Assertion::GenericDiscordRole(role_type) =>
			lc_assertion_build::generic_discord_role::build(req, role_type, &data_provider_config),

		Assertion::BnbDomainHolding =>
			lc_assertion_build::nodereal::bnb_domain::bnb_domain_holding_amount::build(
				req,
				&data_provider_config,
			),

		Assertion::BnbDigitDomainClub(digit_domain_type) =>
			lc_assertion_build::nodereal::bnb_domain::bnb_digit_domain_club_amount::build(
				req,
				digit_domain_type,
				&data_provider_config,
			),

		Assertion::VIP3MembershipCard(level) =>
			lc_assertion_build::vip3::card::build(req, level, &data_provider_config),

		Assertion::WeirdoGhostGangHolder =>
			lc_assertion_build::nodereal::nft_holder::weirdo_ghost_gang_holder::build(
				req,
				&data_provider_config,
			),

		Assertion::LITStaking => lc_assertion_build::lit_staking::build(req),

		Assertion::EVMAmountHolding(token_type) =>
			lc_assertion_build::nodereal::amount_holding::evm_amount_holding::build(
				req,
				token_type,
				&data_provider_config,
			),

		Assertion::BRC20AmountHolder =>
			lc_assertion_build::brc20::amount_holder::build(req, &data_provider_config),

		Assertion::CryptoSummary =>
			lc_assertion_build::nodereal::crypto_summary::build(req, &data_provider_config),

		Assertion::TokenHoldingAmount(token_type) =>
			lc_assertion_build_v2::token_holding_amount::build(
				req,
				token_type,
				&data_provider_config,
			),

		Assertion::PlatformUser(platform_user_type) => lc_assertion_build_v2::platform_user::build(
			req,
			platform_user_type,
			&data_provider_config,
		),

		Assertion::NftHolder(nft_type) =>
			lc_assertion_build_v2::nft_holder::build(req, nft_type, &data_provider_config),

		Assertion::Dynamic(params) => {
			let result =
				lc_assertion_build::dynamic::build(req, params, assertion_repository, ocall_api)?;
			vc_logs = Some(result.1);
			Ok(result.0)
		},

		Assertion::LinkedIdentities => lc_assertion_build_v2::linked_identities::build(req),
	}?;

	// post-process the credential
	let enclave_signer_account = enclave_signer.get_enclave_account().map_err(|e| {
		VCMPError::RequestVCFailed(
			req.assertion.clone(),
			ErrorDetail::StfError(ErrorString::truncate_from(format!("{e:?}").into())),
		)
	})?;

	credential.parachain_block_number = req.parachain_block_number;
	credential.sidechain_block_number = req.sidechain_block_number;

	credential.credential_subject.endpoint = data_provider_config.credential_endpoint.to_string();

	if let Some(schema) = credential_schema::get_schema_url(&req.assertion) {
		credential.credential_schema.id = schema;
	}

	let issuer_identity: Identity = enclave_account.public().into();
	credential.issuer.id = issuer_identity.to_did().map_err(|e| {
		VCMPError::RequestVCFailed(
			req.assertion.clone(),
			ErrorDetail::StfError(ErrorString::truncate_from(format!("{e:?}").into())),
		)
	})?;

	let json_string = credential
		.to_json()
		.map_err(|_| VCMPError::RequestVCFailed(req.assertion.clone(), ErrorDetail::ParseError))?;
	let payload = json_string.as_bytes();
	let sig = enclave_signer.sign(payload).map_err(|e| {
		VCMPError::RequestVCFailed(
			req.assertion.clone(),
			ErrorDetail::StfError(ErrorString::truncate_from(format!("{e:?}").into())),
		)
	})?;

	credential.add_proof(&sig, account_id_to_string(&enclave_signer_account));
	credential.validate().map_err(|e| {
		VCMPError::RequestVCFailed(
			req.assertion.clone(),
			ErrorDetail::StfError(ErrorString::truncate_from(format!("{e:?}").into())),
		)
	})?;

	let credential_str = credential
		.to_json()
		.map_err(|_| VCMPError::RequestVCFailed(req.assertion.clone(), ErrorDetail::ParseError))?;
	debug!("Credential: {}, length: {}", credential_str, credential_str.len());

	Ok((
		credential_str.as_bytes().to_vec(),
		vc_logs.map(|v| {
			v.iter().flat_map(|s| s.as_bytes().iter().cloned().chain(once(b'\n'))).collect()
		}),
	))
}
