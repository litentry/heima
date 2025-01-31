use parity_scale_codec::Codec;
use subxt::backend::legacy::rpc_methods::TransactionStatus as SubxtTransactionStatus;

/// Simplified TransactionStatus to allow the user to choose until when to watch
/// an extrinsic.
// Indexes must match the TransactionStatus::as_u8 from below.
#[derive(Debug, PartialEq, PartialOrd, Eq, Copy, Clone)]
pub enum XtStatus {
	Ready = 1,
	Broadcast = 2,
	InBlock = 3,
	Retracted = 4,
	Finalized = 6,
}

/// TxStatus that is not expected during the watch process. Will be returned
/// as unexpected error if encountered due to the potential danger of endless loops.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UnexpectedTxStatus {
	Future,
	FinalityTimeout,
	Usurped,
	Dropped,
	Invalid,
}

pub enum TransactionStatus<Hash: Codec> {
	/// Transaction is part of the future queue.
	Future,
	/// Transaction is part of the ready queue.
	Ready,
	/// The transaction has been broadcast to the given peers.
	#[allow(dead_code)]
	Broadcast(Vec<String>),
	/// Transaction has been included in block with given hash.
	InBlock(Hash),
	/// The block this transaction was included in has been retracted.
	Retracted(Hash),
	/// Maximum number of finality watchers has been reached,
	/// old watchers are being removed.
	FinalityTimeout(Hash),
	/// Transaction has been finalized by a finality-gadget, e.g GRANDPA
	Finalized(Hash),
	/// Transaction has been replaced in the pool, by another transaction
	/// that provides the same tags. (e.g. same (sender, nonce)).
	Usurped(Hash),
	/// Transaction has been dropped from the pool because of the limit.
	Dropped,
	/// Transaction is no longer valid in the current state.
	Invalid,
}

impl<Hash: Codec> TransactionStatus<Hash> {
	pub fn as_u8(&self) -> u8 {
		match self {
			Self::Future => 0,
			Self::Ready => 1,
			Self::Broadcast(_) => 2,
			Self::InBlock(_) => 3,
			Self::Retracted(_) => 4,
			Self::FinalityTimeout(_) => 5,
			Self::Finalized(_) => 6,
			Self::Usurped(_) => 7,
			Self::Dropped => 8,
			Self::Invalid => 9,
		}
	}

	pub fn is_expected(&self) -> Result<(), UnexpectedTxStatus> {
		match self {
			Self::Ready
			| Self::Broadcast(_)
			| Self::InBlock(_)
			| Self::Retracted(_)
			| Self::Finalized(_) => Ok(()),
			Self::Future => Err(UnexpectedTxStatus::Future),
			Self::FinalityTimeout(_) => Err(UnexpectedTxStatus::FinalityTimeout),
			Self::Usurped(_) => Err(UnexpectedTxStatus::Usurped),
			Self::Dropped => Err(UnexpectedTxStatus::Dropped),
			Self::Invalid => Err(UnexpectedTxStatus::Invalid),
		}
	}

	/// Returns true if the input status has been reached (or overreached)
	/// and false in case the status is not yet on the expected level.
	pub fn reached_status(&self, status: XtStatus) -> bool {
		self.as_u8() >= status as u8
	}

	pub fn get_maybe_block_hash(&self) -> Option<&Hash> {
		match self {
			Self::InBlock(block_hash) => Some(block_hash),
			Self::Retracted(block_hash) => Some(block_hash),
			Self::FinalityTimeout(block_hash) => Some(block_hash),
			Self::Finalized(block_hash) => Some(block_hash),
			_ => None,
		}
	}
}

impl<Hash: Codec> From<SubxtTransactionStatus<Hash>> for TransactionStatus<Hash> {
	fn from(value: SubxtTransactionStatus<Hash>) -> Self {
		match value {
			SubxtTransactionStatus::Future => Self::Future,
			SubxtTransactionStatus::Ready => Self::Ready,
			SubxtTransactionStatus::Broadcast(peers) => Self::Broadcast(peers),
			SubxtTransactionStatus::InBlock(hash) => Self::InBlock(hash),
			SubxtTransactionStatus::Retracted(hash) => Self::Retracted(hash),
			SubxtTransactionStatus::FinalityTimeout(hash) => Self::FinalityTimeout(hash),
			SubxtTransactionStatus::Finalized(hash) => Self::Finalized(hash),
			SubxtTransactionStatus::Usurped(hash) => Self::Usurped(hash),
			SubxtTransactionStatus::Dropped => Self::Dropped,
			SubxtTransactionStatus::Invalid => Self::Invalid,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{TransactionStatus as GenericTransactionStatus, *};
	use executor_primitives::Hash;

	type TransactionStatus = GenericTransactionStatus<Hash>;

	#[test]
	fn test_xt_status_as_u8() {
		assert_eq!(1, XtStatus::Ready as u8);
		assert_eq!(2, XtStatus::Broadcast as u8);
		assert_eq!(3, XtStatus::InBlock as u8);
		assert_eq!(6, XtStatus::Finalized as u8);
	}

	#[test]
	fn test_transaction_status_as_u8() {
		assert_eq!(0, TransactionStatus::Future.as_u8());
		assert_eq!(1, TransactionStatus::Ready.as_u8());
		assert_eq!(2, TransactionStatus::Broadcast(vec![]).as_u8());
		assert_eq!(3, TransactionStatus::InBlock(Hash::random()).as_u8());
		assert_eq!(4, TransactionStatus::Retracted(Hash::random()).as_u8());
		assert_eq!(5, TransactionStatus::FinalityTimeout(Hash::random()).as_u8());
		assert_eq!(6, TransactionStatus::Finalized(Hash::random()).as_u8());
		assert_eq!(7, TransactionStatus::Usurped(Hash::random()).as_u8());
		assert_eq!(8, TransactionStatus::Dropped.as_u8());
		assert_eq!(9, TransactionStatus::Invalid.as_u8());
	}

	#[test]
	fn test_transaction_status_is_expected() {
		// Supported.
		assert!(TransactionStatus::Ready.is_expected().is_ok());
		assert!(TransactionStatus::Broadcast(vec![]).is_expected().is_ok());
		assert!(TransactionStatus::InBlock(Hash::random()).is_expected().is_ok());
		assert!(TransactionStatus::Retracted(Hash::random()).is_expected().is_ok());
		assert!(TransactionStatus::Finalized(Hash::random()).is_expected().is_ok());

		// Not supported.
		assert!(TransactionStatus::Future.is_expected().is_err());
		assert!(TransactionStatus::FinalityTimeout(Hash::random()).is_expected().is_err());
		assert!(TransactionStatus::Usurped(Hash::random()).is_expected().is_err());
		assert!(TransactionStatus::Dropped.is_expected().is_err());
		assert!(TransactionStatus::Invalid.is_expected().is_err());
	}

	#[test]
	fn test_reached_xt_status_for_ready() {
		let status = XtStatus::Ready;

		// Has not yet reached XtStatus.
		assert!(!TransactionStatus::Future.reached_status(status));

		// Reached XtStatus.
		assert!(TransactionStatus::Ready.reached_status(status));
		assert!(TransactionStatus::Broadcast(vec![]).reached_status(status));
		assert!(TransactionStatus::InBlock(Hash::random()).reached_status(status));
		assert!(TransactionStatus::FinalityTimeout(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Finalized(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Retracted(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Usurped(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Dropped.reached_status(status));
		assert!(TransactionStatus::Invalid.reached_status(status));
	}

	#[test]
	fn test_reached_xt_status_for_broadcast() {
		let status = XtStatus::Broadcast;

		// Has not yet reached XtStatus.
		assert!(!TransactionStatus::Future.reached_status(status));
		assert!(!TransactionStatus::Ready.reached_status(status));

		// Reached XtStatus.
		assert!(TransactionStatus::Broadcast(vec![]).reached_status(status));
		assert!(TransactionStatus::InBlock(Hash::random()).reached_status(status));
		assert!(TransactionStatus::FinalityTimeout(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Finalized(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Retracted(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Usurped(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Dropped.reached_status(status));
		assert!(TransactionStatus::Invalid.reached_status(status));
	}

	#[test]
	fn test_reached_xt_status_for_in_block() {
		let status = XtStatus::InBlock;

		// Has not yet reached XtStatus.
		assert!(!TransactionStatus::Future.reached_status(status));
		assert!(!TransactionStatus::Ready.reached_status(status));
		assert!(!TransactionStatus::Broadcast(vec![]).reached_status(status));

		// Reached XtStatus.
		assert!(TransactionStatus::InBlock(Hash::random()).reached_status(status));
		assert!(TransactionStatus::FinalityTimeout(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Finalized(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Retracted(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Usurped(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Dropped.reached_status(status));
		assert!(TransactionStatus::Invalid.reached_status(status));
	}

	#[test]
	fn test_reached_xt_status_for_finalized() {
		let status = XtStatus::Finalized;

		// Has not yet reached XtStatus.
		assert!(!TransactionStatus::Future.reached_status(status));
		assert!(!TransactionStatus::Ready.reached_status(status));
		assert!(!TransactionStatus::Broadcast(vec![]).reached_status(status));
		assert!(!TransactionStatus::InBlock(Hash::random()).reached_status(status));
		assert!(!TransactionStatus::Retracted(Hash::random()).reached_status(status));
		assert!(!TransactionStatus::FinalityTimeout(Hash::random()).reached_status(status));

		// Reached XtStatus.
		assert!(TransactionStatus::Finalized(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Usurped(Hash::random()).reached_status(status));
		assert!(TransactionStatus::Dropped.reached_status(status));
		assert!(TransactionStatus::Invalid.reached_status(status));
	}
}
