use crate::types::{
	createAccountCall, depositToCall, getSenderAddressCall, getUserOpHashCall, handleOpsCall,
	SenderAddressResult,
};
use crate::PackedUserOperation;
use alloy::primitives::bytes::Bytes;
use alloy::primitives::{Address, FixedBytes, TxKind, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::sol_types::{SolCall, SolError, SolValue};
use ethereum_rpc::RpcProvider;
use std::sync::Arc;
use tracing::error;

/// Client for interacting with on-chain EntryPoint instance
pub struct EntryPointClient<P: RpcProvider<Transaction = TransactionRequest>> {
	entry_point_address: Address,
	rpc_client: Arc<P>,
}

impl<P: RpcProvider<Transaction = TransactionRequest>> EntryPointClient<P> {
	pub fn new(entry_point_address: Address, rpc_client: Arc<P>) -> Self {
		Self { entry_point_address, rpc_client }
	}

	pub async fn handle_ops(
		&self,
		user_ops: &[PackedUserOperation],
		beneficiary: Address,
	) -> Result<(), ()> {
		let ops = user_ops.to_vec();
		let call_data = handleOpsCall { ops, beneficiary }.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.entry_point_address)),
			input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
			..Default::default()
		};
		self.rpc_client
			.send_transaction(tx)
			.await
			.map_err(|_| error!("Could not send tx"))?;
		Ok(())
	}

	pub async fn get_sender_address(&self, init_code: Bytes) -> Result<Address, ()> {
		let call_data = getSenderAddressCall { initCode: init_code.into() }.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.entry_point_address)),
			input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
			..Default::default()
		};
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

	pub async fn deposit_to(&self, account: Address, amount: U256) -> Result<(), ()> {
		let call_data = depositToCall { account }.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.entry_point_address)),
			input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
			value: Some(amount),
			..Default::default()
		};
		self.rpc_client
			.send_transaction(tx)
			.await
			.map_err(|_| error!("Could not send tx"))?;
		Ok(())
	}

	pub async fn get_user_op_hash(
		&self,
		user_op: PackedUserOperation,
	) -> Result<FixedBytes<32>, ()> {
		let call_data = getUserOpHashCall { userOp: user_op }.abi_encode();
		let tx = TransactionRequest {
			to: Some(TxKind::Call(self.entry_point_address)),
			input: TransactionInput { data: Some(call_data.into()), ..Default::default() },
			..Default::default()
		};
		let result = self.rpc_client.call(tx).await.map_err(|_| error!("Could not send tx"))?;
		let hash: FixedBytes<32> = FixedBytes::abi_decode(&result).unwrap();
		Ok(hash)
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

#[cfg(test)]
pub mod test {
	use crate::{prepare_factory_init_code, EntryPointClient, PackedUserOperation};
	use alloy::hex;
	use alloy::network::EthereumWallet;
	use alloy::primitives::{address, Bytes, FixedBytes, U256};
	use alloy::signers::local::PrivateKeySigner;
	use alloy::signers::Signer;
	use ethereum_rpc::mocks::MockRpcProvider;
	use ethereum_rpc::AlloyRpcProvider;
	use heima_primitives::{AccountId, Identity};
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

		let init_code = alloy::primitives::bytes::Bytes::from(init_code_bytes.to_vec());
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
				.unwrap()
				.try_into()
				.unwrap();
		let root_address = address!("0x0000000000000000000000000000000000000001");

		let init_code = prepare_factory_init_code(factory_address, oa, &client_id, root_address);

		assert_eq!(expected, hex::encode(init_code));
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_get_sender_address() {
		let expected_sender = address!("0x5dfec187c82986cf670f4e2ed6de1cd001cee5be");
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

		let init_code = alloy::primitives::bytes::Bytes::from(init_code_bytes.to_vec());
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
		let client_id = "test_client";
		// calculate from wallet
		let user_address = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
		let entrypoint_address = address!("0x5FbDB2315678afecb367f032d93F642f64180aa3");
		let factory_address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
		let root_address = address!("0x0000000000000000000000000000000000000001");
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

		let init_code = alloy::primitives::bytes::Bytes::from(init_code_bytes.to_vec());
		let sender = entrypoint_client.get_sender_address(init_code.clone()).await.unwrap();
		println!("Sender address: {:?}", sender);
		let nonce = U256::from(0); // it's fresh account

		let account_gas_limits: FixedBytes<32> = FixedBytes::<32>::from_str(
			"0x0000000000000000000000000003d09000000000000000000000000000005b8d",
		)
		.unwrap();
		let pre_verification_gas = U256::from(21000);
		let gas_fees: FixedBytes<32> = FixedBytes::<32>::from_str(
			"0x0000000000000000000000003b9aca00000000000000000000000000b2d05e00",
		)
		.unwrap();
		let paymaster_and_data: Bytes = Bytes::new();
		let session_account = alloy::primitives::Address::default();
		let session_expiration = U256::from(0);
		let session_account_proof = Bytes::new();
		let signature = Bytes::new();

		let mut user_op = PackedUserOperation {
			sender,
			nonce,
			initCode: init_code.clone().into(),
			callData: init_code.into(),
			accountGasLimits: account_gas_limits,
			preVerificationGas: pre_verification_gas,
			gasFees: gas_fees,
			paymasterAndData: paymaster_and_data,
			sessionAccount: session_account,
			sessionExpiration: session_expiration,
			sessionAccountProof: session_account_proof,
			signature,
		};

		let user_op_hash = entrypoint_client.get_user_op_hash(user_op.clone()).await.unwrap();
		let signature = user_signer.sign_hash(&user_op_hash).await.unwrap();

		user_op.signature = signature.as_bytes().into();

		// use paymaster instead of paying on account own
		// (bool callSuccess,) = address(ep).call{value: 1000000000000000}(abi.encodeCall(ep.depositTo, (to)));

		// deposit some eth
		entrypoint_client
			.deposit_to(sender, U256::from_str("100000000000000000000").unwrap())
			.await
			.unwrap();
		entrypoint_client.handle_ops(&vec![user_op], entrypoint_address).await.unwrap();
	}
}
