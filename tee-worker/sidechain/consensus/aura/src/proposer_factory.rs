/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

use crate::slot_proposer::{ExternalitiesFor, SlotProposer};
use codec::Encode;
use finality_grandpa::BlockNumberOps;
use ita_stf::{Getter, TrustedCallSigned};
use itp_ocall_api::EnclaveMetricsOCallApi;
use itp_sgx_externalities::{SgxExternalitiesTrait, StateHash};
use itp_stf_executor::traits::StateUpdateProposer;
use itp_top_pool_author::traits::AuthorApi;
use itp_types::H256;
use its_block_composer::ComposeBlock;
use its_consensus_common::{Environment, Error as ConsensusError};
use its_primitives::traits::{
	Block as SidechainBlockTrait, Header as HeaderTrait, ShardIdentifierFor,
	SignedBlock as SignedSidechainBlockTrait,
};
use its_state::{SidechainState, SidechainSystemExt};
use sp_runtime::{
	traits::{Block, Header as ParentchainHeaderTrait, NumberFor},
	MultiSignature,
};
use std::{marker::PhantomData, sync::Arc};

///! `ProposerFactory` instance containing all the data to create the `SlotProposer` for the
/// next `Slot`.
pub struct ProposerFactory<
	ParentchainBlock: Block,
	TopPoolAuthor,
	StfExecutor,
	BlockComposer,
	MetricsApi,
	ParentchainHeader,
> {
	top_pool_author: Arc<TopPoolAuthor>,
	stf_executor: Arc<StfExecutor>,
	block_composer: Arc<BlockComposer>,
	metrics_api: Arc<MetricsApi>,
	_phantom: PhantomData<ParentchainBlock>,
	_phantom_header: PhantomData<ParentchainHeader>,
}

impl<ParentchainBlock: Block, TopPoolAuthor, StfExecutor, BlockComposer, MetricsApi, PH>
	ProposerFactory<ParentchainBlock, TopPoolAuthor, StfExecutor, BlockComposer, MetricsApi, PH>
{
	pub fn new(
		top_pool_executor: Arc<TopPoolAuthor>,
		stf_executor: Arc<StfExecutor>,
		block_composer: Arc<BlockComposer>,
		metrics_api: Arc<MetricsApi>,
	) -> Self {
		Self {
			top_pool_author: top_pool_executor,
			stf_executor,
			block_composer,
			metrics_api,
			_phantom: Default::default(),
			_phantom_header: Default::default(),
		}
	}
}

impl<
		ParentchainBlock: Block<Hash = H256, Header = PH>,
		SignedSidechainBlock,
		TopPoolAuthor,
		StfExecutor,
		BlockComposer,
		MetricsApi,
		PH,
	> Environment<ParentchainBlock, SignedSidechainBlock>
	for ProposerFactory<ParentchainBlock, TopPoolAuthor, StfExecutor, BlockComposer, MetricsApi, PH>
where
	NumberFor<ParentchainBlock>: BlockNumberOps,
	SignedSidechainBlock: SignedSidechainBlockTrait<Public = sp_core::ed25519::Public, Signature = MultiSignature>
		+ 'static,
	SignedSidechainBlock::Block: SidechainBlockTrait<Public = sp_core::ed25519::Public>,
	<<SignedSidechainBlock as SignedSidechainBlockTrait>::Block as SidechainBlockTrait>::HeaderType:
		HeaderTrait<ShardIdentifier = H256>,
	TopPoolAuthor:
		AuthorApi<H256, ParentchainBlock::Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	StfExecutor: StateUpdateProposer<TrustedCallSigned, Getter, PH> + Send + Sync + 'static,
	ExternalitiesFor<StfExecutor, PH>:
		SgxExternalitiesTrait + SidechainState + SidechainSystemExt + StateHash,
	<ExternalitiesFor<StfExecutor, PH> as SgxExternalitiesTrait>::SgxExternalitiesType: Encode,
	BlockComposer: ComposeBlock<
			ExternalitiesFor<StfExecutor, PH>,
			ParentchainBlock,
			SignedSidechainBlock = SignedSidechainBlock,
		> + Send
		+ Sync
		+ 'static,
	MetricsApi: EnclaveMetricsOCallApi,
	PH: ParentchainHeaderTrait<Hash = H256>,
{
	type Proposer = SlotProposer<
		ParentchainBlock,
		SignedSidechainBlock,
		TopPoolAuthor,
		StfExecutor,
		BlockComposer,
		MetricsApi,
		PH,
	>;
	type Error = ConsensusError;

	fn init(
		&mut self,
		parent_header: ParentchainBlock::Header,
		shard: ShardIdentifierFor<SignedSidechainBlock>,
	) -> Result<Self::Proposer, Self::Error> {
		Ok(SlotProposer {
			top_pool_author: self.top_pool_author.clone(),
			stf_executor: self.stf_executor.clone(),
			block_composer: self.block_composer.clone(),
			parentchain_header: parent_header,
			shard,
			metrics_api: self.metrics_api.clone(),
			_phantom: PhantomData,
			_phantom_header: PhantomData,
		})
	}
}
