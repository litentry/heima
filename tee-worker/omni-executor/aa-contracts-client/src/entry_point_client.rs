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

use crate::types::{
	createAccountCall, depositToCall, getSenderAddressCall, getUserOpHashCall, handleOpsCall,
	simulateValidationCall, SenderAddressResult, ValidationResult,
};
use crate::utils::{
	build_call_transaction, build_payable_transaction, calculate_omni_account_address,
};
use crate::{OmniAccountClient, PackedUserOperation};
use alloy::hex;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::rpc::types::state::AccountOverride;
use alloy::rpc::types::TransactionRequest;
use alloy::sol_types::{SolCall, SolError, SolValue};
use ethereum_rpc::RpcProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

/// EntryPointSimulations deployed bytecode embedded at compile time
const SIMULATION_BYTECODE: &str =
	include_str!("bytecode/EntryPointSimulations_deployed_bytecode.hex");

/// Client for interacting with on-chain EntryPoint instance
pub struct EntryPointClient<P: RpcProvider<Transaction = TransactionRequest>> {
	entry_point_address: Address,
	rpc_client: Arc<P>,
}

impl<P: RpcProvider<Transaction = TransactionRequest, Addr = Address>> EntryPointClient<P> {
	pub fn new(entry_point_address: Address, rpc_client: Arc<P>) -> Self {
		Self { entry_point_address, rpc_client }
	}

	pub fn entry_point_address(&self) -> Address {
		self.entry_point_address
	}

	pub async fn get_wallet_address(&self) -> Result<Address, ()> {
		self.rpc_client.get_wallet_address().await
	}

	/// Simulate user operation validation using EntryPointSimulations contract
	/// This function uses state override to temporarily deploy the simulation contract
	pub async fn simulate_validation(
		&self,
		user_op: PackedUserOperation,
	) -> Result<ValidationResult, ()> {
		// Create state override to deploy simulation contract at EntryPoint address
		let mut state_override = HashMap::new();
		state_override.insert(
			self.entry_point_address,
			AccountOverride {
				code: Some(hex::decode(SIMULATION_BYTECODE.trim()).map_err(|_| ())?.into()),
				..Default::default()
			},
		);

		// Build call to simulateValidation
		let call_data = simulateValidationCall { userOp: user_op }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);

		// Make the call with state override
		// EntryPointSimulations.simulateValidation() returns ValidationResult on success
		match self.rpc_client.call_with_state_override(tx, state_override).await {
			Ok(result) => {
				// Decode the ValidationResult from the successful response
				ValidationResult::abi_decode(&result).map_err(|_| {
					error!("Could not decode ValidationResult from response");
				})
			},
			Err(Some(revert_data)) => {
				// In some cases, the simulation might revert with ValidationResult data
				ValidationResult::abi_decode(&revert_data).map_err(|_| {
					error!("Could not decode ValidationResult from revert data");
				})
			},
			Err(None) => {
				error!("Simulation failed with no data");
				Err(())
			},
		}
	}

	pub async fn handle_ops(
		&self,
		user_ops: &[PackedUserOperation],
		beneficiary: Address,
	) -> Result<String, ()> {
		let ops = user_ops.to_vec();
		let call_data = handleOpsCall { ops, beneficiary }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);
		self.rpc_client
			.send_transaction(tx)
			.await
			.map_err(|_| error!("Could not send tx"))
	}

	pub async fn get_sender_address(&self, init_code: Bytes) -> Result<Address, ()> {
		let call_data = getSenderAddressCall { initCode: init_code }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);
		match self.rpc_client.call(tx).await {
			Err(e) => {
				if let Some(bytes) = e {
					let result = SenderAddressResult::abi_decode(&bytes)
						.map_err(|_| error!("Could not decode SenderAddressResult"))?;
					Ok(result.sender)
				} else {
					Err(())
				}
			},
			Ok(_) => Err(()),
		}
	}

	/// Calculate sender address locally using CREATE2 without calling EntryPoint
	/// This is more efficient as it doesn't require an RPC call
	pub fn calculate_sender_address(
		&self,
		factory_address: Address,
		account_implementation: Address,
		oa: FixedBytes<32>,
		client_id: &[u8],
		root: Address,
	) -> Address {
		calculate_omni_account_address(factory_address, account_implementation, oa, client_id, root)
	}

	pub async fn deposit_to(&self, account: Address, amount: U256) -> Result<String, ()> {
		let call_data = depositToCall { account }.abi_encode();
		let tx = build_payable_transaction(self.entry_point_address, call_data, amount);
		self.rpc_client
			.send_transaction(tx)
			.await
			.map_err(|_| error!("Could not send tx"))
	}

	pub async fn get_user_op_hash(
		&self,
		user_op: PackedUserOperation,
	) -> Result<FixedBytes<32>, ()> {
		let call_data = getUserOpHashCall { userOp: user_op }.abi_encode();
		let tx = build_call_transaction(self.entry_point_address, call_data);
		let result = self.rpc_client.call(tx).await.map_err(|_| error!("Could not send tx"))?;
		let hash: FixedBytes<32> = FixedBytes::abi_decode(&result).unwrap();
		Ok(hash)
	}

	#[allow(clippy::too_many_arguments)]
	pub async fn create_packed_user_operation(
		&self,
		factory_address: Address,
		oa: [u8; 32],
		client_id: &[u8],
		root_address: Address,
		call_data: Bytes,
		paymaster_address: Option<Address>,
	) -> Result<PackedUserOperation, ()> {
		// Create init code using existing helper
		let init_code_bytes =
			prepare_factory_init_code(factory_address, oa, client_id, root_address);
		let init_code = Bytes::from(init_code_bytes);

		// Get sender address from EntryPoint
		let sender = self.get_sender_address(init_code).await?;

		// Use common helper for the rest
		self.build_packed_user_operation(
			sender,
			factory_address,
			oa,
			client_id,
			root_address,
			call_data,
			paymaster_address,
		)
		.await
	}

	/// Create PackedUserOperation using local CREATE2 address calculation
	/// This is more efficient as it avoids the EntryPoint.getSenderAddress RPC call
	#[allow(clippy::too_many_arguments)]
	pub async fn create_packed_user_operation_with_local_address(
		&self,
		factory_address: Address,
		account_implementation: Address,
		oa: [u8; 32],
		client_id: &[u8],
		root_address: Address,
		call_data: Bytes,
		paymaster_address: Option<Address>,
	) -> Result<PackedUserOperation, ()> {
		let oa_fixed = FixedBytes::from(oa);

		// Calculate sender address locally using CREATE2
		let sender = self.calculate_sender_address(
			factory_address,
			account_implementation,
			oa_fixed,
			client_id,
			root_address,
		);

		// Use common helper for the rest
		self.build_packed_user_operation(
			sender,
			factory_address,
			oa,
			client_id,
			root_address,
			call_data,
			paymaster_address,
		)
		.await
	}

	/// Common helper to build PackedUserOperation with shared logic
	#[allow(clippy::too_many_arguments)]
	async fn build_packed_user_operation(
		&self,
		sender: Address,
		factory_address: Address,
		oa: [u8; 32],
		client_id: &[u8],
		root_address: Address,
		call_data: Bytes,
		paymaster_address: Option<Address>,
	) -> Result<PackedUserOperation, ()> {
		// Check if the account already has code deployed
		let code = self.rpc_client.get_code_at(sender).await.map_err(|_| ())?;

		// Get nonce - if smart account doesn't exist yet, use 0
		let nonce = if code.is_empty() {
			// Smart account doesn't exist yet, use 0 as nonce
			U256::from(0)
		} else {
			// Smart account exists, get nonce from contract
			let smart_account_client = OmniAccountClient::new(sender, self.rpc_client.clone());
			smart_account_client.get_nonce().await?
		};

		let init_code_to_use = if code.is_empty() {
			// No code at address, include init code
			let init_code_bytes =
				prepare_factory_init_code(factory_address, oa, client_id, root_address);
			Bytes::from(init_code_bytes)
		} else {
			// Code already exists, no init code needed
			Bytes::new()
		};

		// Default gas limits - can be adjusted based on requirements
		let verification_gas_limit = U256::from(250000u64); // Higher for verification
		let call_gas_limit = U256::from(50000u64); // Lower for simple calls
		let account_gas_limits = create_account_gas_limits(verification_gas_limit, call_gas_limit);

		let pre_verification_gas = U256::from(21000u64);
		let max_fee_per_gas = U256::from(50000000000u64); // 50 gwei - increased for mainnet
		let max_priority_fee_per_gas = U256::from(2000000000u64); // 2 gwei
		let gas_fees = create_gas_fees(max_fee_per_gas, max_priority_fee_per_gas);

		let paymaster_and_data = if let Some(paymaster_addr) = paymaster_address {
			create_paymaster_and_data(paymaster_addr, U256::from(50000u64), U256::from(50000u64))
		} else {
			Bytes::new()
		};

		Ok(PackedUserOperation {
			sender,
			nonce,
			initCode: init_code_to_use,
			callData: call_data,
			accountGasLimits: account_gas_limits,
			preVerificationGas: pre_verification_gas,
			gasFees: gas_fees,
			paymasterAndData: paymaster_and_data,
			signature: Bytes::new(),
		})
	}
}

#[allow(dead_code)]
pub fn prepare_factory_init_code(
	factory_address: Address,
	oa: [u8; 32],
	client_id: &[u8],
	root: Address,
) -> Vec<u8> {
	let mut init_code = vec![];

	let mut call = createAccountCall {
		//safe to unwrap
		oa: oa.into(),
		//safe to unwrap
		clientId: client_id.to_owned().into(),
		root,
	}
	.abi_encode();
	init_code.append(&mut factory_address.to_vec());
	init_code.append(&mut call);

	init_code
}

pub fn create_paymaster_and_data(
	paymaster_address: Address,
	verification_gas_limit: U256,
	post_op_gas_limit: U256,
) -> Bytes {
	let mut paymaster_and_data = Vec::new();
	paymaster_and_data.extend_from_slice(paymaster_address.as_slice()); // 20 bytes
	paymaster_and_data.extend_from_slice(&verification_gas_limit.to_be_bytes::<32>()[16..]); // 16 bytes
	paymaster_and_data.extend_from_slice(&post_op_gas_limit.to_be_bytes::<32>()[16..]); // 16 bytes
	paymaster_and_data.into()
}

pub fn create_account_gas_limits(
	verification_gas_limit: U256,
	call_gas_limit: U256,
) -> FixedBytes<32> {
	let mut gas_limits = [0u8; 32];
	// First 16 bytes: verification gas limit
	gas_limits[0..16].copy_from_slice(&verification_gas_limit.to_be_bytes::<32>()[16..]);
	// Last 16 bytes: call gas limit
	gas_limits[16..32].copy_from_slice(&call_gas_limit.to_be_bytes::<32>()[16..]);
	FixedBytes::from(gas_limits)
}

pub fn create_gas_fees(max_fee_per_gas: U256, max_priority_fee_per_gas: U256) -> FixedBytes<32> {
	let mut gas_fees = [0u8; 32];
	// First 16 bytes: max priority fee per gas
	gas_fees[0..16].copy_from_slice(&max_priority_fee_per_gas.to_be_bytes::<32>()[16..]);
	// Last 16 bytes: max fee per gas
	gas_fees[16..32].copy_from_slice(&max_fee_per_gas.to_be_bytes::<32>()[16..]);
	FixedBytes::from(gas_fees)
}

#[cfg(test)]
pub mod test {
	use crate::types::depositCall;
	use crate::utils::build_payable_transaction;
	use crate::{prepare_factory_init_code, EntryPointClient};
	use alloy::hex;
	use alloy::network::EthereumWallet;
	use alloy::primitives::{address, Bytes, FixedBytes, U256};
	use alloy::signers::local::PrivateKeySigner;
	use alloy::signers::Signer;
	use alloy::sol_types::SolCall;
	use ethereum_rpc::mocks::MockRpcProvider;
	use ethereum_rpc::{AlloyRpcProvider, RpcProvider};
	use heima_primitives::{AccountId, Identity, Web2IdentityType};
	use std::str::FromStr;
	use std::sync::Arc;
	use test_log::test;

	#[test(tokio::test)]
	pub async fn test_get_sender_address() {
		let expected_sender = address!("0x5dfec187c82986cf670f4e2ed6de1cd001cee5be");
		let client_id = "test_client";
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let root_address = address!("0x0000000000000000000000000000000000000001");
		let mut rpc_client = MockRpcProvider::new();

		rpc_client.expect_call()
            .with(mockall::predicate::always())
            .times(1)
            .returning(|_| {
                Err(Some(hex::decode("0x6ca7b8060000000000000000000000005dfec187c82986cf670f4e2ed6de1cd001cee5be").unwrap()))
            });

		let entrypoint_client = EntryPointClient::new(entrypoint_address, Arc::new(rpc_client));

		let oa: AccountId =
			Identity::Evm(user_address.0.as_slice().try_into().unwrap()).to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let client_id_fixed_bytes = Bytes::from(client_id_bytes);
		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());
		let init_code_bytes = prepare_factory_init_code(
			factory_address,
			oa_bytes.0,
			client_id_fixed_bytes.as_ref(),
			root_address,
		);

		let init_code = Bytes::from(init_code_bytes.to_vec());
		let sender = entrypoint_client.get_sender_address(init_code.clone()).await.unwrap();

		assert_eq!(expected_sender, sender);
	}

	#[test(test)]
	pub fn test_prepare_init_code() {
		let expected = "e7f1725e7734ce288f8367e1bb143e90bb3f05120db21afe6d659c2361df3754aa7d46d9f0c993ec0335e60508128c6e3843f7dd962113910000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";

		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let oa: [u8; 32] =
			hex::decode("0x6d659c2361df3754aa7d46d9f0c993ec0335e60508128c6e3843f7dd96211391")
				.unwrap()
				.try_into()
				.unwrap();
		let client_id: Vec<u8> =
			hex::decode("0x0000000000000000000000000000000000000000000000000000000000000000")
				.unwrap();
		let root_address = address!("0x0000000000000000000000000000000000000001");

		let init_code = prepare_factory_init_code(factory_address, oa, &client_id, root_address);

		assert_eq!(expected, hex::encode(init_code));
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_get_sender_address() {
		let expected_sender = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let client_id = "test_client";
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let root_address = address!("0x0000000000000000000000000000000000000001");
		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let entrypoint_client = EntryPointClient::new(entrypoint_address, rpc_client);
		// first 20 bytes factory address, then call data which should be oa + client_id + root_address encode packed
		let oa: AccountId =
			Identity::Evm(user_address.0.as_slice().try_into().unwrap()).to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let client_id_fixed_bytes = Bytes::from(client_id_bytes);
		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());
		let init_code_bytes = prepare_factory_init_code(
			factory_address,
			oa_bytes.0,
			&client_id_fixed_bytes,
			root_address,
		);

		let init_code = Bytes::from(init_code_bytes.to_vec());
		let sender = entrypoint_client.get_sender_address(init_code.clone()).await.unwrap();

		assert_eq!(expected_sender, sender);
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_full_flow() {
		let user_signer = PrivateKeySigner::from_str(
			"0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
		)
		.unwrap();
		let client_id = "heima";
		// calculate from wallet
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0xe7f1725e7734ce288f8367e1bb143e90bb3f0512");
		let factory_address = address!("0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0");
		let root_address = user_address;
		// This should be the address where SimplePaymaster contract is deployed
		let paymaster_address = address!("0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9");
		let signer = PrivateKeySigner::from_str(
			"0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
		)
		.unwrap();
		let wallet = EthereumWallet::new(signer);
		let rpc_client =
			Arc::new(AlloyRpcProvider::new_with_wallet("http://localhost:8545", wallet));
		let entrypoint_client = EntryPointClient::new(entrypoint_address, rpc_client);
		// first 20 bytes factory address, then call data which should be oa + client_id + root_address encode packed
		let oa: AccountId =
			Identity::from_web2_account("user@example.com", Web2IdentityType::Email)
				.to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let client_id_fixed_bytes = Bytes::from(client_id_bytes);

		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());
		let init_code_bytes = prepare_factory_init_code(
			factory_address,
			oa_bytes.0,
			&client_id_fixed_bytes,
			root_address,
		);

		// Create PackedUserOperation using the utility function
		let call_data = Bytes::from(init_code_bytes.to_vec()); // Use init_code as call_data for account creation

		let mut user_op = entrypoint_client
			.create_packed_user_operation(
				factory_address,
				oa_bytes.0,
				&client_id_fixed_bytes,
				root_address,
				call_data.clone(),
				None,
			)
			.await
			.unwrap();

		println!("Sender address: {:?}", user_op.sender);
		println!("Init code: {:?}", hex::encode(init_code_bytes));
		println!("Call data: {:?}", hex::encode(call_data));

		let user_op_hash = entrypoint_client.get_user_op_hash(user_op.clone()).await.unwrap();
		let signature = user_signer.sign_hash(&user_op_hash).await.unwrap();

		// Prepend 0x00 byte to indicate Owner signature type (according to UserOpSigner enum)
		let mut signature_with_prefix: Vec<u8> = vec![0x01];
		signature_with_prefix.extend_from_slice(&signature.as_bytes());
		user_op.signature = signature_with_prefix.into();

		// Fund the paymaster with ETH deposits to EntryPoint
		let paymaster_deposit_call = depositCall {}.abi_encode();
		let paymaster_deposit_tx = build_payable_transaction(
			paymaster_address,
			paymaster_deposit_call,
			U256::from_str("1000000000000000000").unwrap(), // 1 ETH
		);
		entrypoint_client
			.rpc_client
			.send_transaction(paymaster_deposit_tx)
			.await
			.unwrap();

		// Test simulate_validation before executing the user operation
		let _validation_result = entrypoint_client
			.simulate_validation(user_op.clone())
			.await
			.expect("simulate_validation should succeed");

		// Execute user operation with paymaster sponsorship
		let tx_hash =
			entrypoint_client.handle_ops(&vec![user_op], entrypoint_address).await.unwrap();
		println!("Transaction hash: {}", tx_hash);
	}

	/// Integration test to verify that local CREATE2 calculation matches EntryPoint.getSenderAddress
	///
	/// To run this test:
	/// 1. Deploy contracts using: `cd aa-contracts && ./local-deploy.sh`
	/// 2. Run: `cargo test test_local_vs_entrypoint_address_calculation -- --ignored`
	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn test_local_vs_entrypoint_address_calculation() {
		let client_id = "test_client";
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let account_implementation = address!("0xCafaC3dD18aC6c6e92c921884f9E4176737C052C");
		let root_address = address!("0x0000000000000000000000000000000000000001");

		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let entrypoint_client = EntryPointClient::new(entrypoint_address, rpc_client);

		// Create test parameters
		let oa: AccountId =
			Identity::Evm(user_address.0.as_slice().try_into().unwrap()).to_omni_account(client_id);
		let client_id_bytes = client_id.as_bytes();
		let oa_bytes: FixedBytes<32> = FixedBytes::from_slice(oa.as_ref());

		// Calculate address using EntryPoint.getSenderAddress
		let init_code_bytes =
			prepare_factory_init_code(factory_address, oa_bytes.0, client_id_bytes, root_address);
		let init_code = Bytes::from(init_code_bytes);
		let entrypoint_address_result = entrypoint_client
			.get_sender_address(init_code)
			.await
			.expect("EntryPoint address calculation should succeed");

		// Calculate address using local CREATE2 calculation
		let local_address_result = entrypoint_client.calculate_sender_address(
			factory_address,
			account_implementation,
			oa_bytes,
			client_id_bytes,
			root_address,
		);

		// Assert that all three methods return the same address
		assert_eq!(entrypoint_address_result, local_address_result,);
	}
}
