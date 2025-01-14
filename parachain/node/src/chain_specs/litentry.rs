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

use super::*;
use cumulus_primitives_core::ParaId;
use litentry_parachain_runtime::{
	AccountId, AuraId, Balance, BalancesConfig, CouncilMembershipConfig,
	DeveloperCommitteeMembershipConfig, OmniBridgeConfig, ParachainInfoConfig,
	ParachainStakingConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig,
	TechnicalCommitteeMembershipConfig, TeebagConfig, TeebagOperationalMode, VCManagementConfig,
	WASM_BINARY,
};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::Deserialize;
use sp_core::sr25519;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Get default parachain properties for Litentry which will be filled into chain spec
fn default_parachain_properties() -> Properties {
	parachain_properties("LIT", 18, 31)
}

const DEFAULT_ENDOWED_ACCOUNT_BALANCE: Balance = 1000 * UNIT;

/// GenesisInfo struct to store the parsed genesis_info JSON
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GenesisInfo {
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<(AccountId, String)>,
	council: Vec<AccountId>,
	technical_committee: Vec<AccountId>,
	developer_committee: Vec<AccountId>,
	boot_nodes: Vec<String>,
	telemetry_endpoints: Vec<String>,
}

pub fn get_chain_spec_dev() -> ChainSpec {
	ChainSpec::builder(
		WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions { relay_chain: "rococo-local".into(), para_id: LITENTRY_PARA_ID },
	)
	.with_name("Litentry-dev")
	.with_id("litentry-dev")
	.with_protocol_id("litentry")
	.with_chain_type(ChainType::Development)
	.with_properties(default_parachain_properties())
	.with_genesis_config(generate_genesis(
		vec![(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_collator_keys_from_seed("Alice"),
		)],
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				6 * DEFAULT_ENDOWED_ACCOUNT_BALANCE,
			),
			(get_account_id_from_seed::<sr25519::Public>("Bob"), DEFAULT_ENDOWED_ACCOUNT_BALANCE),
			(
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				DEFAULT_ENDOWED_ACCOUNT_BALANCE,
			),
			(get_account_id_from_seed::<sr25519::Public>("Eve"), DEFAULT_ENDOWED_ACCOUNT_BALANCE),
		],
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
		],
		vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
		vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
		LITENTRY_PARA_ID.into(),
	))
	.build()
}

pub fn get_chain_spec_staging() -> ChainSpec {
	// Staging keys are derivative keys based on a single master secret phrase:
	//
	// root: 	$SECRET
	// account:	$SECRET//collator//<id>
	// aura: 	$SECRET//collator//<id>//aura
	get_chain_spec_from_genesis_info(
		include_bytes!("../../res/genesis_info/staging.json"),
		"Litentry-staging",
		"litentry-staging",
		ChainType::Local,
		"rococo-local".into(),
		LITENTRY_PARA_ID.into(),
	)
}

pub fn get_chain_spec_prod() -> ChainSpec {
	get_chain_spec_from_genesis_info(
		include_bytes!("../../res/genesis_info/litentry.json"),
		"Litentry",
		"litentry",
		ChainType::Live,
		"polkadot".into(),
		LITENTRY_PARA_ID.into(),
	)
}

/// Private function to get a ChainSpec from a `genesis_info_json_file`,
/// used in both staging and prod env.
fn get_chain_spec_from_genesis_info(
	genesis_info_bytes: &[u8],
	name: &str,
	id: &str,
	chain_type: ChainType,
	relay_chain_name: String,
	para_id: ParaId,
) -> ChainSpec {
	let genesis_info: GenesisInfo =
		serde_json::from_slice(genesis_info_bytes).expect("Invalid GenesisInfo; qed.");

	let boot_nodes = genesis_info.boot_nodes.clone();
	let telemetry_endpoints = genesis_info.telemetry_endpoints.clone();
	let genesis_info_cloned = genesis_info.clone();

	use std::str::FromStr;

	ChainSpec::builder(
		WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions { relay_chain: relay_chain_name, para_id: para_id.into() },
	)
	.with_name(name)
	.with_id(id)
	.with_chain_type(chain_type)
	.with_protocol_id("litentry")
	.with_properties(default_parachain_properties())
	.with_boot_nodes(
		boot_nodes
			.into_iter()
			.map(|k| k.parse().expect("Wrong bootnode format; qed."))
			.collect(),
	)
	.with_telemetry_endpoints(
		TelemetryEndpoints::new(
			telemetry_endpoints
				.into_iter()
				.map(|k| (k, 0)) // 0 is verbose level
				.collect(),
		)
		.expect("Invalid telemetry URL; qed."),
	)
	.with_genesis_config(generate_genesis(
		genesis_info_cloned.invulnerables,
		genesis_info_cloned
			.endowed_accounts
			.into_iter()
			.map(|(k, b)| (k, u128::from_str(&b).expect("Bad endowed balance; qed.")))
			.collect(),
		genesis_info_cloned.council,
		genesis_info_cloned.technical_committee,
		genesis_info_cloned.developer_committee,
		para_id,
	))
	.build()
}

fn generate_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	council_members: Vec<AccountId>,
	technical_committee_members: Vec<AccountId>,
	developer_committee_members: Vec<AccountId>,
	id: ParaId,
) -> serde_json::Value {
	let config = RuntimeGenesisConfig {
		system: Default::default(),
		balances: BalancesConfig { balances: endowed_accounts },
		parachain_info: ParachainInfoConfig { parachain_id: id, ..Default::default() },
		parachain_staking: ParachainStakingConfig {
			// Should be enough for both Litentry and rococo
			candidates: invulnerables.iter().cloned().map(|(acc, _)| (acc, 5000 * UNIT)).collect(),
			..Default::default()
		},
		session: SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                      // account id
						acc,                                              // validator id
						litentry_parachain_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		democracy: Default::default(),
		council: Default::default(),
		council_membership: CouncilMembershipConfig {
			members: council_members.try_into().expect("error convert to BoundedVec"),
			phantom: Default::default(),
		},
		technical_committee: Default::default(),
		technical_committee_membership: TechnicalCommitteeMembershipConfig {
			members: technical_committee_members.try_into().expect("error convert to BoundedVec"),
			phantom: Default::default(),
		},
		developer_committee: Default::default(),
		developer_committee_membership: DeveloperCommitteeMembershipConfig {
			members: developer_committee_members.try_into().expect("error convert to BoundedVec"),
			phantom: Default::default(),
		},
		treasury: Default::default(),
		vesting: Default::default(),
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		vc_management: VCManagementConfig { admin: None },
		transaction_payment: Default::default(),
		assets: Default::default(),
		ethereum: Default::default(),
		evm: Default::default(),
		teebag: TeebagConfig {
			allow_sgx_debug_mode: true,
			admin: None,
			mode: TeebagOperationalMode::Development,
		},
		score_staking: Default::default(),
		omni_bridge: OmniBridgeConfig { admin: None, default_relayers: Default::default() },
	};

	serde_json::to_value(&config).expect("Could not build genesis config")
}
