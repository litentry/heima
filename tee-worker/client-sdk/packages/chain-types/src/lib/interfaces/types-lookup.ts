// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

// import type lookup before we augment - in some environments
// this is required to allow for ambient/previous definitions
import '@polkadot/types/lookup';

import type { Data } from '@polkadot/types';
import type { BTreeMap, BTreeSet, Bytes, Compact, Enum, Null, Option, Result, Set, Struct, Text, U256, U8aFixed, Vec, bool, u128, u16, u32, u64, u8 } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { Vote } from '@polkadot/types/interfaces/elections';
import type { AccountId32, Call, H160, H256, MultiAddress, Perbill, Percent } from '@polkadot/types/interfaces/runtime';
import type { Event } from '@polkadot/types/interfaces/system';

declare module '@polkadot/types/lookup' {
  /** @name FrameSystemAccountInfo (3) */
  interface FrameSystemAccountInfo extends Struct {
    readonly nonce: u32;
    readonly consumers: u32;
    readonly providers: u32;
    readonly sufficients: u32;
    readonly data: PalletBalancesAccountData;
  }

  /** @name PalletBalancesAccountData (5) */
  interface PalletBalancesAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly frozen: u128;
    readonly flags: u128;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeight (8) */
  interface FrameSupportDispatchPerDispatchClassWeight extends Struct {
    readonly normal: SpWeightsWeightV2Weight;
    readonly operational: SpWeightsWeightV2Weight;
    readonly mandatory: SpWeightsWeightV2Weight;
  }

  /** @name SpWeightsWeightV2Weight (9) */
  interface SpWeightsWeightV2Weight extends Struct {
    readonly refTime: Compact<u64>;
    readonly proofSize: Compact<u64>;
  }

  /** @name SpRuntimeDigest (14) */
  interface SpRuntimeDigest extends Struct {
    readonly logs: Vec<SpRuntimeDigestDigestItem>;
  }

  /** @name SpRuntimeDigestDigestItem (16) */
  interface SpRuntimeDigestDigestItem extends Enum {
    readonly isOther: boolean;
    readonly asOther: Bytes;
    readonly isConsensus: boolean;
    readonly asConsensus: ITuple<[U8aFixed, Bytes]>;
    readonly isSeal: boolean;
    readonly asSeal: ITuple<[U8aFixed, Bytes]>;
    readonly isPreRuntime: boolean;
    readonly asPreRuntime: ITuple<[U8aFixed, Bytes]>;
    readonly isRuntimeEnvironmentUpdated: boolean;
    readonly type: 'Other' | 'Consensus' | 'Seal' | 'PreRuntime' | 'RuntimeEnvironmentUpdated';
  }

  /** @name FrameSystemEventRecord (19) */
  interface FrameSystemEventRecord extends Struct {
    readonly phase: FrameSystemPhase;
    readonly event: Event;
    readonly topics: Vec<H256>;
  }

  /** @name FrameSystemEvent (21) */
  interface FrameSystemEvent extends Enum {
    readonly isExtrinsicSuccess: boolean;
    readonly asExtrinsicSuccess: {
      readonly dispatchInfo: FrameSupportDispatchDispatchInfo;
    } & Struct;
    readonly isExtrinsicFailed: boolean;
    readonly asExtrinsicFailed: {
      readonly dispatchError: SpRuntimeDispatchError;
      readonly dispatchInfo: FrameSupportDispatchDispatchInfo;
    } & Struct;
    readonly isCodeUpdated: boolean;
    readonly isNewAccount: boolean;
    readonly asNewAccount: {
      readonly account: AccountId32;
    } & Struct;
    readonly isKilledAccount: boolean;
    readonly asKilledAccount: {
      readonly account: AccountId32;
    } & Struct;
    readonly isRemarked: boolean;
    readonly asRemarked: {
      readonly sender: AccountId32;
      readonly hash_: H256;
    } & Struct;
    readonly type: 'ExtrinsicSuccess' | 'ExtrinsicFailed' | 'CodeUpdated' | 'NewAccount' | 'KilledAccount' | 'Remarked';
  }

  /** @name FrameSupportDispatchDispatchInfo (22) */
  interface FrameSupportDispatchDispatchInfo extends Struct {
    readonly weight: SpWeightsWeightV2Weight;
    readonly class: FrameSupportDispatchDispatchClass;
    readonly paysFee: FrameSupportDispatchPays;
  }

  /** @name FrameSupportDispatchDispatchClass (23) */
  interface FrameSupportDispatchDispatchClass extends Enum {
    readonly isNormal: boolean;
    readonly isOperational: boolean;
    readonly isMandatory: boolean;
    readonly type: 'Normal' | 'Operational' | 'Mandatory';
  }

  /** @name FrameSupportDispatchPays (24) */
  interface FrameSupportDispatchPays extends Enum {
    readonly isYes: boolean;
    readonly isNo: boolean;
    readonly type: 'Yes' | 'No';
  }

  /** @name SpRuntimeDispatchError (25) */
  interface SpRuntimeDispatchError extends Enum {
    readonly isOther: boolean;
    readonly isCannotLookup: boolean;
    readonly isBadOrigin: boolean;
    readonly isModule: boolean;
    readonly asModule: SpRuntimeModuleError;
    readonly isConsumerRemaining: boolean;
    readonly isNoProviders: boolean;
    readonly isTooManyConsumers: boolean;
    readonly isToken: boolean;
    readonly asToken: SpRuntimeTokenError;
    readonly isArithmetic: boolean;
    readonly asArithmetic: SpArithmeticArithmeticError;
    readonly isTransactional: boolean;
    readonly asTransactional: SpRuntimeTransactionalError;
    readonly isExhausted: boolean;
    readonly isCorruption: boolean;
    readonly isUnavailable: boolean;
    readonly type: 'Other' | 'CannotLookup' | 'BadOrigin' | 'Module' | 'ConsumerRemaining' | 'NoProviders' | 'TooManyConsumers' | 'Token' | 'Arithmetic' | 'Transactional' | 'Exhausted' | 'Corruption' | 'Unavailable';
  }

  /** @name SpRuntimeModuleError (26) */
  interface SpRuntimeModuleError extends Struct {
    readonly index: u8;
    readonly error: U8aFixed;
  }

  /** @name SpRuntimeTokenError (27) */
  interface SpRuntimeTokenError extends Enum {
    readonly isFundsUnavailable: boolean;
    readonly isOnlyProvider: boolean;
    readonly isBelowMinimum: boolean;
    readonly isCannotCreate: boolean;
    readonly isUnknownAsset: boolean;
    readonly isFrozen: boolean;
    readonly isUnsupported: boolean;
    readonly isCannotCreateHold: boolean;
    readonly isNotExpendable: boolean;
    readonly type: 'FundsUnavailable' | 'OnlyProvider' | 'BelowMinimum' | 'CannotCreate' | 'UnknownAsset' | 'Frozen' | 'Unsupported' | 'CannotCreateHold' | 'NotExpendable';
  }

  /** @name SpArithmeticArithmeticError (28) */
  interface SpArithmeticArithmeticError extends Enum {
    readonly isUnderflow: boolean;
    readonly isOverflow: boolean;
    readonly isDivisionByZero: boolean;
    readonly type: 'Underflow' | 'Overflow' | 'DivisionByZero';
  }

  /** @name SpRuntimeTransactionalError (29) */
  interface SpRuntimeTransactionalError extends Enum {
    readonly isLimitReached: boolean;
    readonly isNoLayer: boolean;
    readonly type: 'LimitReached' | 'NoLayer';
  }

  /** @name PalletSchedulerEvent (30) */
  interface PalletSchedulerEvent extends Enum {
    readonly isScheduled: boolean;
    readonly asScheduled: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isCanceled: boolean;
    readonly asCanceled: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isDispatched: boolean;
    readonly asDispatched: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isCallUnavailable: boolean;
    readonly asCallUnavailable: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPeriodicFailed: boolean;
    readonly asPeriodicFailed: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly isPermanentlyOverweight: boolean;
    readonly asPermanentlyOverweight: {
      readonly task: ITuple<[u32, u32]>;
      readonly id: Option<U8aFixed>;
    } & Struct;
    readonly type: 'Scheduled' | 'Canceled' | 'Dispatched' | 'CallUnavailable' | 'PeriodicFailed' | 'PermanentlyOverweight';
  }

  /** @name PalletUtilityEvent (35) */
  interface PalletUtilityEvent extends Enum {
    readonly isBatchInterrupted: boolean;
    readonly asBatchInterrupted: {
      readonly index: u32;
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isBatchCompleted: boolean;
    readonly isBatchCompletedWithErrors: boolean;
    readonly isItemCompleted: boolean;
    readonly isItemFailed: boolean;
    readonly asItemFailed: {
      readonly error: SpRuntimeDispatchError;
    } & Struct;
    readonly isDispatchedAs: boolean;
    readonly asDispatchedAs: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'BatchInterrupted' | 'BatchCompleted' | 'BatchCompletedWithErrors' | 'ItemCompleted' | 'ItemFailed' | 'DispatchedAs';
  }

  /** @name PalletMultisigEvent (36) */
  interface PalletMultisigEvent extends Enum {
    readonly isNewMultisig: boolean;
    readonly asNewMultisig: {
      readonly approving: AccountId32;
      readonly multisig: AccountId32;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly isMultisigApproval: boolean;
    readonly asMultisigApproval: {
      readonly approving: AccountId32;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId32;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly isMultisigExecuted: boolean;
    readonly asMultisigExecuted: {
      readonly approving: AccountId32;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId32;
      readonly callHash: U8aFixed;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isMultisigCancelled: boolean;
    readonly asMultisigCancelled: {
      readonly cancelling: AccountId32;
      readonly timepoint: PalletMultisigTimepoint;
      readonly multisig: AccountId32;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly type: 'NewMultisig' | 'MultisigApproval' | 'MultisigExecuted' | 'MultisigCancelled';
  }

  /** @name PalletMultisigTimepoint (37) */
  interface PalletMultisigTimepoint extends Struct {
    readonly height: u32;
    readonly index: u32;
  }

  /** @name PalletProxyEvent (38) */
  interface PalletProxyEvent extends Enum {
    readonly isProxyExecuted: boolean;
    readonly asProxyExecuted: {
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isPureCreated: boolean;
    readonly asPureCreated: {
      readonly pure: AccountId32;
      readonly who: AccountId32;
      readonly proxyType: RococoParachainRuntimeProxyType;
      readonly disambiguationIndex: u16;
    } & Struct;
    readonly isAnnounced: boolean;
    readonly asAnnounced: {
      readonly real: AccountId32;
      readonly proxy: AccountId32;
      readonly callHash: H256;
    } & Struct;
    readonly isProxyAdded: boolean;
    readonly asProxyAdded: {
      readonly delegator: AccountId32;
      readonly delegatee: AccountId32;
      readonly proxyType: RococoParachainRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly isProxyRemoved: boolean;
    readonly asProxyRemoved: {
      readonly delegator: AccountId32;
      readonly delegatee: AccountId32;
      readonly proxyType: RococoParachainRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly type: 'ProxyExecuted' | 'PureCreated' | 'Announced' | 'ProxyAdded' | 'ProxyRemoved';
  }

  /** @name RococoParachainRuntimeProxyType (39) */
  interface RococoParachainRuntimeProxyType extends Enum {
    readonly isAny: boolean;
    readonly isNonTransfer: boolean;
    readonly isCancelProxy: boolean;
    readonly isCollator: boolean;
    readonly isGovernance: boolean;
    readonly type: 'Any' | 'NonTransfer' | 'CancelProxy' | 'Collator' | 'Governance';
  }

  /** @name PalletPreimageEvent (41) */
  interface PalletPreimageEvent extends Enum {
    readonly isNoted: boolean;
    readonly asNoted: {
      readonly hash_: H256;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly hash_: H256;
    } & Struct;
    readonly isCleared: boolean;
    readonly asCleared: {
      readonly hash_: H256;
    } & Struct;
    readonly type: 'Noted' | 'Requested' | 'Cleared';
  }

  /** @name PalletBalancesEvent (42) */
  interface PalletBalancesEvent extends Enum {
    readonly isEndowed: boolean;
    readonly asEndowed: {
      readonly account: AccountId32;
      readonly freeBalance: u128;
    } & Struct;
    readonly isDustLost: boolean;
    readonly asDustLost: {
      readonly account: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isBalanceSet: boolean;
    readonly asBalanceSet: {
      readonly who: AccountId32;
      readonly free: u128;
    } & Struct;
    readonly isReserved: boolean;
    readonly asReserved: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnreserved: boolean;
    readonly asUnreserved: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isReserveRepatriated: boolean;
    readonly asReserveRepatriated: {
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
      readonly destinationStatus: FrameSupportTokensMiscBalanceStatus;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isMinted: boolean;
    readonly asMinted: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isBurned: boolean;
    readonly asBurned: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSuspended: boolean;
    readonly asSuspended: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isRestored: boolean;
    readonly asRestored: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUpgraded: boolean;
    readonly asUpgraded: {
      readonly who: AccountId32;
    } & Struct;
    readonly isIssued: boolean;
    readonly asIssued: {
      readonly amount: u128;
    } & Struct;
    readonly isRescinded: boolean;
    readonly asRescinded: {
      readonly amount: u128;
    } & Struct;
    readonly isLocked: boolean;
    readonly asLocked: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnlocked: boolean;
    readonly asUnlocked: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isFrozen: boolean;
    readonly asFrozen: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isThawed: boolean;
    readonly asThawed: {
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Endowed' | 'DustLost' | 'Transfer' | 'BalanceSet' | 'Reserved' | 'Unreserved' | 'ReserveRepatriated' | 'Deposit' | 'Withdraw' | 'Slashed' | 'Minted' | 'Burned' | 'Suspended' | 'Restored' | 'Upgraded' | 'Issued' | 'Rescinded' | 'Locked' | 'Unlocked' | 'Frozen' | 'Thawed';
  }

  /** @name FrameSupportTokensMiscBalanceStatus (43) */
  interface FrameSupportTokensMiscBalanceStatus extends Enum {
    readonly isFree: boolean;
    readonly isReserved: boolean;
    readonly type: 'Free' | 'Reserved';
  }

  /** @name PalletVestingEvent (44) */
  interface PalletVestingEvent extends Enum {
    readonly isVestingUpdated: boolean;
    readonly asVestingUpdated: {
      readonly account: AccountId32;
      readonly unvested: u128;
    } & Struct;
    readonly isVestingCompleted: boolean;
    readonly asVestingCompleted: {
      readonly account: AccountId32;
    } & Struct;
    readonly type: 'VestingUpdated' | 'VestingCompleted';
  }

  /** @name PalletTransactionPaymentEvent (45) */
  interface PalletTransactionPaymentEvent extends Enum {
    readonly isTransactionFeePaid: boolean;
    readonly asTransactionFeePaid: {
      readonly who: AccountId32;
      readonly actualFee: u128;
      readonly tip: u128;
    } & Struct;
    readonly type: 'TransactionFeePaid';
  }

  /** @name PalletTreasuryEvent (46) */
  interface PalletTreasuryEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly proposalIndex: u32;
    } & Struct;
    readonly isSpending: boolean;
    readonly asSpending: {
      readonly budgetRemaining: u128;
    } & Struct;
    readonly isAwarded: boolean;
    readonly asAwarded: {
      readonly proposalIndex: u32;
      readonly award: u128;
      readonly account: AccountId32;
    } & Struct;
    readonly isRejected: boolean;
    readonly asRejected: {
      readonly proposalIndex: u32;
      readonly slashed: u128;
    } & Struct;
    readonly isBurnt: boolean;
    readonly asBurnt: {
      readonly burntFunds: u128;
    } & Struct;
    readonly isRollover: boolean;
    readonly asRollover: {
      readonly rolloverBalance: u128;
    } & Struct;
    readonly isDeposit: boolean;
    readonly asDeposit: {
      readonly value: u128;
    } & Struct;
    readonly isSpendApproved: boolean;
    readonly asSpendApproved: {
      readonly proposalIndex: u32;
      readonly amount: u128;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isUpdatedInactive: boolean;
    readonly asUpdatedInactive: {
      readonly reactivated: u128;
      readonly deactivated: u128;
    } & Struct;
    readonly type: 'Proposed' | 'Spending' | 'Awarded' | 'Rejected' | 'Burnt' | 'Rollover' | 'Deposit' | 'SpendApproved' | 'UpdatedInactive';
  }

  /** @name PalletDemocracyEvent (47) */
  interface PalletDemocracyEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly proposalIndex: u32;
      readonly deposit: u128;
    } & Struct;
    readonly isTabled: boolean;
    readonly asTabled: {
      readonly proposalIndex: u32;
      readonly deposit: u128;
    } & Struct;
    readonly isExternalTabled: boolean;
    readonly isStarted: boolean;
    readonly asStarted: {
      readonly refIndex: u32;
      readonly threshold: PalletDemocracyVoteThreshold;
    } & Struct;
    readonly isPassed: boolean;
    readonly asPassed: {
      readonly refIndex: u32;
    } & Struct;
    readonly isNotPassed: boolean;
    readonly asNotPassed: {
      readonly refIndex: u32;
    } & Struct;
    readonly isCancelled: boolean;
    readonly asCancelled: {
      readonly refIndex: u32;
    } & Struct;
    readonly isDelegated: boolean;
    readonly asDelegated: {
      readonly who: AccountId32;
      readonly target: AccountId32;
    } & Struct;
    readonly isUndelegated: boolean;
    readonly asUndelegated: {
      readonly account: AccountId32;
    } & Struct;
    readonly isVetoed: boolean;
    readonly asVetoed: {
      readonly who: AccountId32;
      readonly proposalHash: H256;
      readonly until: u32;
    } & Struct;
    readonly isBlacklisted: boolean;
    readonly asBlacklisted: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isVoted: boolean;
    readonly asVoted: {
      readonly voter: AccountId32;
      readonly refIndex: u32;
      readonly vote: PalletDemocracyVoteAccountVote;
    } & Struct;
    readonly isSeconded: boolean;
    readonly asSeconded: {
      readonly seconder: AccountId32;
      readonly propIndex: u32;
    } & Struct;
    readonly isProposalCanceled: boolean;
    readonly asProposalCanceled: {
      readonly propIndex: u32;
    } & Struct;
    readonly isMetadataSet: boolean;
    readonly asMetadataSet: {
      readonly owner: PalletDemocracyMetadataOwner;
      readonly hash_: H256;
    } & Struct;
    readonly isMetadataCleared: boolean;
    readonly asMetadataCleared: {
      readonly owner: PalletDemocracyMetadataOwner;
      readonly hash_: H256;
    } & Struct;
    readonly isMetadataTransferred: boolean;
    readonly asMetadataTransferred: {
      readonly prevOwner: PalletDemocracyMetadataOwner;
      readonly owner: PalletDemocracyMetadataOwner;
      readonly hash_: H256;
    } & Struct;
    readonly type: 'Proposed' | 'Tabled' | 'ExternalTabled' | 'Started' | 'Passed' | 'NotPassed' | 'Cancelled' | 'Delegated' | 'Undelegated' | 'Vetoed' | 'Blacklisted' | 'Voted' | 'Seconded' | 'ProposalCanceled' | 'MetadataSet' | 'MetadataCleared' | 'MetadataTransferred';
  }

  /** @name PalletDemocracyVoteThreshold (48) */
  interface PalletDemocracyVoteThreshold extends Enum {
    readonly isSuperMajorityApprove: boolean;
    readonly isSuperMajorityAgainst: boolean;
    readonly isSimpleMajority: boolean;
    readonly type: 'SuperMajorityApprove' | 'SuperMajorityAgainst' | 'SimpleMajority';
  }

  /** @name PalletDemocracyVoteAccountVote (49) */
  interface PalletDemocracyVoteAccountVote extends Enum {
    readonly isStandard: boolean;
    readonly asStandard: {
      readonly vote: Vote;
      readonly balance: u128;
    } & Struct;
    readonly isSplit: boolean;
    readonly asSplit: {
      readonly aye: u128;
      readonly nay: u128;
    } & Struct;
    readonly type: 'Standard' | 'Split';
  }

  /** @name PalletDemocracyMetadataOwner (51) */
  interface PalletDemocracyMetadataOwner extends Enum {
    readonly isExternal: boolean;
    readonly isProposal: boolean;
    readonly asProposal: u32;
    readonly isReferendum: boolean;
    readonly asReferendum: u32;
    readonly type: 'External' | 'Proposal' | 'Referendum';
  }

  /** @name PalletCollectiveEvent (52) */
  interface PalletCollectiveEvent extends Enum {
    readonly isProposed: boolean;
    readonly asProposed: {
      readonly account: AccountId32;
      readonly proposalIndex: u32;
      readonly proposalHash: H256;
      readonly threshold: u32;
    } & Struct;
    readonly isVoted: boolean;
    readonly asVoted: {
      readonly account: AccountId32;
      readonly proposalHash: H256;
      readonly voted: bool;
      readonly yes: u32;
      readonly no: u32;
    } & Struct;
    readonly isApproved: boolean;
    readonly asApproved: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isDisapproved: boolean;
    readonly asDisapproved: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly proposalHash: H256;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isMemberExecuted: boolean;
    readonly asMemberExecuted: {
      readonly proposalHash: H256;
      readonly result: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isClosed: boolean;
    readonly asClosed: {
      readonly proposalHash: H256;
      readonly yes: u32;
      readonly no: u32;
    } & Struct;
    readonly type: 'Proposed' | 'Voted' | 'Approved' | 'Disapproved' | 'Executed' | 'MemberExecuted' | 'Closed';
  }

  /** @name PalletMembershipEvent (54) */
  interface PalletMembershipEvent extends Enum {
    readonly isMemberAdded: boolean;
    readonly isMemberRemoved: boolean;
    readonly isMembersSwapped: boolean;
    readonly isMembersReset: boolean;
    readonly isKeyChanged: boolean;
    readonly isDummy: boolean;
    readonly type: 'MemberAdded' | 'MemberRemoved' | 'MembersSwapped' | 'MembersReset' | 'KeyChanged' | 'Dummy';
  }

  /** @name PalletBountiesEvent (57) */
  interface PalletBountiesEvent extends Enum {
    readonly isBountyProposed: boolean;
    readonly asBountyProposed: {
      readonly index: u32;
    } & Struct;
    readonly isBountyRejected: boolean;
    readonly asBountyRejected: {
      readonly index: u32;
      readonly bond: u128;
    } & Struct;
    readonly isBountyBecameActive: boolean;
    readonly asBountyBecameActive: {
      readonly index: u32;
    } & Struct;
    readonly isBountyAwarded: boolean;
    readonly asBountyAwarded: {
      readonly index: u32;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isBountyClaimed: boolean;
    readonly asBountyClaimed: {
      readonly index: u32;
      readonly payout: u128;
      readonly beneficiary: AccountId32;
    } & Struct;
    readonly isBountyCanceled: boolean;
    readonly asBountyCanceled: {
      readonly index: u32;
    } & Struct;
    readonly isBountyExtended: boolean;
    readonly asBountyExtended: {
      readonly index: u32;
    } & Struct;
    readonly type: 'BountyProposed' | 'BountyRejected' | 'BountyBecameActive' | 'BountyAwarded' | 'BountyClaimed' | 'BountyCanceled' | 'BountyExtended';
  }

  /** @name PalletTipsEvent (58) */
  interface PalletTipsEvent extends Enum {
    readonly isNewTip: boolean;
    readonly asNewTip: {
      readonly tipHash: H256;
    } & Struct;
    readonly isTipClosing: boolean;
    readonly asTipClosing: {
      readonly tipHash: H256;
    } & Struct;
    readonly isTipClosed: boolean;
    readonly asTipClosed: {
      readonly tipHash: H256;
      readonly who: AccountId32;
      readonly payout: u128;
    } & Struct;
    readonly isTipRetracted: boolean;
    readonly asTipRetracted: {
      readonly tipHash: H256;
    } & Struct;
    readonly isTipSlashed: boolean;
    readonly asTipSlashed: {
      readonly tipHash: H256;
      readonly finder: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly type: 'NewTip' | 'TipClosing' | 'TipClosed' | 'TipRetracted' | 'TipSlashed';
  }

  /** @name PalletIdentityEvent (59) */
  interface PalletIdentityEvent extends Enum {
    readonly isIdentitySet: boolean;
    readonly asIdentitySet: {
      readonly who: AccountId32;
    } & Struct;
    readonly isIdentityCleared: boolean;
    readonly asIdentityCleared: {
      readonly who: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly isIdentityKilled: boolean;
    readonly asIdentityKilled: {
      readonly who: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly isJudgementRequested: boolean;
    readonly asJudgementRequested: {
      readonly who: AccountId32;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isJudgementUnrequested: boolean;
    readonly asJudgementUnrequested: {
      readonly who: AccountId32;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isJudgementGiven: boolean;
    readonly asJudgementGiven: {
      readonly target: AccountId32;
      readonly registrarIndex: u32;
    } & Struct;
    readonly isRegistrarAdded: boolean;
    readonly asRegistrarAdded: {
      readonly registrarIndex: u32;
    } & Struct;
    readonly isSubIdentityAdded: boolean;
    readonly asSubIdentityAdded: {
      readonly sub: AccountId32;
      readonly main: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly isSubIdentityRemoved: boolean;
    readonly asSubIdentityRemoved: {
      readonly sub: AccountId32;
      readonly main: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly isSubIdentityRevoked: boolean;
    readonly asSubIdentityRevoked: {
      readonly sub: AccountId32;
      readonly main: AccountId32;
      readonly deposit: u128;
    } & Struct;
    readonly type: 'IdentitySet' | 'IdentityCleared' | 'IdentityKilled' | 'JudgementRequested' | 'JudgementUnrequested' | 'JudgementGiven' | 'RegistrarAdded' | 'SubIdentityAdded' | 'SubIdentityRemoved' | 'SubIdentityRevoked';
  }

  /** @name CumulusPalletParachainSystemEvent (60) */
  interface CumulusPalletParachainSystemEvent extends Enum {
    readonly isValidationFunctionStored: boolean;
    readonly isValidationFunctionApplied: boolean;
    readonly asValidationFunctionApplied: {
      readonly relayChainBlockNum: u32;
    } & Struct;
    readonly isValidationFunctionDiscarded: boolean;
    readonly isUpgradeAuthorized: boolean;
    readonly asUpgradeAuthorized: {
      readonly codeHash: H256;
    } & Struct;
    readonly isDownwardMessagesReceived: boolean;
    readonly asDownwardMessagesReceived: {
      readonly count: u32;
    } & Struct;
    readonly isDownwardMessagesProcessed: boolean;
    readonly asDownwardMessagesProcessed: {
      readonly weightUsed: SpWeightsWeightV2Weight;
      readonly dmqHead: H256;
    } & Struct;
    readonly isUpwardMessageSent: boolean;
    readonly asUpwardMessageSent: {
      readonly messageHash: Option<U8aFixed>;
    } & Struct;
    readonly type: 'ValidationFunctionStored' | 'ValidationFunctionApplied' | 'ValidationFunctionDiscarded' | 'UpgradeAuthorized' | 'DownwardMessagesReceived' | 'DownwardMessagesProcessed' | 'UpwardMessageSent';
  }

  /** @name PalletSessionEvent (61) */
  interface PalletSessionEvent extends Enum {
    readonly isNewSession: boolean;
    readonly asNewSession: {
      readonly sessionIndex: u32;
    } & Struct;
    readonly type: 'NewSession';
  }

  /** @name PalletParachainStakingEvent (62) */
  interface PalletParachainStakingEvent extends Enum {
    readonly isNewRound: boolean;
    readonly asNewRound: {
      readonly startingBlock: u32;
      readonly round: u32;
      readonly selectedCollatorsNumber: u32;
      readonly totalBalance: u128;
    } & Struct;
    readonly isJoinedCollatorCandidates: boolean;
    readonly asJoinedCollatorCandidates: {
      readonly account: AccountId32;
      readonly amountLocked: u128;
      readonly newTotalAmtLocked: u128;
    } & Struct;
    readonly isCollatorChosen: boolean;
    readonly asCollatorChosen: {
      readonly round: u32;
      readonly collatorAccount: AccountId32;
      readonly totalExposedAmount: u128;
    } & Struct;
    readonly isCandidateBondLessRequested: boolean;
    readonly asCandidateBondLessRequested: {
      readonly candidate: AccountId32;
      readonly amountToDecrease: u128;
      readonly executeRound: u32;
    } & Struct;
    readonly isCandidateBondedMore: boolean;
    readonly asCandidateBondedMore: {
      readonly candidate: AccountId32;
      readonly amount: u128;
      readonly newTotalBond: u128;
    } & Struct;
    readonly isCandidateBondedLess: boolean;
    readonly asCandidateBondedLess: {
      readonly candidate: AccountId32;
      readonly amount: u128;
      readonly newBond: u128;
    } & Struct;
    readonly isCandidateWentOffline: boolean;
    readonly asCandidateWentOffline: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isCandidateBackOnline: boolean;
    readonly asCandidateBackOnline: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isCandidateScheduledExit: boolean;
    readonly asCandidateScheduledExit: {
      readonly exitAllowedRound: u32;
      readonly candidate: AccountId32;
      readonly scheduledExit: u32;
    } & Struct;
    readonly isCancelledCandidateExit: boolean;
    readonly asCancelledCandidateExit: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isCancelledCandidateBondLess: boolean;
    readonly asCancelledCandidateBondLess: {
      readonly candidate: AccountId32;
      readonly amount: u128;
      readonly executeRound: u32;
    } & Struct;
    readonly isCandidateLeft: boolean;
    readonly asCandidateLeft: {
      readonly exCandidate: AccountId32;
      readonly unlockedAmount: u128;
      readonly newTotalAmtLocked: u128;
    } & Struct;
    readonly isDelegationDecreaseScheduled: boolean;
    readonly asDelegationDecreaseScheduled: {
      readonly delegator: AccountId32;
      readonly candidate: AccountId32;
      readonly amountToDecrease: u128;
      readonly executeRound: u32;
    } & Struct;
    readonly isDelegationIncreased: boolean;
    readonly asDelegationIncreased: {
      readonly delegator: AccountId32;
      readonly candidate: AccountId32;
      readonly amount: u128;
      readonly inTop: bool;
    } & Struct;
    readonly isDelegationDecreased: boolean;
    readonly asDelegationDecreased: {
      readonly delegator: AccountId32;
      readonly candidate: AccountId32;
      readonly amount: u128;
      readonly inTop: bool;
    } & Struct;
    readonly isDelegatorExitScheduled: boolean;
    readonly asDelegatorExitScheduled: {
      readonly round: u32;
      readonly delegator: AccountId32;
      readonly scheduledExit: u32;
    } & Struct;
    readonly isDelegationRevocationScheduled: boolean;
    readonly asDelegationRevocationScheduled: {
      readonly round: u32;
      readonly delegator: AccountId32;
      readonly candidate: AccountId32;
      readonly scheduledExit: u32;
    } & Struct;
    readonly isDelegatorLeft: boolean;
    readonly asDelegatorLeft: {
      readonly delegator: AccountId32;
      readonly unstakedAmount: u128;
    } & Struct;
    readonly isDelegationRevoked: boolean;
    readonly asDelegationRevoked: {
      readonly delegator: AccountId32;
      readonly candidate: AccountId32;
      readonly unstakedAmount: u128;
    } & Struct;
    readonly isDelegationKicked: boolean;
    readonly asDelegationKicked: {
      readonly delegator: AccountId32;
      readonly candidate: AccountId32;
      readonly unstakedAmount: u128;
    } & Struct;
    readonly isDelegatorExitCancelled: boolean;
    readonly asDelegatorExitCancelled: {
      readonly delegator: AccountId32;
    } & Struct;
    readonly isCancelledDelegationRequest: boolean;
    readonly asCancelledDelegationRequest: {
      readonly delegator: AccountId32;
      readonly cancelledRequest: PalletParachainStakingDelegationRequestsCancelledScheduledRequest;
      readonly collator: AccountId32;
    } & Struct;
    readonly isDelegation: boolean;
    readonly asDelegation: {
      readonly delegator: AccountId32;
      readonly lockedAmount: u128;
      readonly candidate: AccountId32;
      readonly delegatorPosition: PalletParachainStakingDelegatorAdded;
      readonly autoCompound: Percent;
    } & Struct;
    readonly isDelegatorLeftCandidate: boolean;
    readonly asDelegatorLeftCandidate: {
      readonly delegator: AccountId32;
      readonly candidate: AccountId32;
      readonly unstakedAmount: u128;
      readonly totalCandidateStaked: u128;
    } & Struct;
    readonly isRewarded: boolean;
    readonly asRewarded: {
      readonly account: AccountId32;
      readonly rewards: u128;
    } & Struct;
    readonly isReservedForParachainBond: boolean;
    readonly asReservedForParachainBond: {
      readonly account: AccountId32;
      readonly value: u128;
    } & Struct;
    readonly isParachainBondAccountSet: boolean;
    readonly asParachainBondAccountSet: {
      readonly old: AccountId32;
      readonly new_: AccountId32;
    } & Struct;
    readonly isParachainBondReservePercentSet: boolean;
    readonly asParachainBondReservePercentSet: {
      readonly old: Percent;
      readonly new_: Percent;
    } & Struct;
    readonly isInflationSet: boolean;
    readonly asInflationSet: {
      readonly annualMin: Perbill;
      readonly annualIdeal: Perbill;
      readonly annualMax: Perbill;
      readonly roundMin: Perbill;
      readonly roundIdeal: Perbill;
      readonly roundMax: Perbill;
    } & Struct;
    readonly isStakeExpectationsSet: boolean;
    readonly asStakeExpectationsSet: {
      readonly expectMin: u128;
      readonly expectIdeal: u128;
      readonly expectMax: u128;
    } & Struct;
    readonly isTotalSelectedSet: boolean;
    readonly asTotalSelectedSet: {
      readonly old: u32;
      readonly new_: u32;
    } & Struct;
    readonly isCollatorCommissionSet: boolean;
    readonly asCollatorCommissionSet: {
      readonly old: Perbill;
      readonly new_: Perbill;
    } & Struct;
    readonly isBlocksPerRoundSet: boolean;
    readonly asBlocksPerRoundSet: {
      readonly currentRound: u32;
      readonly firstBlock: u32;
      readonly old: u32;
      readonly new_: u32;
      readonly newPerRoundInflationMin: Perbill;
      readonly newPerRoundInflationIdeal: Perbill;
      readonly newPerRoundInflationMax: Perbill;
    } & Struct;
    readonly isCandidateWhiteListAdded: boolean;
    readonly asCandidateWhiteListAdded: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isCandidateWhiteListRemoved: boolean;
    readonly asCandidateWhiteListRemoved: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isAutoCompoundSet: boolean;
    readonly asAutoCompoundSet: {
      readonly candidate: AccountId32;
      readonly delegator: AccountId32;
      readonly value: Percent;
    } & Struct;
    readonly isCompounded: boolean;
    readonly asCompounded: {
      readonly candidate: AccountId32;
      readonly delegator: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'NewRound' | 'JoinedCollatorCandidates' | 'CollatorChosen' | 'CandidateBondLessRequested' | 'CandidateBondedMore' | 'CandidateBondedLess' | 'CandidateWentOffline' | 'CandidateBackOnline' | 'CandidateScheduledExit' | 'CancelledCandidateExit' | 'CancelledCandidateBondLess' | 'CandidateLeft' | 'DelegationDecreaseScheduled' | 'DelegationIncreased' | 'DelegationDecreased' | 'DelegatorExitScheduled' | 'DelegationRevocationScheduled' | 'DelegatorLeft' | 'DelegationRevoked' | 'DelegationKicked' | 'DelegatorExitCancelled' | 'CancelledDelegationRequest' | 'Delegation' | 'DelegatorLeftCandidate' | 'Rewarded' | 'ReservedForParachainBond' | 'ParachainBondAccountSet' | 'ParachainBondReservePercentSet' | 'InflationSet' | 'StakeExpectationsSet' | 'TotalSelectedSet' | 'CollatorCommissionSet' | 'BlocksPerRoundSet' | 'CandidateWhiteListAdded' | 'CandidateWhiteListRemoved' | 'AutoCompoundSet' | 'Compounded';
  }

  /** @name PalletParachainStakingDelegationRequestsCancelledScheduledRequest (63) */
  interface PalletParachainStakingDelegationRequestsCancelledScheduledRequest extends Struct {
    readonly whenExecutable: u32;
    readonly action: PalletParachainStakingDelegationRequestsDelegationAction;
  }

  /** @name PalletParachainStakingDelegationRequestsDelegationAction (64) */
  interface PalletParachainStakingDelegationRequestsDelegationAction extends Enum {
    readonly isRevoke: boolean;
    readonly asRevoke: u128;
    readonly isDecrease: boolean;
    readonly asDecrease: u128;
    readonly type: 'Revoke' | 'Decrease';
  }

  /** @name PalletParachainStakingDelegatorAdded (65) */
  interface PalletParachainStakingDelegatorAdded extends Enum {
    readonly isAddedToTop: boolean;
    readonly asAddedToTop: {
      readonly newTotal: u128;
    } & Struct;
    readonly isAddedToBottom: boolean;
    readonly type: 'AddedToTop' | 'AddedToBottom';
  }

  /** @name CumulusPalletXcmpQueueEvent (68) */
  interface CumulusPalletXcmpQueueEvent extends Enum {
    readonly isSuccess: boolean;
    readonly asSuccess: {
      readonly messageHash: Option<U8aFixed>;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isFail: boolean;
    readonly asFail: {
      readonly messageHash: Option<U8aFixed>;
      readonly error: XcmV3TraitsError;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isBadVersion: boolean;
    readonly asBadVersion: {
      readonly messageHash: Option<U8aFixed>;
    } & Struct;
    readonly isBadFormat: boolean;
    readonly asBadFormat: {
      readonly messageHash: Option<U8aFixed>;
    } & Struct;
    readonly isXcmpMessageSent: boolean;
    readonly asXcmpMessageSent: {
      readonly messageHash: Option<U8aFixed>;
    } & Struct;
    readonly isOverweightEnqueued: boolean;
    readonly asOverweightEnqueued: {
      readonly sender: u32;
      readonly sentAt: u32;
      readonly index: u64;
      readonly required: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isOverweightServiced: boolean;
    readonly asOverweightServiced: {
      readonly index: u64;
      readonly used: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type: 'Success' | 'Fail' | 'BadVersion' | 'BadFormat' | 'XcmpMessageSent' | 'OverweightEnqueued' | 'OverweightServiced';
  }

  /** @name XcmV3TraitsError (69) */
  interface XcmV3TraitsError extends Enum {
    readonly isOverflow: boolean;
    readonly isUnimplemented: boolean;
    readonly isUntrustedReserveLocation: boolean;
    readonly isUntrustedTeleportLocation: boolean;
    readonly isLocationFull: boolean;
    readonly isLocationNotInvertible: boolean;
    readonly isBadOrigin: boolean;
    readonly isInvalidLocation: boolean;
    readonly isAssetNotFound: boolean;
    readonly isFailedToTransactAsset: boolean;
    readonly isNotWithdrawable: boolean;
    readonly isLocationCannotHold: boolean;
    readonly isExceedsMaxMessageSize: boolean;
    readonly isDestinationUnsupported: boolean;
    readonly isTransport: boolean;
    readonly isUnroutable: boolean;
    readonly isUnknownClaim: boolean;
    readonly isFailedToDecode: boolean;
    readonly isMaxWeightInvalid: boolean;
    readonly isNotHoldingFees: boolean;
    readonly isTooExpensive: boolean;
    readonly isTrap: boolean;
    readonly asTrap: u64;
    readonly isExpectationFalse: boolean;
    readonly isPalletNotFound: boolean;
    readonly isNameMismatch: boolean;
    readonly isVersionIncompatible: boolean;
    readonly isHoldingWouldOverflow: boolean;
    readonly isExportError: boolean;
    readonly isReanchorFailed: boolean;
    readonly isNoDeal: boolean;
    readonly isFeesNotMet: boolean;
    readonly isLockError: boolean;
    readonly isNoPermission: boolean;
    readonly isUnanchored: boolean;
    readonly isNotDepositable: boolean;
    readonly isUnhandledXcmVersion: boolean;
    readonly isWeightLimitReached: boolean;
    readonly asWeightLimitReached: SpWeightsWeightV2Weight;
    readonly isBarrier: boolean;
    readonly isWeightNotComputable: boolean;
    readonly isExceedsStackLimit: boolean;
    readonly type: 'Overflow' | 'Unimplemented' | 'UntrustedReserveLocation' | 'UntrustedTeleportLocation' | 'LocationFull' | 'LocationNotInvertible' | 'BadOrigin' | 'InvalidLocation' | 'AssetNotFound' | 'FailedToTransactAsset' | 'NotWithdrawable' | 'LocationCannotHold' | 'ExceedsMaxMessageSize' | 'DestinationUnsupported' | 'Transport' | 'Unroutable' | 'UnknownClaim' | 'FailedToDecode' | 'MaxWeightInvalid' | 'NotHoldingFees' | 'TooExpensive' | 'Trap' | 'ExpectationFalse' | 'PalletNotFound' | 'NameMismatch' | 'VersionIncompatible' | 'HoldingWouldOverflow' | 'ExportError' | 'ReanchorFailed' | 'NoDeal' | 'FeesNotMet' | 'LockError' | 'NoPermission' | 'Unanchored' | 'NotDepositable' | 'UnhandledXcmVersion' | 'WeightLimitReached' | 'Barrier' | 'WeightNotComputable' | 'ExceedsStackLimit';
  }

  /** @name PalletXcmEvent (71) */
  interface PalletXcmEvent extends Enum {
    readonly isAttempted: boolean;
    readonly asAttempted: XcmV3TraitsOutcome;
    readonly isSent: boolean;
    readonly asSent: ITuple<[XcmV3MultiLocation, XcmV3MultiLocation, XcmV3Xcm]>;
    readonly isUnexpectedResponse: boolean;
    readonly asUnexpectedResponse: ITuple<[XcmV3MultiLocation, u64]>;
    readonly isResponseReady: boolean;
    readonly asResponseReady: ITuple<[u64, XcmV3Response]>;
    readonly isNotified: boolean;
    readonly asNotified: ITuple<[u64, u8, u8]>;
    readonly isNotifyOverweight: boolean;
    readonly asNotifyOverweight: ITuple<[u64, u8, u8, SpWeightsWeightV2Weight, SpWeightsWeightV2Weight]>;
    readonly isNotifyDispatchError: boolean;
    readonly asNotifyDispatchError: ITuple<[u64, u8, u8]>;
    readonly isNotifyDecodeFailed: boolean;
    readonly asNotifyDecodeFailed: ITuple<[u64, u8, u8]>;
    readonly isInvalidResponder: boolean;
    readonly asInvalidResponder: ITuple<[XcmV3MultiLocation, u64, Option<XcmV3MultiLocation>]>;
    readonly isInvalidResponderVersion: boolean;
    readonly asInvalidResponderVersion: ITuple<[XcmV3MultiLocation, u64]>;
    readonly isResponseTaken: boolean;
    readonly asResponseTaken: u64;
    readonly isAssetsTrapped: boolean;
    readonly asAssetsTrapped: ITuple<[H256, XcmV3MultiLocation, XcmVersionedMultiAssets]>;
    readonly isVersionChangeNotified: boolean;
    readonly asVersionChangeNotified: ITuple<[XcmV3MultiLocation, u32, XcmV3MultiassetMultiAssets]>;
    readonly isSupportedVersionChanged: boolean;
    readonly asSupportedVersionChanged: ITuple<[XcmV3MultiLocation, u32]>;
    readonly isNotifyTargetSendFail: boolean;
    readonly asNotifyTargetSendFail: ITuple<[XcmV3MultiLocation, u64, XcmV3TraitsError]>;
    readonly isNotifyTargetMigrationFail: boolean;
    readonly asNotifyTargetMigrationFail: ITuple<[XcmVersionedMultiLocation, u64]>;
    readonly isInvalidQuerierVersion: boolean;
    readonly asInvalidQuerierVersion: ITuple<[XcmV3MultiLocation, u64]>;
    readonly isInvalidQuerier: boolean;
    readonly asInvalidQuerier: ITuple<[XcmV3MultiLocation, u64, XcmV3MultiLocation, Option<XcmV3MultiLocation>]>;
    readonly isVersionNotifyStarted: boolean;
    readonly asVersionNotifyStarted: ITuple<[XcmV3MultiLocation, XcmV3MultiassetMultiAssets]>;
    readonly isVersionNotifyRequested: boolean;
    readonly asVersionNotifyRequested: ITuple<[XcmV3MultiLocation, XcmV3MultiassetMultiAssets]>;
    readonly isVersionNotifyUnrequested: boolean;
    readonly asVersionNotifyUnrequested: ITuple<[XcmV3MultiLocation, XcmV3MultiassetMultiAssets]>;
    readonly isFeesPaid: boolean;
    readonly asFeesPaid: ITuple<[XcmV3MultiLocation, XcmV3MultiassetMultiAssets]>;
    readonly isAssetsClaimed: boolean;
    readonly asAssetsClaimed: ITuple<[H256, XcmV3MultiLocation, XcmVersionedMultiAssets]>;
    readonly type: 'Attempted' | 'Sent' | 'UnexpectedResponse' | 'ResponseReady' | 'Notified' | 'NotifyOverweight' | 'NotifyDispatchError' | 'NotifyDecodeFailed' | 'InvalidResponder' | 'InvalidResponderVersion' | 'ResponseTaken' | 'AssetsTrapped' | 'VersionChangeNotified' | 'SupportedVersionChanged' | 'NotifyTargetSendFail' | 'NotifyTargetMigrationFail' | 'InvalidQuerierVersion' | 'InvalidQuerier' | 'VersionNotifyStarted' | 'VersionNotifyRequested' | 'VersionNotifyUnrequested' | 'FeesPaid' | 'AssetsClaimed';
  }

  /** @name XcmV3TraitsOutcome (72) */
  interface XcmV3TraitsOutcome extends Enum {
    readonly isComplete: boolean;
    readonly asComplete: SpWeightsWeightV2Weight;
    readonly isIncomplete: boolean;
    readonly asIncomplete: ITuple<[SpWeightsWeightV2Weight, XcmV3TraitsError]>;
    readonly isError: boolean;
    readonly asError: XcmV3TraitsError;
    readonly type: 'Complete' | 'Incomplete' | 'Error';
  }

  /** @name XcmV3MultiLocation (73) */
  interface XcmV3MultiLocation extends Struct {
    readonly parents: u8;
    readonly interior: XcmV3Junctions;
  }

  /** @name XcmV3Junctions (74) */
  interface XcmV3Junctions extends Enum {
    readonly isHere: boolean;
    readonly isX1: boolean;
    readonly asX1: XcmV3Junction;
    readonly isX2: boolean;
    readonly asX2: ITuple<[XcmV3Junction, XcmV3Junction]>;
    readonly isX3: boolean;
    readonly asX3: ITuple<[XcmV3Junction, XcmV3Junction, XcmV3Junction]>;
    readonly isX4: boolean;
    readonly asX4: ITuple<[XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction]>;
    readonly isX5: boolean;
    readonly asX5: ITuple<[XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction]>;
    readonly isX6: boolean;
    readonly asX6: ITuple<[XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction]>;
    readonly isX7: boolean;
    readonly asX7: ITuple<[XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction]>;
    readonly isX8: boolean;
    readonly asX8: ITuple<[XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction, XcmV3Junction]>;
    readonly type: 'Here' | 'X1' | 'X2' | 'X3' | 'X4' | 'X5' | 'X6' | 'X7' | 'X8';
  }

  /** @name XcmV3Junction (75) */
  interface XcmV3Junction extends Enum {
    readonly isParachain: boolean;
    readonly asParachain: Compact<u32>;
    readonly isAccountId32: boolean;
    readonly asAccountId32: {
      readonly network: Option<XcmV3JunctionNetworkId>;
      readonly id: U8aFixed;
    } & Struct;
    readonly isAccountIndex64: boolean;
    readonly asAccountIndex64: {
      readonly network: Option<XcmV3JunctionNetworkId>;
      readonly index: Compact<u64>;
    } & Struct;
    readonly isAccountKey20: boolean;
    readonly asAccountKey20: {
      readonly network: Option<XcmV3JunctionNetworkId>;
      readonly key: U8aFixed;
    } & Struct;
    readonly isPalletInstance: boolean;
    readonly asPalletInstance: u8;
    readonly isGeneralIndex: boolean;
    readonly asGeneralIndex: Compact<u128>;
    readonly isGeneralKey: boolean;
    readonly asGeneralKey: {
      readonly length: u8;
      readonly data: U8aFixed;
    } & Struct;
    readonly isOnlyChild: boolean;
    readonly isPlurality: boolean;
    readonly asPlurality: {
      readonly id: XcmV3JunctionBodyId;
      readonly part: XcmV3JunctionBodyPart;
    } & Struct;
    readonly isGlobalConsensus: boolean;
    readonly asGlobalConsensus: XcmV3JunctionNetworkId;
    readonly type: 'Parachain' | 'AccountId32' | 'AccountIndex64' | 'AccountKey20' | 'PalletInstance' | 'GeneralIndex' | 'GeneralKey' | 'OnlyChild' | 'Plurality' | 'GlobalConsensus';
  }

  /** @name XcmV3JunctionNetworkId (78) */
  interface XcmV3JunctionNetworkId extends Enum {
    readonly isByGenesis: boolean;
    readonly asByGenesis: U8aFixed;
    readonly isByFork: boolean;
    readonly asByFork: {
      readonly blockNumber: u64;
      readonly blockHash: U8aFixed;
    } & Struct;
    readonly isPolkadot: boolean;
    readonly isKusama: boolean;
    readonly isWestend: boolean;
    readonly isRococo: boolean;
    readonly isWococo: boolean;
    readonly isEthereum: boolean;
    readonly asEthereum: {
      readonly chainId: Compact<u64>;
    } & Struct;
    readonly isBitcoinCore: boolean;
    readonly isBitcoinCash: boolean;
    readonly type: 'ByGenesis' | 'ByFork' | 'Polkadot' | 'Kusama' | 'Westend' | 'Rococo' | 'Wococo' | 'Ethereum' | 'BitcoinCore' | 'BitcoinCash';
  }

  /** @name XcmV3JunctionBodyId (81) */
  interface XcmV3JunctionBodyId extends Enum {
    readonly isUnit: boolean;
    readonly isMoniker: boolean;
    readonly asMoniker: U8aFixed;
    readonly isIndex: boolean;
    readonly asIndex: Compact<u32>;
    readonly isExecutive: boolean;
    readonly isTechnical: boolean;
    readonly isLegislative: boolean;
    readonly isJudicial: boolean;
    readonly isDefense: boolean;
    readonly isAdministration: boolean;
    readonly isTreasury: boolean;
    readonly type: 'Unit' | 'Moniker' | 'Index' | 'Executive' | 'Technical' | 'Legislative' | 'Judicial' | 'Defense' | 'Administration' | 'Treasury';
  }

  /** @name XcmV3JunctionBodyPart (82) */
  interface XcmV3JunctionBodyPart extends Enum {
    readonly isVoice: boolean;
    readonly isMembers: boolean;
    readonly asMembers: {
      readonly count: Compact<u32>;
    } & Struct;
    readonly isFraction: boolean;
    readonly asFraction: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly isAtLeastProportion: boolean;
    readonly asAtLeastProportion: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly isMoreThanProportion: boolean;
    readonly asMoreThanProportion: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly type: 'Voice' | 'Members' | 'Fraction' | 'AtLeastProportion' | 'MoreThanProportion';
  }

  /** @name XcmV3Xcm (83) */
  interface XcmV3Xcm extends Vec<XcmV3Instruction> {}

  /** @name XcmV3Instruction (85) */
  interface XcmV3Instruction extends Enum {
    readonly isWithdrawAsset: boolean;
    readonly asWithdrawAsset: XcmV3MultiassetMultiAssets;
    readonly isReserveAssetDeposited: boolean;
    readonly asReserveAssetDeposited: XcmV3MultiassetMultiAssets;
    readonly isReceiveTeleportedAsset: boolean;
    readonly asReceiveTeleportedAsset: XcmV3MultiassetMultiAssets;
    readonly isQueryResponse: boolean;
    readonly asQueryResponse: {
      readonly queryId: Compact<u64>;
      readonly response: XcmV3Response;
      readonly maxWeight: SpWeightsWeightV2Weight;
      readonly querier: Option<XcmV3MultiLocation>;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assets: XcmV3MultiassetMultiAssets;
      readonly beneficiary: XcmV3MultiLocation;
    } & Struct;
    readonly isTransferReserveAsset: boolean;
    readonly asTransferReserveAsset: {
      readonly assets: XcmV3MultiassetMultiAssets;
      readonly dest: XcmV3MultiLocation;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly originKind: XcmV2OriginKind;
      readonly requireWeightAtMost: SpWeightsWeightV2Weight;
      readonly call: XcmDoubleEncoded;
    } & Struct;
    readonly isHrmpNewChannelOpenRequest: boolean;
    readonly asHrmpNewChannelOpenRequest: {
      readonly sender: Compact<u32>;
      readonly maxMessageSize: Compact<u32>;
      readonly maxCapacity: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelAccepted: boolean;
    readonly asHrmpChannelAccepted: {
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelClosing: boolean;
    readonly asHrmpChannelClosing: {
      readonly initiator: Compact<u32>;
      readonly sender: Compact<u32>;
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isClearOrigin: boolean;
    readonly isDescendOrigin: boolean;
    readonly asDescendOrigin: XcmV3Junctions;
    readonly isReportError: boolean;
    readonly asReportError: XcmV3QueryResponseInfo;
    readonly isDepositAsset: boolean;
    readonly asDepositAsset: {
      readonly assets: XcmV3MultiassetMultiAssetFilter;
      readonly beneficiary: XcmV3MultiLocation;
    } & Struct;
    readonly isDepositReserveAsset: boolean;
    readonly asDepositReserveAsset: {
      readonly assets: XcmV3MultiassetMultiAssetFilter;
      readonly dest: XcmV3MultiLocation;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isExchangeAsset: boolean;
    readonly asExchangeAsset: {
      readonly give: XcmV3MultiassetMultiAssetFilter;
      readonly want: XcmV3MultiassetMultiAssets;
      readonly maximal: bool;
    } & Struct;
    readonly isInitiateReserveWithdraw: boolean;
    readonly asInitiateReserveWithdraw: {
      readonly assets: XcmV3MultiassetMultiAssetFilter;
      readonly reserve: XcmV3MultiLocation;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isInitiateTeleport: boolean;
    readonly asInitiateTeleport: {
      readonly assets: XcmV3MultiassetMultiAssetFilter;
      readonly dest: XcmV3MultiLocation;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isReportHolding: boolean;
    readonly asReportHolding: {
      readonly responseInfo: XcmV3QueryResponseInfo;
      readonly assets: XcmV3MultiassetMultiAssetFilter;
    } & Struct;
    readonly isBuyExecution: boolean;
    readonly asBuyExecution: {
      readonly fees: XcmV3MultiAsset;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isRefundSurplus: boolean;
    readonly isSetErrorHandler: boolean;
    readonly asSetErrorHandler: XcmV3Xcm;
    readonly isSetAppendix: boolean;
    readonly asSetAppendix: XcmV3Xcm;
    readonly isClearError: boolean;
    readonly isClaimAsset: boolean;
    readonly asClaimAsset: {
      readonly assets: XcmV3MultiassetMultiAssets;
      readonly ticket: XcmV3MultiLocation;
    } & Struct;
    readonly isTrap: boolean;
    readonly asTrap: Compact<u64>;
    readonly isSubscribeVersion: boolean;
    readonly asSubscribeVersion: {
      readonly queryId: Compact<u64>;
      readonly maxResponseWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isUnsubscribeVersion: boolean;
    readonly isBurnAsset: boolean;
    readonly asBurnAsset: XcmV3MultiassetMultiAssets;
    readonly isExpectAsset: boolean;
    readonly asExpectAsset: XcmV3MultiassetMultiAssets;
    readonly isExpectOrigin: boolean;
    readonly asExpectOrigin: Option<XcmV3MultiLocation>;
    readonly isExpectError: boolean;
    readonly asExpectError: Option<ITuple<[u32, XcmV3TraitsError]>>;
    readonly isExpectTransactStatus: boolean;
    readonly asExpectTransactStatus: XcmV3MaybeErrorCode;
    readonly isQueryPallet: boolean;
    readonly asQueryPallet: {
      readonly moduleName: Bytes;
      readonly responseInfo: XcmV3QueryResponseInfo;
    } & Struct;
    readonly isExpectPallet: boolean;
    readonly asExpectPallet: {
      readonly index: Compact<u32>;
      readonly name: Bytes;
      readonly moduleName: Bytes;
      readonly crateMajor: Compact<u32>;
      readonly minCrateMinor: Compact<u32>;
    } & Struct;
    readonly isReportTransactStatus: boolean;
    readonly asReportTransactStatus: XcmV3QueryResponseInfo;
    readonly isClearTransactStatus: boolean;
    readonly isUniversalOrigin: boolean;
    readonly asUniversalOrigin: XcmV3Junction;
    readonly isExportMessage: boolean;
    readonly asExportMessage: {
      readonly network: XcmV3JunctionNetworkId;
      readonly destination: XcmV3Junctions;
      readonly xcm: XcmV3Xcm;
    } & Struct;
    readonly isLockAsset: boolean;
    readonly asLockAsset: {
      readonly asset: XcmV3MultiAsset;
      readonly unlocker: XcmV3MultiLocation;
    } & Struct;
    readonly isUnlockAsset: boolean;
    readonly asUnlockAsset: {
      readonly asset: XcmV3MultiAsset;
      readonly target: XcmV3MultiLocation;
    } & Struct;
    readonly isNoteUnlockable: boolean;
    readonly asNoteUnlockable: {
      readonly asset: XcmV3MultiAsset;
      readonly owner: XcmV3MultiLocation;
    } & Struct;
    readonly isRequestUnlock: boolean;
    readonly asRequestUnlock: {
      readonly asset: XcmV3MultiAsset;
      readonly locker: XcmV3MultiLocation;
    } & Struct;
    readonly isSetFeesMode: boolean;
    readonly asSetFeesMode: {
      readonly jitWithdraw: bool;
    } & Struct;
    readonly isSetTopic: boolean;
    readonly asSetTopic: U8aFixed;
    readonly isClearTopic: boolean;
    readonly isAliasOrigin: boolean;
    readonly asAliasOrigin: XcmV3MultiLocation;
    readonly isUnpaidExecution: boolean;
    readonly asUnpaidExecution: {
      readonly weightLimit: XcmV3WeightLimit;
      readonly checkOrigin: Option<XcmV3MultiLocation>;
    } & Struct;
    readonly type: 'WithdrawAsset' | 'ReserveAssetDeposited' | 'ReceiveTeleportedAsset' | 'QueryResponse' | 'TransferAsset' | 'TransferReserveAsset' | 'Transact' | 'HrmpNewChannelOpenRequest' | 'HrmpChannelAccepted' | 'HrmpChannelClosing' | 'ClearOrigin' | 'DescendOrigin' | 'ReportError' | 'DepositAsset' | 'DepositReserveAsset' | 'ExchangeAsset' | 'InitiateReserveWithdraw' | 'InitiateTeleport' | 'ReportHolding' | 'BuyExecution' | 'RefundSurplus' | 'SetErrorHandler' | 'SetAppendix' | 'ClearError' | 'ClaimAsset' | 'Trap' | 'SubscribeVersion' | 'UnsubscribeVersion' | 'BurnAsset' | 'ExpectAsset' | 'ExpectOrigin' | 'ExpectError' | 'ExpectTransactStatus' | 'QueryPallet' | 'ExpectPallet' | 'ReportTransactStatus' | 'ClearTransactStatus' | 'UniversalOrigin' | 'ExportMessage' | 'LockAsset' | 'UnlockAsset' | 'NoteUnlockable' | 'RequestUnlock' | 'SetFeesMode' | 'SetTopic' | 'ClearTopic' | 'AliasOrigin' | 'UnpaidExecution';
  }

  /** @name XcmV3MultiassetMultiAssets (86) */
  interface XcmV3MultiassetMultiAssets extends Vec<XcmV3MultiAsset> {}

  /** @name XcmV3MultiAsset (88) */
  interface XcmV3MultiAsset extends Struct {
    readonly id: XcmV3MultiassetAssetId;
    readonly fun: XcmV3MultiassetFungibility;
  }

  /** @name XcmV3MultiassetAssetId (89) */
  interface XcmV3MultiassetAssetId extends Enum {
    readonly isConcrete: boolean;
    readonly asConcrete: XcmV3MultiLocation;
    readonly isAbstract: boolean;
    readonly asAbstract: U8aFixed;
    readonly type: 'Concrete' | 'Abstract';
  }

  /** @name XcmV3MultiassetFungibility (90) */
  interface XcmV3MultiassetFungibility extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: Compact<u128>;
    readonly isNonFungible: boolean;
    readonly asNonFungible: XcmV3MultiassetAssetInstance;
    readonly type: 'Fungible' | 'NonFungible';
  }

  /** @name XcmV3MultiassetAssetInstance (91) */
  interface XcmV3MultiassetAssetInstance extends Enum {
    readonly isUndefined: boolean;
    readonly isIndex: boolean;
    readonly asIndex: Compact<u128>;
    readonly isArray4: boolean;
    readonly asArray4: U8aFixed;
    readonly isArray8: boolean;
    readonly asArray8: U8aFixed;
    readonly isArray16: boolean;
    readonly asArray16: U8aFixed;
    readonly isArray32: boolean;
    readonly asArray32: U8aFixed;
    readonly type: 'Undefined' | 'Index' | 'Array4' | 'Array8' | 'Array16' | 'Array32';
  }

  /** @name XcmV3Response (94) */
  interface XcmV3Response extends Enum {
    readonly isNull: boolean;
    readonly isAssets: boolean;
    readonly asAssets: XcmV3MultiassetMultiAssets;
    readonly isExecutionResult: boolean;
    readonly asExecutionResult: Option<ITuple<[u32, XcmV3TraitsError]>>;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly isPalletsInfo: boolean;
    readonly asPalletsInfo: Vec<XcmV3PalletInfo>;
    readonly isDispatchResult: boolean;
    readonly asDispatchResult: XcmV3MaybeErrorCode;
    readonly type: 'Null' | 'Assets' | 'ExecutionResult' | 'Version' | 'PalletsInfo' | 'DispatchResult';
  }

  /** @name XcmV3PalletInfo (98) */
  interface XcmV3PalletInfo extends Struct {
    readonly index: Compact<u32>;
    readonly name: Bytes;
    readonly moduleName: Bytes;
    readonly major: Compact<u32>;
    readonly minor: Compact<u32>;
    readonly patch: Compact<u32>;
  }

  /** @name XcmV3MaybeErrorCode (101) */
  interface XcmV3MaybeErrorCode extends Enum {
    readonly isSuccess: boolean;
    readonly isError: boolean;
    readonly asError: Bytes;
    readonly isTruncatedError: boolean;
    readonly asTruncatedError: Bytes;
    readonly type: 'Success' | 'Error' | 'TruncatedError';
  }

  /** @name XcmV2OriginKind (104) */
  interface XcmV2OriginKind extends Enum {
    readonly isNative: boolean;
    readonly isSovereignAccount: boolean;
    readonly isSuperuser: boolean;
    readonly isXcm: boolean;
    readonly type: 'Native' | 'SovereignAccount' | 'Superuser' | 'Xcm';
  }

  /** @name XcmDoubleEncoded (105) */
  interface XcmDoubleEncoded extends Struct {
    readonly encoded: Bytes;
  }

  /** @name XcmV3QueryResponseInfo (106) */
  interface XcmV3QueryResponseInfo extends Struct {
    readonly destination: XcmV3MultiLocation;
    readonly queryId: Compact<u64>;
    readonly maxWeight: SpWeightsWeightV2Weight;
  }

  /** @name XcmV3MultiassetMultiAssetFilter (107) */
  interface XcmV3MultiassetMultiAssetFilter extends Enum {
    readonly isDefinite: boolean;
    readonly asDefinite: XcmV3MultiassetMultiAssets;
    readonly isWild: boolean;
    readonly asWild: XcmV3MultiassetWildMultiAsset;
    readonly type: 'Definite' | 'Wild';
  }

  /** @name XcmV3MultiassetWildMultiAsset (108) */
  interface XcmV3MultiassetWildMultiAsset extends Enum {
    readonly isAll: boolean;
    readonly isAllOf: boolean;
    readonly asAllOf: {
      readonly id: XcmV3MultiassetAssetId;
      readonly fun: XcmV3MultiassetWildFungibility;
    } & Struct;
    readonly isAllCounted: boolean;
    readonly asAllCounted: Compact<u32>;
    readonly isAllOfCounted: boolean;
    readonly asAllOfCounted: {
      readonly id: XcmV3MultiassetAssetId;
      readonly fun: XcmV3MultiassetWildFungibility;
      readonly count: Compact<u32>;
    } & Struct;
    readonly type: 'All' | 'AllOf' | 'AllCounted' | 'AllOfCounted';
  }

  /** @name XcmV3MultiassetWildFungibility (109) */
  interface XcmV3MultiassetWildFungibility extends Enum {
    readonly isFungible: boolean;
    readonly isNonFungible: boolean;
    readonly type: 'Fungible' | 'NonFungible';
  }

  /** @name XcmV3WeightLimit (110) */
  interface XcmV3WeightLimit extends Enum {
    readonly isUnlimited: boolean;
    readonly isLimited: boolean;
    readonly asLimited: SpWeightsWeightV2Weight;
    readonly type: 'Unlimited' | 'Limited';
  }

  /** @name XcmVersionedMultiAssets (111) */
  interface XcmVersionedMultiAssets extends Enum {
    readonly isV2: boolean;
    readonly asV2: XcmV2MultiassetMultiAssets;
    readonly isV3: boolean;
    readonly asV3: XcmV3MultiassetMultiAssets;
    readonly type: 'V2' | 'V3';
  }

  /** @name XcmV2MultiassetMultiAssets (112) */
  interface XcmV2MultiassetMultiAssets extends Vec<XcmV2MultiAsset> {}

  /** @name XcmV2MultiAsset (114) */
  interface XcmV2MultiAsset extends Struct {
    readonly id: XcmV2MultiassetAssetId;
    readonly fun: XcmV2MultiassetFungibility;
  }

  /** @name XcmV2MultiassetAssetId (115) */
  interface XcmV2MultiassetAssetId extends Enum {
    readonly isConcrete: boolean;
    readonly asConcrete: XcmV2MultiLocation;
    readonly isAbstract: boolean;
    readonly asAbstract: Bytes;
    readonly type: 'Concrete' | 'Abstract';
  }

  /** @name XcmV2MultiLocation (116) */
  interface XcmV2MultiLocation extends Struct {
    readonly parents: u8;
    readonly interior: XcmV2MultilocationJunctions;
  }

  /** @name XcmV2MultilocationJunctions (117) */
  interface XcmV2MultilocationJunctions extends Enum {
    readonly isHere: boolean;
    readonly isX1: boolean;
    readonly asX1: XcmV2Junction;
    readonly isX2: boolean;
    readonly asX2: ITuple<[XcmV2Junction, XcmV2Junction]>;
    readonly isX3: boolean;
    readonly asX3: ITuple<[XcmV2Junction, XcmV2Junction, XcmV2Junction]>;
    readonly isX4: boolean;
    readonly asX4: ITuple<[XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction]>;
    readonly isX5: boolean;
    readonly asX5: ITuple<[XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction]>;
    readonly isX6: boolean;
    readonly asX6: ITuple<[XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction]>;
    readonly isX7: boolean;
    readonly asX7: ITuple<[XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction]>;
    readonly isX8: boolean;
    readonly asX8: ITuple<[XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction, XcmV2Junction]>;
    readonly type: 'Here' | 'X1' | 'X2' | 'X3' | 'X4' | 'X5' | 'X6' | 'X7' | 'X8';
  }

  /** @name XcmV2Junction (118) */
  interface XcmV2Junction extends Enum {
    readonly isParachain: boolean;
    readonly asParachain: Compact<u32>;
    readonly isAccountId32: boolean;
    readonly asAccountId32: {
      readonly network: XcmV2NetworkId;
      readonly id: U8aFixed;
    } & Struct;
    readonly isAccountIndex64: boolean;
    readonly asAccountIndex64: {
      readonly network: XcmV2NetworkId;
      readonly index: Compact<u64>;
    } & Struct;
    readonly isAccountKey20: boolean;
    readonly asAccountKey20: {
      readonly network: XcmV2NetworkId;
      readonly key: U8aFixed;
    } & Struct;
    readonly isPalletInstance: boolean;
    readonly asPalletInstance: u8;
    readonly isGeneralIndex: boolean;
    readonly asGeneralIndex: Compact<u128>;
    readonly isGeneralKey: boolean;
    readonly asGeneralKey: Bytes;
    readonly isOnlyChild: boolean;
    readonly isPlurality: boolean;
    readonly asPlurality: {
      readonly id: XcmV2BodyId;
      readonly part: XcmV2BodyPart;
    } & Struct;
    readonly type: 'Parachain' | 'AccountId32' | 'AccountIndex64' | 'AccountKey20' | 'PalletInstance' | 'GeneralIndex' | 'GeneralKey' | 'OnlyChild' | 'Plurality';
  }

  /** @name XcmV2NetworkId (119) */
  interface XcmV2NetworkId extends Enum {
    readonly isAny: boolean;
    readonly isNamed: boolean;
    readonly asNamed: Bytes;
    readonly isPolkadot: boolean;
    readonly isKusama: boolean;
    readonly type: 'Any' | 'Named' | 'Polkadot' | 'Kusama';
  }

  /** @name XcmV2BodyId (121) */
  interface XcmV2BodyId extends Enum {
    readonly isUnit: boolean;
    readonly isNamed: boolean;
    readonly asNamed: Bytes;
    readonly isIndex: boolean;
    readonly asIndex: Compact<u32>;
    readonly isExecutive: boolean;
    readonly isTechnical: boolean;
    readonly isLegislative: boolean;
    readonly isJudicial: boolean;
    readonly isDefense: boolean;
    readonly isAdministration: boolean;
    readonly isTreasury: boolean;
    readonly type: 'Unit' | 'Named' | 'Index' | 'Executive' | 'Technical' | 'Legislative' | 'Judicial' | 'Defense' | 'Administration' | 'Treasury';
  }

  /** @name XcmV2BodyPart (122) */
  interface XcmV2BodyPart extends Enum {
    readonly isVoice: boolean;
    readonly isMembers: boolean;
    readonly asMembers: {
      readonly count: Compact<u32>;
    } & Struct;
    readonly isFraction: boolean;
    readonly asFraction: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly isAtLeastProportion: boolean;
    readonly asAtLeastProportion: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly isMoreThanProportion: boolean;
    readonly asMoreThanProportion: {
      readonly nom: Compact<u32>;
      readonly denom: Compact<u32>;
    } & Struct;
    readonly type: 'Voice' | 'Members' | 'Fraction' | 'AtLeastProportion' | 'MoreThanProportion';
  }

  /** @name XcmV2MultiassetFungibility (123) */
  interface XcmV2MultiassetFungibility extends Enum {
    readonly isFungible: boolean;
    readonly asFungible: Compact<u128>;
    readonly isNonFungible: boolean;
    readonly asNonFungible: XcmV2MultiassetAssetInstance;
    readonly type: 'Fungible' | 'NonFungible';
  }

  /** @name XcmV2MultiassetAssetInstance (124) */
  interface XcmV2MultiassetAssetInstance extends Enum {
    readonly isUndefined: boolean;
    readonly isIndex: boolean;
    readonly asIndex: Compact<u128>;
    readonly isArray4: boolean;
    readonly asArray4: U8aFixed;
    readonly isArray8: boolean;
    readonly asArray8: U8aFixed;
    readonly isArray16: boolean;
    readonly asArray16: U8aFixed;
    readonly isArray32: boolean;
    readonly asArray32: U8aFixed;
    readonly isBlob: boolean;
    readonly asBlob: Bytes;
    readonly type: 'Undefined' | 'Index' | 'Array4' | 'Array8' | 'Array16' | 'Array32' | 'Blob';
  }

  /** @name XcmVersionedMultiLocation (125) */
  interface XcmVersionedMultiLocation extends Enum {
    readonly isV2: boolean;
    readonly asV2: XcmV2MultiLocation;
    readonly isV3: boolean;
    readonly asV3: XcmV3MultiLocation;
    readonly type: 'V2' | 'V3';
  }

  /** @name CumulusPalletXcmEvent (126) */
  interface CumulusPalletXcmEvent extends Enum {
    readonly isInvalidFormat: boolean;
    readonly asInvalidFormat: U8aFixed;
    readonly isUnsupportedVersion: boolean;
    readonly asUnsupportedVersion: U8aFixed;
    readonly isExecutedDownward: boolean;
    readonly asExecutedDownward: ITuple<[U8aFixed, XcmV3TraitsOutcome]>;
    readonly type: 'InvalidFormat' | 'UnsupportedVersion' | 'ExecutedDownward';
  }

  /** @name CumulusPalletDmpQueueEvent (127) */
  interface CumulusPalletDmpQueueEvent extends Enum {
    readonly isInvalidFormat: boolean;
    readonly asInvalidFormat: {
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isUnsupportedVersion: boolean;
    readonly asUnsupportedVersion: {
      readonly messageId: U8aFixed;
    } & Struct;
    readonly isExecutedDownward: boolean;
    readonly asExecutedDownward: {
      readonly messageId: U8aFixed;
      readonly outcome: XcmV3TraitsOutcome;
    } & Struct;
    readonly isWeightExhausted: boolean;
    readonly asWeightExhausted: {
      readonly messageId: U8aFixed;
      readonly remainingWeight: SpWeightsWeightV2Weight;
      readonly requiredWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isOverweightEnqueued: boolean;
    readonly asOverweightEnqueued: {
      readonly messageId: U8aFixed;
      readonly overweightIndex: u64;
      readonly requiredWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isOverweightServiced: boolean;
    readonly asOverweightServiced: {
      readonly overweightIndex: u64;
      readonly weightUsed: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isMaxMessagesExhausted: boolean;
    readonly asMaxMessagesExhausted: {
      readonly messageId: U8aFixed;
    } & Struct;
    readonly type: 'InvalidFormat' | 'UnsupportedVersion' | 'ExecutedDownward' | 'WeightExhausted' | 'OverweightEnqueued' | 'OverweightServiced' | 'MaxMessagesExhausted';
  }

  /** @name OrmlXtokensModuleEvent (128) */
  interface OrmlXtokensModuleEvent extends Enum {
    readonly isTransferredMultiAssets: boolean;
    readonly asTransferredMultiAssets: {
      readonly sender: AccountId32;
      readonly assets: XcmV3MultiassetMultiAssets;
      readonly fee: XcmV3MultiAsset;
      readonly dest: XcmV3MultiLocation;
    } & Struct;
    readonly type: 'TransferredMultiAssets';
  }

  /** @name OrmlTokensModuleEvent (129) */
  interface OrmlTokensModuleEvent extends Enum {
    readonly isEndowed: boolean;
    readonly asEndowed: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isDustLost: boolean;
    readonly asDustLost: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly currencyId: u128;
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isReserved: boolean;
    readonly asReserved: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnreserved: boolean;
    readonly asUnreserved: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isReserveRepatriated: boolean;
    readonly asReserveRepatriated: {
      readonly currencyId: u128;
      readonly from: AccountId32;
      readonly to: AccountId32;
      readonly amount: u128;
      readonly status: FrameSupportTokensMiscBalanceStatus;
    } & Struct;
    readonly isBalanceSet: boolean;
    readonly asBalanceSet: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly free: u128;
      readonly reserved: u128;
    } & Struct;
    readonly isTotalIssuanceSet: boolean;
    readonly asTotalIssuanceSet: {
      readonly currencyId: u128;
      readonly amount: u128;
    } & Struct;
    readonly isWithdrawn: boolean;
    readonly asWithdrawn: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isSlashed: boolean;
    readonly asSlashed: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly freeAmount: u128;
      readonly reservedAmount: u128;
    } & Struct;
    readonly isDeposited: boolean;
    readonly asDeposited: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isLockSet: boolean;
    readonly asLockSet: {
      readonly lockId: U8aFixed;
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isLockRemoved: boolean;
    readonly asLockRemoved: {
      readonly lockId: U8aFixed;
      readonly currencyId: u128;
      readonly who: AccountId32;
    } & Struct;
    readonly isLocked: boolean;
    readonly asLocked: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isUnlocked: boolean;
    readonly asUnlocked: {
      readonly currencyId: u128;
      readonly who: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'Endowed' | 'DustLost' | 'Transfer' | 'Reserved' | 'Unreserved' | 'ReserveRepatriated' | 'BalanceSet' | 'TotalIssuanceSet' | 'Withdrawn' | 'Slashed' | 'Deposited' | 'LockSet' | 'LockRemoved' | 'Locked' | 'Unlocked';
  }

  /** @name PalletBridgeEvent (130) */
  interface PalletBridgeEvent extends Enum {
    readonly isRelayerThresholdChanged: boolean;
    readonly asRelayerThresholdChanged: u32;
    readonly isChainWhitelisted: boolean;
    readonly asChainWhitelisted: u8;
    readonly isRelayerAdded: boolean;
    readonly asRelayerAdded: AccountId32;
    readonly isRelayerRemoved: boolean;
    readonly asRelayerRemoved: AccountId32;
    readonly isFungibleTransfer: boolean;
    readonly asFungibleTransfer: ITuple<[u8, u64, U8aFixed, u128, Bytes]>;
    readonly isNonFungibleTransfer: boolean;
    readonly asNonFungibleTransfer: ITuple<[u8, u64, U8aFixed, Bytes, Bytes, Bytes]>;
    readonly isGenericTransfer: boolean;
    readonly asGenericTransfer: ITuple<[u8, u64, U8aFixed, Bytes]>;
    readonly isVoteFor: boolean;
    readonly asVoteFor: ITuple<[u8, u64, AccountId32]>;
    readonly isVoteAgainst: boolean;
    readonly asVoteAgainst: ITuple<[u8, u64, AccountId32]>;
    readonly isProposalApproved: boolean;
    readonly asProposalApproved: ITuple<[u8, u64]>;
    readonly isProposalRejected: boolean;
    readonly asProposalRejected: ITuple<[u8, u64]>;
    readonly isProposalSucceeded: boolean;
    readonly asProposalSucceeded: ITuple<[u8, u64]>;
    readonly isProposalFailed: boolean;
    readonly asProposalFailed: ITuple<[u8, u64]>;
    readonly isFeeUpdated: boolean;
    readonly asFeeUpdated: {
      readonly destId: u8;
      readonly fee: u128;
    } & Struct;
    readonly type: 'RelayerThresholdChanged' | 'ChainWhitelisted' | 'RelayerAdded' | 'RelayerRemoved' | 'FungibleTransfer' | 'NonFungibleTransfer' | 'GenericTransfer' | 'VoteFor' | 'VoteAgainst' | 'ProposalApproved' | 'ProposalRejected' | 'ProposalSucceeded' | 'ProposalFailed' | 'FeeUpdated';
  }

  /** @name PalletBridgeTransferEvent (131) */
  interface PalletBridgeTransferEvent extends Enum {
    readonly isMaximumIssuanceChanged: boolean;
    readonly asMaximumIssuanceChanged: {
      readonly oldValue: u128;
    } & Struct;
    readonly isNativeTokenMinted: boolean;
    readonly asNativeTokenMinted: {
      readonly to: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly type: 'MaximumIssuanceChanged' | 'NativeTokenMinted';
  }

  /** @name PalletExtrinsicFilterEvent (132) */
  interface PalletExtrinsicFilterEvent extends Enum {
    readonly isModeSet: boolean;
    readonly asModeSet: {
      readonly newMode: PalletExtrinsicFilterOperationalMode;
    } & Struct;
    readonly isExtrinsicsBlocked: boolean;
    readonly asExtrinsicsBlocked: {
      readonly palletNameBytes: Bytes;
      readonly functionNameBytes: Option<Bytes>;
    } & Struct;
    readonly isExtrinsicsUnblocked: boolean;
    readonly asExtrinsicsUnblocked: {
      readonly palletNameBytes: Bytes;
      readonly functionNameBytes: Option<Bytes>;
    } & Struct;
    readonly type: 'ModeSet' | 'ExtrinsicsBlocked' | 'ExtrinsicsUnblocked';
  }

  /** @name PalletExtrinsicFilterOperationalMode (133) */
  interface PalletExtrinsicFilterOperationalMode extends Enum {
    readonly isNormal: boolean;
    readonly isSafe: boolean;
    readonly isTest: boolean;
    readonly type: 'Normal' | 'Safe' | 'Test';
  }

  /** @name PalletIdentityManagementEvent (135) */
  interface PalletIdentityManagementEvent extends Enum {
    readonly isDelegateeAdded: boolean;
    readonly asDelegateeAdded: {
      readonly account: AccountId32;
    } & Struct;
    readonly isDelegateeRemoved: boolean;
    readonly asDelegateeRemoved: {
      readonly account: AccountId32;
    } & Struct;
    readonly isLinkIdentityRequested: boolean;
    readonly asLinkIdentityRequested: {
      readonly shard: H256;
      readonly account: AccountId32;
      readonly encryptedIdentity: Bytes;
      readonly encryptedValidationData: Bytes;
      readonly encryptedWeb3networks: Bytes;
    } & Struct;
    readonly isDeactivateIdentityRequested: boolean;
    readonly asDeactivateIdentityRequested: {
      readonly shard: H256;
      readonly account: AccountId32;
      readonly encryptedIdentity: Bytes;
    } & Struct;
    readonly isActivateIdentityRequested: boolean;
    readonly asActivateIdentityRequested: {
      readonly shard: H256;
      readonly account: AccountId32;
      readonly encryptedIdentity: Bytes;
    } & Struct;
    readonly isIdentityLinked: boolean;
    readonly asIdentityLinked: {
      readonly primeIdentity: CorePrimitivesIdentity;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isIdentityDeactivated: boolean;
    readonly asIdentityDeactivated: {
      readonly primeIdentity: CorePrimitivesIdentity;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isIdentityActivated: boolean;
    readonly asIdentityActivated: {
      readonly primeIdentity: CorePrimitivesIdentity;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isIdentityNetworksSet: boolean;
    readonly asIdentityNetworksSet: {
      readonly primeIdentity: CorePrimitivesIdentity;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isLinkIdentityFailed: boolean;
    readonly asLinkIdentityFailed: {
      readonly primeIdentity: Option<CorePrimitivesIdentity>;
      readonly detail: CorePrimitivesErrorErrorDetail;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isDeactivateIdentityFailed: boolean;
    readonly asDeactivateIdentityFailed: {
      readonly primeIdentity: Option<CorePrimitivesIdentity>;
      readonly detail: CorePrimitivesErrorErrorDetail;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isActivateIdentityFailed: boolean;
    readonly asActivateIdentityFailed: {
      readonly primeIdentity: Option<CorePrimitivesIdentity>;
      readonly detail: CorePrimitivesErrorErrorDetail;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isUnclassifiedError: boolean;
    readonly asUnclassifiedError: {
      readonly primeIdentity: Option<CorePrimitivesIdentity>;
      readonly detail: CorePrimitivesErrorErrorDetail;
      readonly reqExtHash: H256;
    } & Struct;
    readonly type: 'DelegateeAdded' | 'DelegateeRemoved' | 'LinkIdentityRequested' | 'DeactivateIdentityRequested' | 'ActivateIdentityRequested' | 'IdentityLinked' | 'IdentityDeactivated' | 'IdentityActivated' | 'IdentityNetworksSet' | 'LinkIdentityFailed' | 'DeactivateIdentityFailed' | 'ActivateIdentityFailed' | 'UnclassifiedError';
  }

  /** @name CorePrimitivesIdentity (136) */
  interface CorePrimitivesIdentity extends Enum {
    readonly isTwitter: boolean;
    readonly asTwitter: Bytes;
    readonly isDiscord: boolean;
    readonly asDiscord: Bytes;
    readonly isGithub: boolean;
    readonly asGithub: Bytes;
    readonly isSubstrate: boolean;
    readonly asSubstrate: CorePrimitivesIdentityAddress32;
    readonly isEvm: boolean;
    readonly asEvm: CorePrimitivesIdentityAddress20;
    readonly isBitcoin: boolean;
    readonly asBitcoin: CorePrimitivesIdentityAddress33;
    readonly isSolana: boolean;
    readonly asSolana: CorePrimitivesIdentityAddress32;
    readonly type: 'Twitter' | 'Discord' | 'Github' | 'Substrate' | 'Evm' | 'Bitcoin' | 'Solana';
  }

  /** @name CorePrimitivesIdentityAddress32 (138) */
  interface CorePrimitivesIdentityAddress32 extends U8aFixed {}

  /** @name CorePrimitivesIdentityAddress20 (139) */
  interface CorePrimitivesIdentityAddress20 extends U8aFixed {}

  /** @name CorePrimitivesIdentityAddress33 (140) */
  interface CorePrimitivesIdentityAddress33 extends U8aFixed {}

  /** @name CorePrimitivesErrorErrorDetail (143) */
  interface CorePrimitivesErrorErrorDetail extends Enum {
    readonly isImportError: boolean;
    readonly isUnauthorizedSigner: boolean;
    readonly isStfError: boolean;
    readonly asStfError: Bytes;
    readonly isSendStfRequestFailed: boolean;
    readonly isParseError: boolean;
    readonly isDataProviderError: boolean;
    readonly asDataProviderError: Bytes;
    readonly isInvalidIdentity: boolean;
    readonly isWrongWeb2Handle: boolean;
    readonly isUnexpectedMessage: boolean;
    readonly isVerifyWeb3SignatureFailed: boolean;
    readonly isNoEligibleIdentity: boolean;
    readonly type: 'ImportError' | 'UnauthorizedSigner' | 'StfError' | 'SendStfRequestFailed' | 'ParseError' | 'DataProviderError' | 'InvalidIdentity' | 'WrongWeb2Handle' | 'UnexpectedMessage' | 'VerifyWeb3SignatureFailed' | 'NoEligibleIdentity';
  }

  /** @name PalletAssetManagerEvent (145) */
  interface PalletAssetManagerEvent extends Enum {
    readonly isForeignAssetMetadataUpdated: boolean;
    readonly asForeignAssetMetadataUpdated: {
      readonly assetId: u128;
      readonly metadata: PalletAssetManagerAssetMetadata;
    } & Struct;
    readonly isForeignAssetTrackerUpdated: boolean;
    readonly asForeignAssetTrackerUpdated: {
      readonly oldAssetTracker: u128;
      readonly newAssetTracker: u128;
    } & Struct;
    readonly isForeignAssetTypeRegistered: boolean;
    readonly asForeignAssetTypeRegistered: {
      readonly assetId: u128;
      readonly assetType: RuntimeCommonXcmImplCurrencyId;
    } & Struct;
    readonly isForeignAssetTypeRemoved: boolean;
    readonly asForeignAssetTypeRemoved: {
      readonly assetId: u128;
      readonly removedAssetType: RuntimeCommonXcmImplCurrencyId;
      readonly defaultAssetType: RuntimeCommonXcmImplCurrencyId;
    } & Struct;
    readonly isUnitsPerSecondChanged: boolean;
    readonly asUnitsPerSecondChanged: {
      readonly assetId: u128;
      readonly unitsPerSecond: u128;
    } & Struct;
    readonly type: 'ForeignAssetMetadataUpdated' | 'ForeignAssetTrackerUpdated' | 'ForeignAssetTypeRegistered' | 'ForeignAssetTypeRemoved' | 'UnitsPerSecondChanged';
  }

  /** @name PalletAssetManagerAssetMetadata (146) */
  interface PalletAssetManagerAssetMetadata extends Struct {
    readonly name: Bytes;
    readonly symbol: Bytes;
    readonly decimals: u8;
    readonly minimalBalance: u128;
    readonly isFrozen: bool;
  }

  /** @name RuntimeCommonXcmImplCurrencyId (147) */
  interface RuntimeCommonXcmImplCurrencyId extends Enum {
    readonly isSelfReserve: boolean;
    readonly isParachainReserve: boolean;
    readonly asParachainReserve: XcmV3MultiLocation;
    readonly type: 'SelfReserve' | 'ParachainReserve';
  }

  /** @name RococoParachainRuntimeRuntime (148) */
  type RococoParachainRuntimeRuntime = Null;

  /** @name PalletVcManagementEvent (149) */
  interface PalletVcManagementEvent extends Enum {
    readonly isDelegateeAdded: boolean;
    readonly asDelegateeAdded: {
      readonly account: AccountId32;
    } & Struct;
    readonly isDelegateeRemoved: boolean;
    readonly asDelegateeRemoved: {
      readonly account: AccountId32;
    } & Struct;
    readonly isVcRequested: boolean;
    readonly asVcRequested: {
      readonly account: AccountId32;
      readonly shard: H256;
      readonly assertion: CorePrimitivesAssertion;
    } & Struct;
    readonly isVcIssued: boolean;
    readonly asVcIssued: {
      readonly identity: CorePrimitivesIdentity;
      readonly assertion: CorePrimitivesAssertion;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isAdminChanged: boolean;
    readonly asAdminChanged: {
      readonly oldAdmin: Option<AccountId32>;
      readonly newAdmin: Option<AccountId32>;
    } & Struct;
    readonly isSchemaIssued: boolean;
    readonly asSchemaIssued: {
      readonly account: AccountId32;
      readonly shard: H256;
      readonly index: u64;
    } & Struct;
    readonly isSchemaDisabled: boolean;
    readonly asSchemaDisabled: {
      readonly account: AccountId32;
      readonly shard: H256;
      readonly index: u64;
    } & Struct;
    readonly isSchemaActivated: boolean;
    readonly asSchemaActivated: {
      readonly account: AccountId32;
      readonly shard: H256;
      readonly index: u64;
    } & Struct;
    readonly isSchemaRevoked: boolean;
    readonly asSchemaRevoked: {
      readonly account: AccountId32;
      readonly shard: H256;
      readonly index: u64;
    } & Struct;
    readonly isRequestVCFailed: boolean;
    readonly asRequestVCFailed: {
      readonly identity: Option<CorePrimitivesIdentity>;
      readonly assertion: CorePrimitivesAssertion;
      readonly detail: CorePrimitivesErrorErrorDetail;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isUnclassifiedError: boolean;
    readonly asUnclassifiedError: {
      readonly identity: Option<CorePrimitivesIdentity>;
      readonly detail: CorePrimitivesErrorErrorDetail;
      readonly reqExtHash: H256;
    } & Struct;
    readonly type: 'DelegateeAdded' | 'DelegateeRemoved' | 'VcRequested' | 'VcIssued' | 'AdminChanged' | 'SchemaIssued' | 'SchemaDisabled' | 'SchemaActivated' | 'SchemaRevoked' | 'RequestVCFailed' | 'UnclassifiedError';
  }

  /** @name CorePrimitivesAssertion (150) */
  interface CorePrimitivesAssertion extends Enum {
    readonly isA1: boolean;
    readonly isA2: boolean;
    readonly asA2: Bytes;
    readonly isA3: boolean;
    readonly asA3: ITuple<[Bytes, Bytes, Bytes]>;
    readonly isA4: boolean;
    readonly asA4: Bytes;
    readonly isA6: boolean;
    readonly isA7: boolean;
    readonly asA7: Bytes;
    readonly isA8: boolean;
    readonly asA8: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly isA10: boolean;
    readonly asA10: Bytes;
    readonly isA11: boolean;
    readonly asA11: Bytes;
    readonly isA13: boolean;
    readonly asA13: AccountId32;
    readonly isA14: boolean;
    readonly isAchainable: boolean;
    readonly asAchainable: CorePrimitivesAssertionAchainableAchainableParams;
    readonly isA20: boolean;
    readonly isOneBlock: boolean;
    readonly asOneBlock: CorePrimitivesAssertionOneblockOneBlockCourseType;
    readonly isGenericDiscordRole: boolean;
    readonly asGenericDiscordRole: CorePrimitivesAssertionGenericDiscordRoleGenericDiscordRoleType;
    readonly isBnbDomainHolding: boolean;
    readonly isBnbDigitDomainClub: boolean;
    readonly asBnbDigitDomainClub: CorePrimitivesAssertionBnbDomainBnbDigitDomainType;
    readonly isVip3MembershipCard: boolean;
    readonly asVip3MembershipCard: CorePrimitivesAssertionVip3Vip3MembershipCardLevel;
    readonly isWeirdoGhostGangHolder: boolean;
    readonly isLitStaking: boolean;
    readonly isEvmAmountHolding: boolean;
    readonly asEvmAmountHolding: CorePrimitivesAssertionEvmAmountHoldingEvmTokenType;
    readonly isBrc20AmountHolder: boolean;
    readonly isCryptoSummary: boolean;
    readonly isTokenHoldingAmount: boolean;
    readonly asTokenHoldingAmount: CorePrimitivesAssertionWeb3TokenWeb3TokenType;
    readonly isPlatformUser: boolean;
    readonly asPlatformUser: CorePrimitivesAssertionPlatformUserPlatformUserType;
    readonly isNftHolder: boolean;
    readonly asNftHolder: CorePrimitivesAssertionWeb3NftWeb3NftType;
    readonly isDynamic: boolean;
    readonly asDynamic: CorePrimitivesAssertionDynamicDynamicParams;
    readonly type: 'A1' | 'A2' | 'A3' | 'A4' | 'A6' | 'A7' | 'A8' | 'A10' | 'A11' | 'A13' | 'A14' | 'Achainable' | 'A20' | 'OneBlock' | 'GenericDiscordRole' | 'BnbDomainHolding' | 'BnbDigitDomainClub' | 'Vip3MembershipCard' | 'WeirdoGhostGangHolder' | 'LitStaking' | 'EvmAmountHolding' | 'Brc20AmountHolder' | 'CryptoSummary' | 'TokenHoldingAmount' | 'PlatformUser' | 'NftHolder' | 'Dynamic';
  }

  /** @name CorePrimitivesAssertionNetworkWeb3Network (152) */
  interface CorePrimitivesAssertionNetworkWeb3Network extends Enum {
    readonly isPolkadot: boolean;
    readonly isKusama: boolean;
    readonly isLitentry: boolean;
    readonly isLitmus: boolean;
    readonly isLitentryRococo: boolean;
    readonly isKhala: boolean;
    readonly isSubstrateTestnet: boolean;
    readonly isEthereum: boolean;
    readonly isBsc: boolean;
    readonly isBitcoinP2tr: boolean;
    readonly isBitcoinP2pkh: boolean;
    readonly isBitcoinP2sh: boolean;
    readonly isBitcoinP2wpkh: boolean;
    readonly isBitcoinP2wsh: boolean;
    readonly isPolygon: boolean;
    readonly isArbitrum: boolean;
    readonly isSolana: boolean;
    readonly isCombo: boolean;
    readonly type: 'Polkadot' | 'Kusama' | 'Litentry' | 'Litmus' | 'LitentryRococo' | 'Khala' | 'SubstrateTestnet' | 'Ethereum' | 'Bsc' | 'BitcoinP2tr' | 'BitcoinP2pkh' | 'BitcoinP2sh' | 'BitcoinP2wpkh' | 'BitcoinP2wsh' | 'Polygon' | 'Arbitrum' | 'Solana' | 'Combo';
  }

  /** @name CorePrimitivesAssertionAchainableAchainableParams (154) */
  interface CorePrimitivesAssertionAchainableAchainableParams extends Enum {
    readonly isAmountHolding: boolean;
    readonly asAmountHolding: CorePrimitivesAssertionAchainableAchainableAmountHolding;
    readonly isAmountToken: boolean;
    readonly asAmountToken: CorePrimitivesAssertionAchainableAchainableAmountToken;
    readonly isAmount: boolean;
    readonly asAmount: CorePrimitivesAssertionAchainableAchainableAmount;
    readonly isAmounts: boolean;
    readonly asAmounts: CorePrimitivesAssertionAchainableAchainableAmounts;
    readonly isBasic: boolean;
    readonly asBasic: CorePrimitivesAssertionAchainableAchainableBasic;
    readonly isBetweenPercents: boolean;
    readonly asBetweenPercents: CorePrimitivesAssertionAchainableAchainableBetweenPercents;
    readonly isClassOfYear: boolean;
    readonly asClassOfYear: CorePrimitivesAssertionAchainableAchainableClassOfYear;
    readonly isDateInterval: boolean;
    readonly asDateInterval: CorePrimitivesAssertionAchainableAchainableDateInterval;
    readonly isDatePercent: boolean;
    readonly asDatePercent: CorePrimitivesAssertionAchainableAchainableDatePercent;
    readonly isDate: boolean;
    readonly asDate: CorePrimitivesAssertionAchainableAchainableDate;
    readonly isToken: boolean;
    readonly asToken: CorePrimitivesAssertionAchainableAchainableToken;
    readonly isMirror: boolean;
    readonly asMirror: CorePrimitivesAssertionAchainableAchainableMirror;
    readonly type: 'AmountHolding' | 'AmountToken' | 'Amount' | 'Amounts' | 'Basic' | 'BetweenPercents' | 'ClassOfYear' | 'DateInterval' | 'DatePercent' | 'Date' | 'Token' | 'Mirror';
  }

  /** @name CorePrimitivesAssertionAchainableAchainableAmountHolding (155) */
  interface CorePrimitivesAssertionAchainableAchainableAmountHolding extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly amount: Bytes;
    readonly date: Bytes;
    readonly token: Option<Bytes>;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableAmountToken (157) */
  interface CorePrimitivesAssertionAchainableAchainableAmountToken extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly amount: Bytes;
    readonly token: Option<Bytes>;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableAmount (158) */
  interface CorePrimitivesAssertionAchainableAchainableAmount extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly amount: Bytes;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableAmounts (159) */
  interface CorePrimitivesAssertionAchainableAchainableAmounts extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly amount1: Bytes;
    readonly amount2: Bytes;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableBasic (160) */
  interface CorePrimitivesAssertionAchainableAchainableBasic extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableBetweenPercents (161) */
  interface CorePrimitivesAssertionAchainableAchainableBetweenPercents extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly greaterThanOrEqualTo: Bytes;
    readonly lessThanOrEqualTo: Bytes;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableClassOfYear (162) */
  interface CorePrimitivesAssertionAchainableAchainableClassOfYear extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableDateInterval (163) */
  interface CorePrimitivesAssertionAchainableAchainableDateInterval extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly startDate: Bytes;
    readonly endDate: Bytes;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableDatePercent (164) */
  interface CorePrimitivesAssertionAchainableAchainableDatePercent extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly token: Bytes;
    readonly date: Bytes;
    readonly percent: Bytes;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableDate (165) */
  interface CorePrimitivesAssertionAchainableAchainableDate extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly date: Bytes;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableToken (166) */
  interface CorePrimitivesAssertionAchainableAchainableToken extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly token: Bytes;
  }

  /** @name CorePrimitivesAssertionAchainableAchainableMirror (167) */
  interface CorePrimitivesAssertionAchainableAchainableMirror extends Struct {
    readonly name: Bytes;
    readonly chain: Vec<CorePrimitivesAssertionNetworkWeb3Network>;
    readonly postQuantity: Option<Bytes>;
  }

  /** @name CorePrimitivesAssertionOneblockOneBlockCourseType (168) */
  interface CorePrimitivesAssertionOneblockOneBlockCourseType extends Enum {
    readonly isCourseCompletion: boolean;
    readonly isCourseOutstanding: boolean;
    readonly isCourseParticipation: boolean;
    readonly type: 'CourseCompletion' | 'CourseOutstanding' | 'CourseParticipation';
  }

  /** @name CorePrimitivesAssertionGenericDiscordRoleGenericDiscordRoleType (169) */
  interface CorePrimitivesAssertionGenericDiscordRoleGenericDiscordRoleType extends Enum {
    readonly isContest: boolean;
    readonly asContest: CorePrimitivesAssertionContestContestType;
    readonly isSoraQuiz: boolean;
    readonly asSoraQuiz: CorePrimitivesAssertionSoraquizSoraQuizType;
    readonly type: 'Contest' | 'SoraQuiz';
  }

  /** @name CorePrimitivesAssertionContestContestType (170) */
  interface CorePrimitivesAssertionContestContestType extends Enum {
    readonly isLegend: boolean;
    readonly isPopularity: boolean;
    readonly isParticipant: boolean;
    readonly type: 'Legend' | 'Popularity' | 'Participant';
  }

  /** @name CorePrimitivesAssertionSoraquizSoraQuizType (171) */
  interface CorePrimitivesAssertionSoraquizSoraQuizType extends Enum {
    readonly isAttendee: boolean;
    readonly isMaster: boolean;
    readonly type: 'Attendee' | 'Master';
  }

  /** @name CorePrimitivesAssertionBnbDomainBnbDigitDomainType (172) */
  interface CorePrimitivesAssertionBnbDomainBnbDigitDomainType extends Enum {
    readonly isBnb999ClubMember: boolean;
    readonly isBnb10kClubMember: boolean;
    readonly type: 'Bnb999ClubMember' | 'Bnb10kClubMember';
  }

  /** @name CorePrimitivesAssertionVip3Vip3MembershipCardLevel (173) */
  interface CorePrimitivesAssertionVip3Vip3MembershipCardLevel extends Enum {
    readonly isGold: boolean;
    readonly isSilver: boolean;
    readonly type: 'Gold' | 'Silver';
  }

  /** @name CorePrimitivesAssertionEvmAmountHoldingEvmTokenType (174) */
  interface CorePrimitivesAssertionEvmAmountHoldingEvmTokenType extends Enum {
    readonly isTon: boolean;
    readonly isTrx: boolean;
    readonly type: 'Ton' | 'Trx';
  }

  /** @name CorePrimitivesAssertionWeb3TokenWeb3TokenType (175) */
  interface CorePrimitivesAssertionWeb3TokenWeb3TokenType extends Enum {
    readonly isBnb: boolean;
    readonly isEth: boolean;
    readonly isSpaceId: boolean;
    readonly isLit: boolean;
    readonly isWbtc: boolean;
    readonly isUsdc: boolean;
    readonly isUsdt: boolean;
    readonly isCrv: boolean;
    readonly isMatic: boolean;
    readonly isDydx: boolean;
    readonly isAmp: boolean;
    readonly isCvx: boolean;
    readonly isTusd: boolean;
    readonly isUsdd: boolean;
    readonly isGusd: boolean;
    readonly isLink: boolean;
    readonly isGrt: boolean;
    readonly isComp: boolean;
    readonly isPeople: boolean;
    readonly isGtc: boolean;
    readonly isTon: boolean;
    readonly isTrx: boolean;
    readonly isNfp: boolean;
    readonly isSol: boolean;
    readonly isMcrt: boolean;
    readonly isBtc: boolean;
    readonly isAda: boolean;
    readonly isDoge: boolean;
    readonly isShib: boolean;
    readonly isUni: boolean;
    readonly isBch: boolean;
    readonly isEtc: boolean;
    readonly isAtom: boolean;
    readonly isDai: boolean;
    readonly isLeo: boolean;
    readonly isFil: boolean;
    readonly isImx: boolean;
    readonly isCro: boolean;
    readonly isInj: boolean;
    readonly isBean: boolean;
    readonly type: 'Bnb' | 'Eth' | 'SpaceId' | 'Lit' | 'Wbtc' | 'Usdc' | 'Usdt' | 'Crv' | 'Matic' | 'Dydx' | 'Amp' | 'Cvx' | 'Tusd' | 'Usdd' | 'Gusd' | 'Link' | 'Grt' | 'Comp' | 'People' | 'Gtc' | 'Ton' | 'Trx' | 'Nfp' | 'Sol' | 'Mcrt' | 'Btc' | 'Ada' | 'Doge' | 'Shib' | 'Uni' | 'Bch' | 'Etc' | 'Atom' | 'Dai' | 'Leo' | 'Fil' | 'Imx' | 'Cro' | 'Inj' | 'Bean';
  }

  /** @name CorePrimitivesAssertionPlatformUserPlatformUserType (176) */
  interface CorePrimitivesAssertionPlatformUserPlatformUserType extends Enum {
    readonly isKaratDaoUser: boolean;
    readonly isMagicCraftStakingUser: boolean;
    readonly type: 'KaratDaoUser' | 'MagicCraftStakingUser';
  }

  /** @name CorePrimitivesAssertionWeb3NftWeb3NftType (177) */
  interface CorePrimitivesAssertionWeb3NftWeb3NftType extends Enum {
    readonly isWeirdoGhostGang: boolean;
    readonly isClub3Sbt: boolean;
    readonly type: 'WeirdoGhostGang' | 'Club3Sbt';
  }

  /** @name CorePrimitivesAssertionDynamicDynamicParams (178) */
  interface CorePrimitivesAssertionDynamicDynamicParams extends Struct {
    readonly smartContractId: H160;
    readonly smartContractParams: Option<Bytes>;
    readonly returnLog: bool;
  }

  /** @name PalletGroupEvent (183) */
  interface PalletGroupEvent extends Enum {
    readonly isGroupMemberAdded: boolean;
    readonly asGroupMemberAdded: AccountId32;
    readonly isGroupMemberRemoved: boolean;
    readonly asGroupMemberRemoved: AccountId32;
    readonly type: 'GroupMemberAdded' | 'GroupMemberRemoved';
  }

  /** @name PalletBitacrossEvent (185) */
  interface PalletBitacrossEvent extends Enum {
    readonly isAdminSet: boolean;
    readonly asAdminSet: {
      readonly newAdmin: Option<AccountId32>;
    } & Struct;
    readonly isRelayerAdded: boolean;
    readonly asRelayerAdded: {
      readonly who: CorePrimitivesIdentity;
    } & Struct;
    readonly isRelayerRemoved: boolean;
    readonly asRelayerRemoved: {
      readonly who: CorePrimitivesIdentity;
    } & Struct;
    readonly isBtcWalletGenerated: boolean;
    readonly asBtcWalletGenerated: {
      readonly pubKey: U8aFixed;
      readonly accountId: AccountId32;
    } & Struct;
    readonly isEthWalletGenerated: boolean;
    readonly asEthWalletGenerated: {
      readonly pubKey: U8aFixed;
    } & Struct;
    readonly type: 'AdminSet' | 'RelayerAdded' | 'RelayerRemoved' | 'BtcWalletGenerated' | 'EthWalletGenerated';
  }

  /** @name PalletBitacrossMimicEvent (186) */
  interface PalletBitacrossMimicEvent extends Enum {
    readonly isBtcToEthSaved: boolean;
    readonly asBtcToEthSaved: {
      readonly btc: U8aFixed;
      readonly txIndex: u64;
    } & Struct;
    readonly isEthToBtcSaved: boolean;
    readonly asEthToBtcSaved: {
      readonly eth: H160;
      readonly txIndex: u64;
    } & Struct;
    readonly type: 'BtcToEthSaved' | 'EthToBtcSaved';
  }

  /** @name PalletEvmAssertionsEvent (187) */
  interface PalletEvmAssertionsEvent extends Enum {
    readonly isAssertionCreated: boolean;
    readonly asAssertionCreated: {
      readonly id: H160;
      readonly byteCode: Bytes;
      readonly secrets: Vec<Bytes>;
    } & Struct;
    readonly type: 'AssertionCreated';
  }

  /** @name PalletTeebagEvent (191) */
  interface PalletTeebagEvent extends Enum {
    readonly isModeSet: boolean;
    readonly asModeSet: {
      readonly newMode: PalletTeebagOperationalMode;
    } & Struct;
    readonly isAdminSet: boolean;
    readonly asAdminSet: {
      readonly newAdmin: Option<AccountId32>;
    } & Struct;
    readonly isEnclaveAdded: boolean;
    readonly asEnclaveAdded: {
      readonly who: AccountId32;
      readonly workerType: PalletTeebagWorkerType;
      readonly url: Bytes;
    } & Struct;
    readonly isEnclaveRemoved: boolean;
    readonly asEnclaveRemoved: {
      readonly who: AccountId32;
    } & Struct;
    readonly isEnclaveAuthorized: boolean;
    readonly asEnclaveAuthorized: {
      readonly workerType: PalletTeebagWorkerType;
      readonly mrenclave: U8aFixed;
    } & Struct;
    readonly isEnclaveUnauthorized: boolean;
    readonly asEnclaveUnauthorized: {
      readonly workerType: PalletTeebagWorkerType;
      readonly mrenclave: U8aFixed;
    } & Struct;
    readonly isOpaqueTaskPosted: boolean;
    readonly asOpaqueTaskPosted: {
      readonly request: PalletTeebagRsaRequest;
    } & Struct;
    readonly isParentchainBlockProcessed: boolean;
    readonly asParentchainBlockProcessed: {
      readonly who: AccountId32;
      readonly blockNumber: u32;
      readonly blockHash: H256;
      readonly taskMerkleRoot: H256;
    } & Struct;
    readonly isSidechainBlockFinalized: boolean;
    readonly asSidechainBlockFinalized: {
      readonly who: AccountId32;
      readonly sidechainBlockNumber: u64;
    } & Struct;
    readonly type: 'ModeSet' | 'AdminSet' | 'EnclaveAdded' | 'EnclaveRemoved' | 'EnclaveAuthorized' | 'EnclaveUnauthorized' | 'OpaqueTaskPosted' | 'ParentchainBlockProcessed' | 'SidechainBlockFinalized';
  }

  /** @name PalletTeebagOperationalMode (192) */
  interface PalletTeebagOperationalMode extends Enum {
    readonly isProduction: boolean;
    readonly isDevelopment: boolean;
    readonly isMaintenance: boolean;
    readonly type: 'Production' | 'Development' | 'Maintenance';
  }

  /** @name PalletTeebagWorkerType (193) */
  interface PalletTeebagWorkerType extends Enum {
    readonly isIdentity: boolean;
    readonly isBitAcross: boolean;
    readonly type: 'Identity' | 'BitAcross';
  }

  /** @name PalletTeebagRsaRequest (194) */
  interface PalletTeebagRsaRequest extends Struct {
    readonly shard: H256;
    readonly payload: Bytes;
  }

  /** @name PalletEvmEvent (195) */
  interface PalletEvmEvent extends Enum {
    readonly isLog: boolean;
    readonly asLog: {
      readonly log: EthereumLog;
    } & Struct;
    readonly isCreated: boolean;
    readonly asCreated: {
      readonly address: H160;
    } & Struct;
    readonly isCreatedFailed: boolean;
    readonly asCreatedFailed: {
      readonly address: H160;
    } & Struct;
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly address: H160;
    } & Struct;
    readonly isExecutedFailed: boolean;
    readonly asExecutedFailed: {
      readonly address: H160;
    } & Struct;
    readonly type: 'Log' | 'Created' | 'CreatedFailed' | 'Executed' | 'ExecutedFailed';
  }

  /** @name EthereumLog (196) */
  interface EthereumLog extends Struct {
    readonly address: H160;
    readonly topics: Vec<H256>;
    readonly data: Bytes;
  }

  /** @name PalletEthereumEvent (198) */
  interface PalletEthereumEvent extends Enum {
    readonly isExecuted: boolean;
    readonly asExecuted: {
      readonly from: H160;
      readonly to: H160;
      readonly transactionHash: H256;
      readonly exitReason: EvmCoreErrorExitReason;
      readonly extraData: Bytes;
    } & Struct;
    readonly type: 'Executed';
  }

  /** @name EvmCoreErrorExitReason (199) */
  interface EvmCoreErrorExitReason extends Enum {
    readonly isSucceed: boolean;
    readonly asSucceed: EvmCoreErrorExitSucceed;
    readonly isError: boolean;
    readonly asError: EvmCoreErrorExitError;
    readonly isRevert: boolean;
    readonly asRevert: EvmCoreErrorExitRevert;
    readonly isFatal: boolean;
    readonly asFatal: EvmCoreErrorExitFatal;
    readonly type: 'Succeed' | 'Error' | 'Revert' | 'Fatal';
  }

  /** @name EvmCoreErrorExitSucceed (200) */
  interface EvmCoreErrorExitSucceed extends Enum {
    readonly isStopped: boolean;
    readonly isReturned: boolean;
    readonly isSuicided: boolean;
    readonly type: 'Stopped' | 'Returned' | 'Suicided';
  }

  /** @name EvmCoreErrorExitError (201) */
  interface EvmCoreErrorExitError extends Enum {
    readonly isStackUnderflow: boolean;
    readonly isStackOverflow: boolean;
    readonly isInvalidJump: boolean;
    readonly isInvalidRange: boolean;
    readonly isDesignatedInvalid: boolean;
    readonly isCallTooDeep: boolean;
    readonly isCreateCollision: boolean;
    readonly isCreateContractLimit: boolean;
    readonly isOutOfOffset: boolean;
    readonly isOutOfGas: boolean;
    readonly isOutOfFund: boolean;
    readonly isPcUnderflow: boolean;
    readonly isCreateEmpty: boolean;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly isMaxNonce: boolean;
    readonly isInvalidCode: boolean;
    readonly asInvalidCode: u8;
    readonly type: 'StackUnderflow' | 'StackOverflow' | 'InvalidJump' | 'InvalidRange' | 'DesignatedInvalid' | 'CallTooDeep' | 'CreateCollision' | 'CreateContractLimit' | 'OutOfOffset' | 'OutOfGas' | 'OutOfFund' | 'PcUnderflow' | 'CreateEmpty' | 'Other' | 'MaxNonce' | 'InvalidCode';
  }

  /** @name EvmCoreErrorExitRevert (205) */
  interface EvmCoreErrorExitRevert extends Enum {
    readonly isReverted: boolean;
    readonly type: 'Reverted';
  }

  /** @name EvmCoreErrorExitFatal (206) */
  interface EvmCoreErrorExitFatal extends Enum {
    readonly isNotSupported: boolean;
    readonly isUnhandledInterrupt: boolean;
    readonly isCallErrorAsFatal: boolean;
    readonly asCallErrorAsFatal: EvmCoreErrorExitError;
    readonly isOther: boolean;
    readonly asOther: Text;
    readonly type: 'NotSupported' | 'UnhandledInterrupt' | 'CallErrorAsFatal' | 'Other';
  }

  /** @name PalletSudoEvent (207) */
  interface PalletSudoEvent extends Enum {
    readonly isSudid: boolean;
    readonly asSudid: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly isKeyChanged: boolean;
    readonly asKeyChanged: {
      readonly oldSudoer: Option<AccountId32>;
    } & Struct;
    readonly isSudoAsDone: boolean;
    readonly asSudoAsDone: {
      readonly sudoResult: Result<Null, SpRuntimeDispatchError>;
    } & Struct;
    readonly type: 'Sudid' | 'KeyChanged' | 'SudoAsDone';
  }

  /** @name FrameSystemPhase (208) */
  interface FrameSystemPhase extends Enum {
    readonly isApplyExtrinsic: boolean;
    readonly asApplyExtrinsic: u32;
    readonly isFinalization: boolean;
    readonly isInitialization: boolean;
    readonly type: 'ApplyExtrinsic' | 'Finalization' | 'Initialization';
  }

  /** @name FrameSystemLastRuntimeUpgradeInfo (210) */
  interface FrameSystemLastRuntimeUpgradeInfo extends Struct {
    readonly specVersion: Compact<u32>;
    readonly specName: Text;
  }

  /** @name FrameSystemCall (211) */
  interface FrameSystemCall extends Enum {
    readonly isRemark: boolean;
    readonly asRemark: {
      readonly remark: Bytes;
    } & Struct;
    readonly isSetHeapPages: boolean;
    readonly asSetHeapPages: {
      readonly pages: u64;
    } & Struct;
    readonly isSetCode: boolean;
    readonly asSetCode: {
      readonly code: Bytes;
    } & Struct;
    readonly isSetCodeWithoutChecks: boolean;
    readonly asSetCodeWithoutChecks: {
      readonly code: Bytes;
    } & Struct;
    readonly isSetStorage: boolean;
    readonly asSetStorage: {
      readonly items: Vec<ITuple<[Bytes, Bytes]>>;
    } & Struct;
    readonly isKillStorage: boolean;
    readonly asKillStorage: {
      readonly keys_: Vec<Bytes>;
    } & Struct;
    readonly isKillPrefix: boolean;
    readonly asKillPrefix: {
      readonly prefix: Bytes;
      readonly subkeys: u32;
    } & Struct;
    readonly isRemarkWithEvent: boolean;
    readonly asRemarkWithEvent: {
      readonly remark: Bytes;
    } & Struct;
    readonly type: 'Remark' | 'SetHeapPages' | 'SetCode' | 'SetCodeWithoutChecks' | 'SetStorage' | 'KillStorage' | 'KillPrefix' | 'RemarkWithEvent';
  }

  /** @name FrameSystemLimitsBlockWeights (214) */
  interface FrameSystemLimitsBlockWeights extends Struct {
    readonly baseBlock: SpWeightsWeightV2Weight;
    readonly maxBlock: SpWeightsWeightV2Weight;
    readonly perClass: FrameSupportDispatchPerDispatchClassWeightsPerClass;
  }

  /** @name FrameSupportDispatchPerDispatchClassWeightsPerClass (215) */
  interface FrameSupportDispatchPerDispatchClassWeightsPerClass extends Struct {
    readonly normal: FrameSystemLimitsWeightsPerClass;
    readonly operational: FrameSystemLimitsWeightsPerClass;
    readonly mandatory: FrameSystemLimitsWeightsPerClass;
  }

  /** @name FrameSystemLimitsWeightsPerClass (216) */
  interface FrameSystemLimitsWeightsPerClass extends Struct {
    readonly baseExtrinsic: SpWeightsWeightV2Weight;
    readonly maxExtrinsic: Option<SpWeightsWeightV2Weight>;
    readonly maxTotal: Option<SpWeightsWeightV2Weight>;
    readonly reserved: Option<SpWeightsWeightV2Weight>;
  }

  /** @name FrameSystemLimitsBlockLength (218) */
  interface FrameSystemLimitsBlockLength extends Struct {
    readonly max: FrameSupportDispatchPerDispatchClassU32;
  }

  /** @name FrameSupportDispatchPerDispatchClassU32 (219) */
  interface FrameSupportDispatchPerDispatchClassU32 extends Struct {
    readonly normal: u32;
    readonly operational: u32;
    readonly mandatory: u32;
  }

  /** @name SpWeightsRuntimeDbWeight (220) */
  interface SpWeightsRuntimeDbWeight extends Struct {
    readonly read: u64;
    readonly write: u64;
  }

  /** @name SpVersionRuntimeVersion (221) */
  interface SpVersionRuntimeVersion extends Struct {
    readonly specName: Text;
    readonly implName: Text;
    readonly authoringVersion: u32;
    readonly specVersion: u32;
    readonly implVersion: u32;
    readonly apis: Vec<ITuple<[U8aFixed, u32]>>;
    readonly transactionVersion: u32;
    readonly stateVersion: u8;
  }

  /** @name FrameSystemError (225) */
  interface FrameSystemError extends Enum {
    readonly isInvalidSpecName: boolean;
    readonly isSpecVersionNeedsToIncrease: boolean;
    readonly isFailedToExtractRuntimeVersion: boolean;
    readonly isNonDefaultComposite: boolean;
    readonly isNonZeroRefCount: boolean;
    readonly isCallFiltered: boolean;
    readonly type: 'InvalidSpecName' | 'SpecVersionNeedsToIncrease' | 'FailedToExtractRuntimeVersion' | 'NonDefaultComposite' | 'NonZeroRefCount' | 'CallFiltered';
  }

  /** @name PalletTimestampCall (226) */
  interface PalletTimestampCall extends Enum {
    readonly isSet: boolean;
    readonly asSet: {
      readonly now: Compact<u64>;
    } & Struct;
    readonly type: 'Set';
  }

  /** @name PalletSchedulerScheduled (229) */
  interface PalletSchedulerScheduled extends Struct {
    readonly maybeId: Option<U8aFixed>;
    readonly priority: u8;
    readonly call: FrameSupportPreimagesBounded;
    readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
    readonly origin: RococoParachainRuntimeOriginCaller;
  }

  /** @name FrameSupportPreimagesBounded (230) */
  interface FrameSupportPreimagesBounded extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: {
      readonly hash_: H256;
    } & Struct;
    readonly isInline: boolean;
    readonly asInline: Bytes;
    readonly isLookup: boolean;
    readonly asLookup: {
      readonly hash_: H256;
      readonly len: u32;
    } & Struct;
    readonly type: 'Legacy' | 'Inline' | 'Lookup';
  }

  /** @name PalletSchedulerCall (232) */
  interface PalletSchedulerCall extends Enum {
    readonly isSchedule: boolean;
    readonly asSchedule: {
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancel: boolean;
    readonly asCancel: {
      readonly when: u32;
      readonly index: u32;
    } & Struct;
    readonly isScheduleNamed: boolean;
    readonly asScheduleNamed: {
      readonly id: U8aFixed;
      readonly when: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isCancelNamed: boolean;
    readonly asCancelNamed: {
      readonly id: U8aFixed;
    } & Struct;
    readonly isScheduleAfter: boolean;
    readonly asScheduleAfter: {
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly isScheduleNamedAfter: boolean;
    readonly asScheduleNamedAfter: {
      readonly id: U8aFixed;
      readonly after: u32;
      readonly maybePeriodic: Option<ITuple<[u32, u32]>>;
      readonly priority: u8;
      readonly call: Call;
    } & Struct;
    readonly type: 'Schedule' | 'Cancel' | 'ScheduleNamed' | 'CancelNamed' | 'ScheduleAfter' | 'ScheduleNamedAfter';
  }

  /** @name PalletUtilityCall (234) */
  interface PalletUtilityCall extends Enum {
    readonly isBatch: boolean;
    readonly asBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isAsDerivative: boolean;
    readonly asAsDerivative: {
      readonly index: u16;
      readonly call: Call;
    } & Struct;
    readonly isBatchAll: boolean;
    readonly asBatchAll: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isDispatchAs: boolean;
    readonly asDispatchAs: {
      readonly asOrigin: RococoParachainRuntimeOriginCaller;
      readonly call: Call;
    } & Struct;
    readonly isForceBatch: boolean;
    readonly asForceBatch: {
      readonly calls: Vec<Call>;
    } & Struct;
    readonly isWithWeight: boolean;
    readonly asWithWeight: {
      readonly call: Call;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type: 'Batch' | 'AsDerivative' | 'BatchAll' | 'DispatchAs' | 'ForceBatch' | 'WithWeight';
  }

  /** @name RococoParachainRuntimeOriginCaller (236) */
  interface RococoParachainRuntimeOriginCaller extends Enum {
    readonly isSystem: boolean;
    readonly asSystem: FrameSupportDispatchRawOrigin;
    readonly isVoid: boolean;
    readonly isCouncil: boolean;
    readonly asCouncil: PalletCollectiveRawOrigin;
    readonly isTechnicalCommittee: boolean;
    readonly asTechnicalCommittee: PalletCollectiveRawOrigin;
    readonly isPolkadotXcm: boolean;
    readonly asPolkadotXcm: PalletXcmOrigin;
    readonly isCumulusXcm: boolean;
    readonly asCumulusXcm: CumulusPalletXcmOrigin;
    readonly isDeveloperCommittee: boolean;
    readonly asDeveloperCommittee: PalletCollectiveRawOrigin;
    readonly isEthereum: boolean;
    readonly asEthereum: PalletEthereumRawOrigin;
    readonly type: 'System' | 'Void' | 'Council' | 'TechnicalCommittee' | 'PolkadotXcm' | 'CumulusXcm' | 'DeveloperCommittee' | 'Ethereum';
  }

  /** @name FrameSupportDispatchRawOrigin (237) */
  interface FrameSupportDispatchRawOrigin extends Enum {
    readonly isRoot: boolean;
    readonly isSigned: boolean;
    readonly asSigned: AccountId32;
    readonly isNone: boolean;
    readonly type: 'Root' | 'Signed' | 'None';
  }

  /** @name PalletCollectiveRawOrigin (238) */
  interface PalletCollectiveRawOrigin extends Enum {
    readonly isMembers: boolean;
    readonly asMembers: ITuple<[u32, u32]>;
    readonly isMember: boolean;
    readonly asMember: AccountId32;
    readonly isPhantom: boolean;
    readonly type: 'Members' | 'Member' | 'Phantom';
  }

  /** @name PalletXcmOrigin (240) */
  interface PalletXcmOrigin extends Enum {
    readonly isXcm: boolean;
    readonly asXcm: XcmV3MultiLocation;
    readonly isResponse: boolean;
    readonly asResponse: XcmV3MultiLocation;
    readonly type: 'Xcm' | 'Response';
  }

  /** @name CumulusPalletXcmOrigin (241) */
  interface CumulusPalletXcmOrigin extends Enum {
    readonly isRelay: boolean;
    readonly isSiblingParachain: boolean;
    readonly asSiblingParachain: u32;
    readonly type: 'Relay' | 'SiblingParachain';
  }

  /** @name PalletEthereumRawOrigin (243) */
  interface PalletEthereumRawOrigin extends Enum {
    readonly isEthereumTransaction: boolean;
    readonly asEthereumTransaction: H160;
    readonly type: 'EthereumTransaction';
  }

  /** @name SpCoreVoid (244) */
  type SpCoreVoid = Null;

  /** @name PalletMultisigCall (245) */
  interface PalletMultisigCall extends Enum {
    readonly isAsMultiThreshold1: boolean;
    readonly asAsMultiThreshold1: {
      readonly otherSignatories: Vec<AccountId32>;
      readonly call: Call;
    } & Struct;
    readonly isAsMulti: boolean;
    readonly asAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId32>;
      readonly maybeTimepoint: Option<PalletMultisigTimepoint>;
      readonly call: Call;
      readonly maxWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isApproveAsMulti: boolean;
    readonly asApproveAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId32>;
      readonly maybeTimepoint: Option<PalletMultisigTimepoint>;
      readonly callHash: U8aFixed;
      readonly maxWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isCancelAsMulti: boolean;
    readonly asCancelAsMulti: {
      readonly threshold: u16;
      readonly otherSignatories: Vec<AccountId32>;
      readonly timepoint: PalletMultisigTimepoint;
      readonly callHash: U8aFixed;
    } & Struct;
    readonly type: 'AsMultiThreshold1' | 'AsMulti' | 'ApproveAsMulti' | 'CancelAsMulti';
  }

  /** @name PalletProxyCall (248) */
  interface PalletProxyCall extends Enum {
    readonly isProxy: boolean;
    readonly asProxy: {
      readonly real: MultiAddress;
      readonly forceProxyType: Option<RococoParachainRuntimeProxyType>;
      readonly call: Call;
    } & Struct;
    readonly isAddProxy: boolean;
    readonly asAddProxy: {
      readonly delegate: MultiAddress;
      readonly proxyType: RococoParachainRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly isRemoveProxy: boolean;
    readonly asRemoveProxy: {
      readonly delegate: MultiAddress;
      readonly proxyType: RococoParachainRuntimeProxyType;
      readonly delay: u32;
    } & Struct;
    readonly isRemoveProxies: boolean;
    readonly isCreatePure: boolean;
    readonly asCreatePure: {
      readonly proxyType: RococoParachainRuntimeProxyType;
      readonly delay: u32;
      readonly index: u16;
    } & Struct;
    readonly isKillPure: boolean;
    readonly asKillPure: {
      readonly spawner: MultiAddress;
      readonly proxyType: RococoParachainRuntimeProxyType;
      readonly index: u16;
      readonly height: Compact<u32>;
      readonly extIndex: Compact<u32>;
    } & Struct;
    readonly isAnnounce: boolean;
    readonly asAnnounce: {
      readonly real: MultiAddress;
      readonly callHash: H256;
    } & Struct;
    readonly isRemoveAnnouncement: boolean;
    readonly asRemoveAnnouncement: {
      readonly real: MultiAddress;
      readonly callHash: H256;
    } & Struct;
    readonly isRejectAnnouncement: boolean;
    readonly asRejectAnnouncement: {
      readonly delegate: MultiAddress;
      readonly callHash: H256;
    } & Struct;
    readonly isProxyAnnounced: boolean;
    readonly asProxyAnnounced: {
      readonly delegate: MultiAddress;
      readonly real: MultiAddress;
      readonly forceProxyType: Option<RococoParachainRuntimeProxyType>;
      readonly call: Call;
    } & Struct;
    readonly type: 'Proxy' | 'AddProxy' | 'RemoveProxy' | 'RemoveProxies' | 'CreatePure' | 'KillPure' | 'Announce' | 'RemoveAnnouncement' | 'RejectAnnouncement' | 'ProxyAnnounced';
  }

  /** @name PalletPreimageCall (252) */
  interface PalletPreimageCall extends Enum {
    readonly isNotePreimage: boolean;
    readonly asNotePreimage: {
      readonly bytes: Bytes;
    } & Struct;
    readonly isUnnotePreimage: boolean;
    readonly asUnnotePreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isRequestPreimage: boolean;
    readonly asRequestPreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly isUnrequestPreimage: boolean;
    readonly asUnrequestPreimage: {
      readonly hash_: H256;
    } & Struct;
    readonly type: 'NotePreimage' | 'UnnotePreimage' | 'RequestPreimage' | 'UnrequestPreimage';
  }

  /** @name PalletBalancesCall (253) */
  interface PalletBalancesCall extends Enum {
    readonly isTransferAllowDeath: boolean;
    readonly asTransferAllowDeath: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isSetBalanceDeprecated: boolean;
    readonly asSetBalanceDeprecated: {
      readonly who: MultiAddress;
      readonly newFree: Compact<u128>;
      readonly oldReserved: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly source: MultiAddress;
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferKeepAlive: boolean;
    readonly asTransferKeepAlive: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isTransferAll: boolean;
    readonly asTransferAll: {
      readonly dest: MultiAddress;
      readonly keepAlive: bool;
    } & Struct;
    readonly isForceUnreserve: boolean;
    readonly asForceUnreserve: {
      readonly who: MultiAddress;
      readonly amount: u128;
    } & Struct;
    readonly isUpgradeAccounts: boolean;
    readonly asUpgradeAccounts: {
      readonly who: Vec<AccountId32>;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly dest: MultiAddress;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isForceSetBalance: boolean;
    readonly asForceSetBalance: {
      readonly who: MultiAddress;
      readonly newFree: Compact<u128>;
    } & Struct;
    readonly type: 'TransferAllowDeath' | 'SetBalanceDeprecated' | 'ForceTransfer' | 'TransferKeepAlive' | 'TransferAll' | 'ForceUnreserve' | 'UpgradeAccounts' | 'Transfer' | 'ForceSetBalance';
  }

  /** @name PalletVestingCall (254) */
  interface PalletVestingCall extends Enum {
    readonly isVest: boolean;
    readonly isVestOther: boolean;
    readonly asVestOther: {
      readonly target: MultiAddress;
    } & Struct;
    readonly isVestedTransfer: boolean;
    readonly asVestedTransfer: {
      readonly target: MultiAddress;
      readonly schedule: PalletVestingVestingInfo;
    } & Struct;
    readonly isForceVestedTransfer: boolean;
    readonly asForceVestedTransfer: {
      readonly source: MultiAddress;
      readonly target: MultiAddress;
      readonly schedule: PalletVestingVestingInfo;
    } & Struct;
    readonly isMergeSchedules: boolean;
    readonly asMergeSchedules: {
      readonly schedule1Index: u32;
      readonly schedule2Index: u32;
    } & Struct;
    readonly type: 'Vest' | 'VestOther' | 'VestedTransfer' | 'ForceVestedTransfer' | 'MergeSchedules';
  }

  /** @name PalletVestingVestingInfo (255) */
  interface PalletVestingVestingInfo extends Struct {
    readonly locked: u128;
    readonly perBlock: u128;
    readonly startingBlock: u32;
  }

  /** @name PalletTreasuryCall (256) */
  interface PalletTreasuryCall extends Enum {
    readonly isProposeSpend: boolean;
    readonly asProposeSpend: {
      readonly value: Compact<u128>;
      readonly beneficiary: MultiAddress;
    } & Struct;
    readonly isRejectProposal: boolean;
    readonly asRejectProposal: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly isApproveProposal: boolean;
    readonly asApproveProposal: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly isSpend: boolean;
    readonly asSpend: {
      readonly amount: Compact<u128>;
      readonly beneficiary: MultiAddress;
    } & Struct;
    readonly isRemoveApproval: boolean;
    readonly asRemoveApproval: {
      readonly proposalId: Compact<u32>;
    } & Struct;
    readonly type: 'ProposeSpend' | 'RejectProposal' | 'ApproveProposal' | 'Spend' | 'RemoveApproval';
  }

  /** @name PalletDemocracyCall (257) */
  interface PalletDemocracyCall extends Enum {
    readonly isPropose: boolean;
    readonly asPropose: {
      readonly proposal: FrameSupportPreimagesBounded;
      readonly value: Compact<u128>;
    } & Struct;
    readonly isSecond: boolean;
    readonly asSecond: {
      readonly proposal: Compact<u32>;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly refIndex: Compact<u32>;
      readonly vote: PalletDemocracyVoteAccountVote;
    } & Struct;
    readonly isEmergencyCancel: boolean;
    readonly asEmergencyCancel: {
      readonly refIndex: u32;
    } & Struct;
    readonly isExternalPropose: boolean;
    readonly asExternalPropose: {
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isExternalProposeMajority: boolean;
    readonly asExternalProposeMajority: {
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isExternalProposeDefault: boolean;
    readonly asExternalProposeDefault: {
      readonly proposal: FrameSupportPreimagesBounded;
    } & Struct;
    readonly isFastTrack: boolean;
    readonly asFastTrack: {
      readonly proposalHash: H256;
      readonly votingPeriod: u32;
      readonly delay: u32;
    } & Struct;
    readonly isVetoExternal: boolean;
    readonly asVetoExternal: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isCancelReferendum: boolean;
    readonly asCancelReferendum: {
      readonly refIndex: Compact<u32>;
    } & Struct;
    readonly isDelegate: boolean;
    readonly asDelegate: {
      readonly to: MultiAddress;
      readonly conviction: PalletDemocracyConviction;
      readonly balance: u128;
    } & Struct;
    readonly isUndelegate: boolean;
    readonly isClearPublicProposals: boolean;
    readonly isUnlock: boolean;
    readonly asUnlock: {
      readonly target: MultiAddress;
    } & Struct;
    readonly isRemoveVote: boolean;
    readonly asRemoveVote: {
      readonly index: u32;
    } & Struct;
    readonly isRemoveOtherVote: boolean;
    readonly asRemoveOtherVote: {
      readonly target: MultiAddress;
      readonly index: u32;
    } & Struct;
    readonly isBlacklist: boolean;
    readonly asBlacklist: {
      readonly proposalHash: H256;
      readonly maybeRefIndex: Option<u32>;
    } & Struct;
    readonly isCancelProposal: boolean;
    readonly asCancelProposal: {
      readonly propIndex: Compact<u32>;
    } & Struct;
    readonly isSetMetadata: boolean;
    readonly asSetMetadata: {
      readonly owner: PalletDemocracyMetadataOwner;
      readonly maybeHash: Option<H256>;
    } & Struct;
    readonly type: 'Propose' | 'Second' | 'Vote' | 'EmergencyCancel' | 'ExternalPropose' | 'ExternalProposeMajority' | 'ExternalProposeDefault' | 'FastTrack' | 'VetoExternal' | 'CancelReferendum' | 'Delegate' | 'Undelegate' | 'ClearPublicProposals' | 'Unlock' | 'RemoveVote' | 'RemoveOtherVote' | 'Blacklist' | 'CancelProposal' | 'SetMetadata';
  }

  /** @name PalletDemocracyConviction (258) */
  interface PalletDemocracyConviction extends Enum {
    readonly isNone: boolean;
    readonly isLocked1x: boolean;
    readonly isLocked2x: boolean;
    readonly isLocked3x: boolean;
    readonly isLocked4x: boolean;
    readonly isLocked5x: boolean;
    readonly isLocked6x: boolean;
    readonly type: 'None' | 'Locked1x' | 'Locked2x' | 'Locked3x' | 'Locked4x' | 'Locked5x' | 'Locked6x';
  }

  /** @name PalletCollectiveCall (261) */
  interface PalletCollectiveCall extends Enum {
    readonly isSetMembers: boolean;
    readonly asSetMembers: {
      readonly newMembers: Vec<AccountId32>;
      readonly prime: Option<AccountId32>;
      readonly oldCount: u32;
    } & Struct;
    readonly isExecute: boolean;
    readonly asExecute: {
      readonly proposal: Call;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isPropose: boolean;
    readonly asPropose: {
      readonly threshold: Compact<u32>;
      readonly proposal: Call;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly isVote: boolean;
    readonly asVote: {
      readonly proposal: H256;
      readonly index: Compact<u32>;
      readonly approve: bool;
    } & Struct;
    readonly isDisapproveProposal: boolean;
    readonly asDisapproveProposal: {
      readonly proposalHash: H256;
    } & Struct;
    readonly isClose: boolean;
    readonly asClose: {
      readonly proposalHash: H256;
      readonly index: Compact<u32>;
      readonly proposalWeightBound: SpWeightsWeightV2Weight;
      readonly lengthBound: Compact<u32>;
    } & Struct;
    readonly type: 'SetMembers' | 'Execute' | 'Propose' | 'Vote' | 'DisapproveProposal' | 'Close';
  }

  /** @name PalletMembershipCall (262) */
  interface PalletMembershipCall extends Enum {
    readonly isAddMember: boolean;
    readonly asAddMember: {
      readonly who: MultiAddress;
    } & Struct;
    readonly isRemoveMember: boolean;
    readonly asRemoveMember: {
      readonly who: MultiAddress;
    } & Struct;
    readonly isSwapMember: boolean;
    readonly asSwapMember: {
      readonly remove: MultiAddress;
      readonly add: MultiAddress;
    } & Struct;
    readonly isResetMembers: boolean;
    readonly asResetMembers: {
      readonly members: Vec<AccountId32>;
    } & Struct;
    readonly isChangeKey: boolean;
    readonly asChangeKey: {
      readonly new_: MultiAddress;
    } & Struct;
    readonly isSetPrime: boolean;
    readonly asSetPrime: {
      readonly who: MultiAddress;
    } & Struct;
    readonly isClearPrime: boolean;
    readonly type: 'AddMember' | 'RemoveMember' | 'SwapMember' | 'ResetMembers' | 'ChangeKey' | 'SetPrime' | 'ClearPrime';
  }

  /** @name PalletBountiesCall (265) */
  interface PalletBountiesCall extends Enum {
    readonly isProposeBounty: boolean;
    readonly asProposeBounty: {
      readonly value: Compact<u128>;
      readonly description: Bytes;
    } & Struct;
    readonly isApproveBounty: boolean;
    readonly asApproveBounty: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isProposeCurator: boolean;
    readonly asProposeCurator: {
      readonly bountyId: Compact<u32>;
      readonly curator: MultiAddress;
      readonly fee: Compact<u128>;
    } & Struct;
    readonly isUnassignCurator: boolean;
    readonly asUnassignCurator: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isAcceptCurator: boolean;
    readonly asAcceptCurator: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isAwardBounty: boolean;
    readonly asAwardBounty: {
      readonly bountyId: Compact<u32>;
      readonly beneficiary: MultiAddress;
    } & Struct;
    readonly isClaimBounty: boolean;
    readonly asClaimBounty: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isCloseBounty: boolean;
    readonly asCloseBounty: {
      readonly bountyId: Compact<u32>;
    } & Struct;
    readonly isExtendBountyExpiry: boolean;
    readonly asExtendBountyExpiry: {
      readonly bountyId: Compact<u32>;
      readonly remark: Bytes;
    } & Struct;
    readonly type: 'ProposeBounty' | 'ApproveBounty' | 'ProposeCurator' | 'UnassignCurator' | 'AcceptCurator' | 'AwardBounty' | 'ClaimBounty' | 'CloseBounty' | 'ExtendBountyExpiry';
  }

  /** @name PalletTipsCall (266) */
  interface PalletTipsCall extends Enum {
    readonly isReportAwesome: boolean;
    readonly asReportAwesome: {
      readonly reason: Bytes;
      readonly who: MultiAddress;
    } & Struct;
    readonly isRetractTip: boolean;
    readonly asRetractTip: {
      readonly hash_: H256;
    } & Struct;
    readonly isTipNew: boolean;
    readonly asTipNew: {
      readonly reason: Bytes;
      readonly who: MultiAddress;
      readonly tipValue: Compact<u128>;
    } & Struct;
    readonly isTip: boolean;
    readonly asTip: {
      readonly hash_: H256;
      readonly tipValue: Compact<u128>;
    } & Struct;
    readonly isCloseTip: boolean;
    readonly asCloseTip: {
      readonly hash_: H256;
    } & Struct;
    readonly isSlashTip: boolean;
    readonly asSlashTip: {
      readonly hash_: H256;
    } & Struct;
    readonly type: 'ReportAwesome' | 'RetractTip' | 'TipNew' | 'Tip' | 'CloseTip' | 'SlashTip';
  }

  /** @name PalletIdentityCall (267) */
  interface PalletIdentityCall extends Enum {
    readonly isAddRegistrar: boolean;
    readonly asAddRegistrar: {
      readonly account: MultiAddress;
    } & Struct;
    readonly isSetIdentity: boolean;
    readonly asSetIdentity: {
      readonly info: PalletIdentityIdentityInfo;
    } & Struct;
    readonly isSetSubs: boolean;
    readonly asSetSubs: {
      readonly subs: Vec<ITuple<[AccountId32, Data]>>;
    } & Struct;
    readonly isClearIdentity: boolean;
    readonly isRequestJudgement: boolean;
    readonly asRequestJudgement: {
      readonly regIndex: Compact<u32>;
      readonly maxFee: Compact<u128>;
    } & Struct;
    readonly isCancelRequest: boolean;
    readonly asCancelRequest: {
      readonly regIndex: u32;
    } & Struct;
    readonly isSetFee: boolean;
    readonly asSetFee: {
      readonly index: Compact<u32>;
      readonly fee: Compact<u128>;
    } & Struct;
    readonly isSetAccountId: boolean;
    readonly asSetAccountId: {
      readonly index: Compact<u32>;
      readonly new_: MultiAddress;
    } & Struct;
    readonly isSetFields: boolean;
    readonly asSetFields: {
      readonly index: Compact<u32>;
      readonly fields: PalletIdentityBitFlags;
    } & Struct;
    readonly isProvideJudgement: boolean;
    readonly asProvideJudgement: {
      readonly regIndex: Compact<u32>;
      readonly target: MultiAddress;
      readonly judgement: PalletIdentityJudgement;
      readonly identity: H256;
    } & Struct;
    readonly isKillIdentity: boolean;
    readonly asKillIdentity: {
      readonly target: MultiAddress;
    } & Struct;
    readonly isAddSub: boolean;
    readonly asAddSub: {
      readonly sub: MultiAddress;
      readonly data: Data;
    } & Struct;
    readonly isRenameSub: boolean;
    readonly asRenameSub: {
      readonly sub: MultiAddress;
      readonly data: Data;
    } & Struct;
    readonly isRemoveSub: boolean;
    readonly asRemoveSub: {
      readonly sub: MultiAddress;
    } & Struct;
    readonly isQuitSub: boolean;
    readonly type: 'AddRegistrar' | 'SetIdentity' | 'SetSubs' | 'ClearIdentity' | 'RequestJudgement' | 'CancelRequest' | 'SetFee' | 'SetAccountId' | 'SetFields' | 'ProvideJudgement' | 'KillIdentity' | 'AddSub' | 'RenameSub' | 'RemoveSub' | 'QuitSub';
  }

  /** @name PalletIdentityIdentityInfo (268) */
  interface PalletIdentityIdentityInfo extends Struct {
    readonly additional: Vec<ITuple<[Data, Data]>>;
    readonly display: Data;
    readonly legal: Data;
    readonly web: Data;
    readonly riot: Data;
    readonly email: Data;
    readonly pgpFingerprint: Option<U8aFixed>;
    readonly image: Data;
    readonly twitter: Data;
  }

  /** @name PalletIdentityBitFlags (304) */
  interface PalletIdentityBitFlags extends Set {
    readonly isDisplay: boolean;
    readonly isLegal: boolean;
    readonly isWeb: boolean;
    readonly isRiot: boolean;
    readonly isEmail: boolean;
    readonly isPgpFingerprint: boolean;
    readonly isImage: boolean;
    readonly isTwitter: boolean;
  }

  /** @name PalletIdentityIdentityField (305) */
  interface PalletIdentityIdentityField extends Enum {
    readonly isDisplay: boolean;
    readonly isLegal: boolean;
    readonly isWeb: boolean;
    readonly isRiot: boolean;
    readonly isEmail: boolean;
    readonly isPgpFingerprint: boolean;
    readonly isImage: boolean;
    readonly isTwitter: boolean;
    readonly type: 'Display' | 'Legal' | 'Web' | 'Riot' | 'Email' | 'PgpFingerprint' | 'Image' | 'Twitter';
  }

  /** @name PalletIdentityJudgement (306) */
  interface PalletIdentityJudgement extends Enum {
    readonly isUnknown: boolean;
    readonly isFeePaid: boolean;
    readonly asFeePaid: u128;
    readonly isReasonable: boolean;
    readonly isKnownGood: boolean;
    readonly isOutOfDate: boolean;
    readonly isLowQuality: boolean;
    readonly isErroneous: boolean;
    readonly type: 'Unknown' | 'FeePaid' | 'Reasonable' | 'KnownGood' | 'OutOfDate' | 'LowQuality' | 'Erroneous';
  }

  /** @name CumulusPalletParachainSystemCall (307) */
  interface CumulusPalletParachainSystemCall extends Enum {
    readonly isSetValidationData: boolean;
    readonly asSetValidationData: {
      readonly data: CumulusPrimitivesParachainInherentParachainInherentData;
    } & Struct;
    readonly isSudoSendUpwardMessage: boolean;
    readonly asSudoSendUpwardMessage: {
      readonly message: Bytes;
    } & Struct;
    readonly isAuthorizeUpgrade: boolean;
    readonly asAuthorizeUpgrade: {
      readonly codeHash: H256;
      readonly checkVersion: bool;
    } & Struct;
    readonly isEnactAuthorizedUpgrade: boolean;
    readonly asEnactAuthorizedUpgrade: {
      readonly code: Bytes;
    } & Struct;
    readonly type: 'SetValidationData' | 'SudoSendUpwardMessage' | 'AuthorizeUpgrade' | 'EnactAuthorizedUpgrade';
  }

  /** @name CumulusPrimitivesParachainInherentParachainInherentData (308) */
  interface CumulusPrimitivesParachainInherentParachainInherentData extends Struct {
    readonly validationData: PolkadotPrimitivesV4PersistedValidationData;
    readonly relayChainState: SpTrieStorageProof;
    readonly downwardMessages: Vec<PolkadotCorePrimitivesInboundDownwardMessage>;
    readonly horizontalMessages: BTreeMap<u32, Vec<PolkadotCorePrimitivesInboundHrmpMessage>>;
  }

  /** @name PolkadotPrimitivesV4PersistedValidationData (309) */
  interface PolkadotPrimitivesV4PersistedValidationData extends Struct {
    readonly parentHead: Bytes;
    readonly relayParentNumber: u32;
    readonly relayParentStorageRoot: H256;
    readonly maxPovSize: u32;
  }

  /** @name SpTrieStorageProof (311) */
  interface SpTrieStorageProof extends Struct {
    readonly trieNodes: BTreeSet<Bytes>;
  }

  /** @name PolkadotCorePrimitivesInboundDownwardMessage (314) */
  interface PolkadotCorePrimitivesInboundDownwardMessage extends Struct {
    readonly sentAt: u32;
    readonly msg: Bytes;
  }

  /** @name PolkadotCorePrimitivesInboundHrmpMessage (317) */
  interface PolkadotCorePrimitivesInboundHrmpMessage extends Struct {
    readonly sentAt: u32;
    readonly data: Bytes;
  }

  /** @name ParachainInfoCall (320) */
  type ParachainInfoCall = Null;

  /** @name PalletSessionCall (321) */
  interface PalletSessionCall extends Enum {
    readonly isSetKeys: boolean;
    readonly asSetKeys: {
      readonly keys_: RococoParachainRuntimeSessionKeys;
      readonly proof: Bytes;
    } & Struct;
    readonly isPurgeKeys: boolean;
    readonly type: 'SetKeys' | 'PurgeKeys';
  }

  /** @name RococoParachainRuntimeSessionKeys (322) */
  interface RococoParachainRuntimeSessionKeys extends Struct {
    readonly aura: SpConsensusAuraSr25519AppSr25519Public;
  }

  /** @name SpConsensusAuraSr25519AppSr25519Public (323) */
  interface SpConsensusAuraSr25519AppSr25519Public extends SpCoreSr25519Public {}

  /** @name SpCoreSr25519Public (324) */
  interface SpCoreSr25519Public extends U8aFixed {}

  /** @name PalletParachainStakingCall (325) */
  interface PalletParachainStakingCall extends Enum {
    readonly isSetStakingExpectations: boolean;
    readonly asSetStakingExpectations: {
      readonly expectations: {
      readonly min: u128;
      readonly ideal: u128;
      readonly max: u128;
    } & Struct;
    } & Struct;
    readonly isSetInflation: boolean;
    readonly asSetInflation: {
      readonly schedule: {
      readonly min: Perbill;
      readonly ideal: Perbill;
      readonly max: Perbill;
    } & Struct;
    } & Struct;
    readonly isSetParachainBondAccount: boolean;
    readonly asSetParachainBondAccount: {
      readonly new_: AccountId32;
    } & Struct;
    readonly isSetParachainBondReservePercent: boolean;
    readonly asSetParachainBondReservePercent: {
      readonly new_: Percent;
    } & Struct;
    readonly isSetTotalSelected: boolean;
    readonly asSetTotalSelected: {
      readonly new_: u32;
    } & Struct;
    readonly isSetCollatorCommission: boolean;
    readonly asSetCollatorCommission: {
      readonly new_: Perbill;
    } & Struct;
    readonly isSetBlocksPerRound: boolean;
    readonly asSetBlocksPerRound: {
      readonly new_: u32;
    } & Struct;
    readonly isAddCandidatesWhitelist: boolean;
    readonly asAddCandidatesWhitelist: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isRemoveCandidatesWhitelist: boolean;
    readonly asRemoveCandidatesWhitelist: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isJoinCandidates: boolean;
    readonly asJoinCandidates: {
      readonly bond: u128;
    } & Struct;
    readonly isScheduleLeaveCandidates: boolean;
    readonly isExecuteLeaveCandidates: boolean;
    readonly asExecuteLeaveCandidates: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isCancelLeaveCandidates: boolean;
    readonly isGoOffline: boolean;
    readonly isGoOnline: boolean;
    readonly isCandidateBondMore: boolean;
    readonly asCandidateBondMore: {
      readonly more: u128;
    } & Struct;
    readonly isScheduleCandidateBondLess: boolean;
    readonly asScheduleCandidateBondLess: {
      readonly less: u128;
    } & Struct;
    readonly isExecuteCandidateBondLess: boolean;
    readonly asExecuteCandidateBondLess: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isCancelCandidateBondLess: boolean;
    readonly isDelegate: boolean;
    readonly asDelegate: {
      readonly candidate: AccountId32;
      readonly amount: u128;
    } & Struct;
    readonly isDelegateWithAutoCompound: boolean;
    readonly asDelegateWithAutoCompound: {
      readonly candidate: AccountId32;
      readonly amount: u128;
      readonly autoCompound: Percent;
    } & Struct;
    readonly isScheduleLeaveDelegators: boolean;
    readonly isExecuteLeaveDelegators: boolean;
    readonly asExecuteLeaveDelegators: {
      readonly delegator: AccountId32;
    } & Struct;
    readonly isCancelLeaveDelegators: boolean;
    readonly isScheduleRevokeDelegation: boolean;
    readonly asScheduleRevokeDelegation: {
      readonly collator: AccountId32;
    } & Struct;
    readonly isDelegatorBondMore: boolean;
    readonly asDelegatorBondMore: {
      readonly candidate: AccountId32;
      readonly more: u128;
    } & Struct;
    readonly isScheduleDelegatorBondLess: boolean;
    readonly asScheduleDelegatorBondLess: {
      readonly candidate: AccountId32;
      readonly less: u128;
    } & Struct;
    readonly isExecuteDelegationRequest: boolean;
    readonly asExecuteDelegationRequest: {
      readonly delegator: AccountId32;
      readonly candidate: AccountId32;
    } & Struct;
    readonly isCancelDelegationRequest: boolean;
    readonly asCancelDelegationRequest: {
      readonly candidate: AccountId32;
    } & Struct;
    readonly isSetAutoCompound: boolean;
    readonly asSetAutoCompound: {
      readonly candidate: AccountId32;
      readonly value: Percent;
    } & Struct;
    readonly type: 'SetStakingExpectations' | 'SetInflation' | 'SetParachainBondAccount' | 'SetParachainBondReservePercent' | 'SetTotalSelected' | 'SetCollatorCommission' | 'SetBlocksPerRound' | 'AddCandidatesWhitelist' | 'RemoveCandidatesWhitelist' | 'JoinCandidates' | 'ScheduleLeaveCandidates' | 'ExecuteLeaveCandidates' | 'CancelLeaveCandidates' | 'GoOffline' | 'GoOnline' | 'CandidateBondMore' | 'ScheduleCandidateBondLess' | 'ExecuteCandidateBondLess' | 'CancelCandidateBondLess' | 'Delegate' | 'DelegateWithAutoCompound' | 'ScheduleLeaveDelegators' | 'ExecuteLeaveDelegators' | 'CancelLeaveDelegators' | 'ScheduleRevokeDelegation' | 'DelegatorBondMore' | 'ScheduleDelegatorBondLess' | 'ExecuteDelegationRequest' | 'CancelDelegationRequest' | 'SetAutoCompound';
  }

  /** @name CumulusPalletXcmpQueueCall (328) */
  interface CumulusPalletXcmpQueueCall extends Enum {
    readonly isServiceOverweight: boolean;
    readonly asServiceOverweight: {
      readonly index: u64;
      readonly weightLimit: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isSuspendXcmExecution: boolean;
    readonly isResumeXcmExecution: boolean;
    readonly isUpdateSuspendThreshold: boolean;
    readonly asUpdateSuspendThreshold: {
      readonly new_: u32;
    } & Struct;
    readonly isUpdateDropThreshold: boolean;
    readonly asUpdateDropThreshold: {
      readonly new_: u32;
    } & Struct;
    readonly isUpdateResumeThreshold: boolean;
    readonly asUpdateResumeThreshold: {
      readonly new_: u32;
    } & Struct;
    readonly isUpdateThresholdWeight: boolean;
    readonly asUpdateThresholdWeight: {
      readonly new_: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isUpdateWeightRestrictDecay: boolean;
    readonly asUpdateWeightRestrictDecay: {
      readonly new_: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isUpdateXcmpMaxIndividualWeight: boolean;
    readonly asUpdateXcmpMaxIndividualWeight: {
      readonly new_: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type: 'ServiceOverweight' | 'SuspendXcmExecution' | 'ResumeXcmExecution' | 'UpdateSuspendThreshold' | 'UpdateDropThreshold' | 'UpdateResumeThreshold' | 'UpdateThresholdWeight' | 'UpdateWeightRestrictDecay' | 'UpdateXcmpMaxIndividualWeight';
  }

  /** @name PalletXcmCall (329) */
  interface PalletXcmCall extends Enum {
    readonly isSend: boolean;
    readonly asSend: {
      readonly dest: XcmVersionedMultiLocation;
      readonly message: XcmVersionedXcm;
    } & Struct;
    readonly isTeleportAssets: boolean;
    readonly asTeleportAssets: {
      readonly dest: XcmVersionedMultiLocation;
      readonly beneficiary: XcmVersionedMultiLocation;
      readonly assets: XcmVersionedMultiAssets;
      readonly feeAssetItem: u32;
    } & Struct;
    readonly isReserveTransferAssets: boolean;
    readonly asReserveTransferAssets: {
      readonly dest: XcmVersionedMultiLocation;
      readonly beneficiary: XcmVersionedMultiLocation;
      readonly assets: XcmVersionedMultiAssets;
      readonly feeAssetItem: u32;
    } & Struct;
    readonly isExecute: boolean;
    readonly asExecute: {
      readonly message: XcmVersionedXcm;
      readonly maxWeight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isForceXcmVersion: boolean;
    readonly asForceXcmVersion: {
      readonly location: XcmV3MultiLocation;
      readonly xcmVersion: u32;
    } & Struct;
    readonly isForceDefaultXcmVersion: boolean;
    readonly asForceDefaultXcmVersion: {
      readonly maybeXcmVersion: Option<u32>;
    } & Struct;
    readonly isForceSubscribeVersionNotify: boolean;
    readonly asForceSubscribeVersionNotify: {
      readonly location: XcmVersionedMultiLocation;
    } & Struct;
    readonly isForceUnsubscribeVersionNotify: boolean;
    readonly asForceUnsubscribeVersionNotify: {
      readonly location: XcmVersionedMultiLocation;
    } & Struct;
    readonly isLimitedReserveTransferAssets: boolean;
    readonly asLimitedReserveTransferAssets: {
      readonly dest: XcmVersionedMultiLocation;
      readonly beneficiary: XcmVersionedMultiLocation;
      readonly assets: XcmVersionedMultiAssets;
      readonly feeAssetItem: u32;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isLimitedTeleportAssets: boolean;
    readonly asLimitedTeleportAssets: {
      readonly dest: XcmVersionedMultiLocation;
      readonly beneficiary: XcmVersionedMultiLocation;
      readonly assets: XcmVersionedMultiAssets;
      readonly feeAssetItem: u32;
      readonly weightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isForceSuspension: boolean;
    readonly asForceSuspension: {
      readonly suspended: bool;
    } & Struct;
    readonly type: 'Send' | 'TeleportAssets' | 'ReserveTransferAssets' | 'Execute' | 'ForceXcmVersion' | 'ForceDefaultXcmVersion' | 'ForceSubscribeVersionNotify' | 'ForceUnsubscribeVersionNotify' | 'LimitedReserveTransferAssets' | 'LimitedTeleportAssets' | 'ForceSuspension';
  }

  /** @name XcmVersionedXcm (330) */
  interface XcmVersionedXcm extends Enum {
    readonly isV2: boolean;
    readonly asV2: XcmV2Xcm;
    readonly isV3: boolean;
    readonly asV3: XcmV3Xcm;
    readonly type: 'V2' | 'V3';
  }

  /** @name XcmV2Xcm (331) */
  interface XcmV2Xcm extends Vec<XcmV2Instruction> {}

  /** @name XcmV2Instruction (333) */
  interface XcmV2Instruction extends Enum {
    readonly isWithdrawAsset: boolean;
    readonly asWithdrawAsset: XcmV2MultiassetMultiAssets;
    readonly isReserveAssetDeposited: boolean;
    readonly asReserveAssetDeposited: XcmV2MultiassetMultiAssets;
    readonly isReceiveTeleportedAsset: boolean;
    readonly asReceiveTeleportedAsset: XcmV2MultiassetMultiAssets;
    readonly isQueryResponse: boolean;
    readonly asQueryResponse: {
      readonly queryId: Compact<u64>;
      readonly response: XcmV2Response;
      readonly maxWeight: Compact<u64>;
    } & Struct;
    readonly isTransferAsset: boolean;
    readonly asTransferAsset: {
      readonly assets: XcmV2MultiassetMultiAssets;
      readonly beneficiary: XcmV2MultiLocation;
    } & Struct;
    readonly isTransferReserveAsset: boolean;
    readonly asTransferReserveAsset: {
      readonly assets: XcmV2MultiassetMultiAssets;
      readonly dest: XcmV2MultiLocation;
      readonly xcm: XcmV2Xcm;
    } & Struct;
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly originType: XcmV2OriginKind;
      readonly requireWeightAtMost: Compact<u64>;
      readonly call: XcmDoubleEncoded;
    } & Struct;
    readonly isHrmpNewChannelOpenRequest: boolean;
    readonly asHrmpNewChannelOpenRequest: {
      readonly sender: Compact<u32>;
      readonly maxMessageSize: Compact<u32>;
      readonly maxCapacity: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelAccepted: boolean;
    readonly asHrmpChannelAccepted: {
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isHrmpChannelClosing: boolean;
    readonly asHrmpChannelClosing: {
      readonly initiator: Compact<u32>;
      readonly sender: Compact<u32>;
      readonly recipient: Compact<u32>;
    } & Struct;
    readonly isClearOrigin: boolean;
    readonly isDescendOrigin: boolean;
    readonly asDescendOrigin: XcmV2MultilocationJunctions;
    readonly isReportError: boolean;
    readonly asReportError: {
      readonly queryId: Compact<u64>;
      readonly dest: XcmV2MultiLocation;
      readonly maxResponseWeight: Compact<u64>;
    } & Struct;
    readonly isDepositAsset: boolean;
    readonly asDepositAsset: {
      readonly assets: XcmV2MultiassetMultiAssetFilter;
      readonly maxAssets: Compact<u32>;
      readonly beneficiary: XcmV2MultiLocation;
    } & Struct;
    readonly isDepositReserveAsset: boolean;
    readonly asDepositReserveAsset: {
      readonly assets: XcmV2MultiassetMultiAssetFilter;
      readonly maxAssets: Compact<u32>;
      readonly dest: XcmV2MultiLocation;
      readonly xcm: XcmV2Xcm;
    } & Struct;
    readonly isExchangeAsset: boolean;
    readonly asExchangeAsset: {
      readonly give: XcmV2MultiassetMultiAssetFilter;
      readonly receive: XcmV2MultiassetMultiAssets;
    } & Struct;
    readonly isInitiateReserveWithdraw: boolean;
    readonly asInitiateReserveWithdraw: {
      readonly assets: XcmV2MultiassetMultiAssetFilter;
      readonly reserve: XcmV2MultiLocation;
      readonly xcm: XcmV2Xcm;
    } & Struct;
    readonly isInitiateTeleport: boolean;
    readonly asInitiateTeleport: {
      readonly assets: XcmV2MultiassetMultiAssetFilter;
      readonly dest: XcmV2MultiLocation;
      readonly xcm: XcmV2Xcm;
    } & Struct;
    readonly isQueryHolding: boolean;
    readonly asQueryHolding: {
      readonly queryId: Compact<u64>;
      readonly dest: XcmV2MultiLocation;
      readonly assets: XcmV2MultiassetMultiAssetFilter;
      readonly maxResponseWeight: Compact<u64>;
    } & Struct;
    readonly isBuyExecution: boolean;
    readonly asBuyExecution: {
      readonly fees: XcmV2MultiAsset;
      readonly weightLimit: XcmV2WeightLimit;
    } & Struct;
    readonly isRefundSurplus: boolean;
    readonly isSetErrorHandler: boolean;
    readonly asSetErrorHandler: XcmV2Xcm;
    readonly isSetAppendix: boolean;
    readonly asSetAppendix: XcmV2Xcm;
    readonly isClearError: boolean;
    readonly isClaimAsset: boolean;
    readonly asClaimAsset: {
      readonly assets: XcmV2MultiassetMultiAssets;
      readonly ticket: XcmV2MultiLocation;
    } & Struct;
    readonly isTrap: boolean;
    readonly asTrap: Compact<u64>;
    readonly isSubscribeVersion: boolean;
    readonly asSubscribeVersion: {
      readonly queryId: Compact<u64>;
      readonly maxResponseWeight: Compact<u64>;
    } & Struct;
    readonly isUnsubscribeVersion: boolean;
    readonly type: 'WithdrawAsset' | 'ReserveAssetDeposited' | 'ReceiveTeleportedAsset' | 'QueryResponse' | 'TransferAsset' | 'TransferReserveAsset' | 'Transact' | 'HrmpNewChannelOpenRequest' | 'HrmpChannelAccepted' | 'HrmpChannelClosing' | 'ClearOrigin' | 'DescendOrigin' | 'ReportError' | 'DepositAsset' | 'DepositReserveAsset' | 'ExchangeAsset' | 'InitiateReserveWithdraw' | 'InitiateTeleport' | 'QueryHolding' | 'BuyExecution' | 'RefundSurplus' | 'SetErrorHandler' | 'SetAppendix' | 'ClearError' | 'ClaimAsset' | 'Trap' | 'SubscribeVersion' | 'UnsubscribeVersion';
  }

  /** @name XcmV2Response (334) */
  interface XcmV2Response extends Enum {
    readonly isNull: boolean;
    readonly isAssets: boolean;
    readonly asAssets: XcmV2MultiassetMultiAssets;
    readonly isExecutionResult: boolean;
    readonly asExecutionResult: Option<ITuple<[u32, XcmV2TraitsError]>>;
    readonly isVersion: boolean;
    readonly asVersion: u32;
    readonly type: 'Null' | 'Assets' | 'ExecutionResult' | 'Version';
  }

  /** @name XcmV2TraitsError (337) */
  interface XcmV2TraitsError extends Enum {
    readonly isOverflow: boolean;
    readonly isUnimplemented: boolean;
    readonly isUntrustedReserveLocation: boolean;
    readonly isUntrustedTeleportLocation: boolean;
    readonly isMultiLocationFull: boolean;
    readonly isMultiLocationNotInvertible: boolean;
    readonly isBadOrigin: boolean;
    readonly isInvalidLocation: boolean;
    readonly isAssetNotFound: boolean;
    readonly isFailedToTransactAsset: boolean;
    readonly isNotWithdrawable: boolean;
    readonly isLocationCannotHold: boolean;
    readonly isExceedsMaxMessageSize: boolean;
    readonly isDestinationUnsupported: boolean;
    readonly isTransport: boolean;
    readonly isUnroutable: boolean;
    readonly isUnknownClaim: boolean;
    readonly isFailedToDecode: boolean;
    readonly isMaxWeightInvalid: boolean;
    readonly isNotHoldingFees: boolean;
    readonly isTooExpensive: boolean;
    readonly isTrap: boolean;
    readonly asTrap: u64;
    readonly isUnhandledXcmVersion: boolean;
    readonly isWeightLimitReached: boolean;
    readonly asWeightLimitReached: u64;
    readonly isBarrier: boolean;
    readonly isWeightNotComputable: boolean;
    readonly type: 'Overflow' | 'Unimplemented' | 'UntrustedReserveLocation' | 'UntrustedTeleportLocation' | 'MultiLocationFull' | 'MultiLocationNotInvertible' | 'BadOrigin' | 'InvalidLocation' | 'AssetNotFound' | 'FailedToTransactAsset' | 'NotWithdrawable' | 'LocationCannotHold' | 'ExceedsMaxMessageSize' | 'DestinationUnsupported' | 'Transport' | 'Unroutable' | 'UnknownClaim' | 'FailedToDecode' | 'MaxWeightInvalid' | 'NotHoldingFees' | 'TooExpensive' | 'Trap' | 'UnhandledXcmVersion' | 'WeightLimitReached' | 'Barrier' | 'WeightNotComputable';
  }

  /** @name XcmV2MultiassetMultiAssetFilter (338) */
  interface XcmV2MultiassetMultiAssetFilter extends Enum {
    readonly isDefinite: boolean;
    readonly asDefinite: XcmV2MultiassetMultiAssets;
    readonly isWild: boolean;
    readonly asWild: XcmV2MultiassetWildMultiAsset;
    readonly type: 'Definite' | 'Wild';
  }

  /** @name XcmV2MultiassetWildMultiAsset (339) */
  interface XcmV2MultiassetWildMultiAsset extends Enum {
    readonly isAll: boolean;
    readonly isAllOf: boolean;
    readonly asAllOf: {
      readonly id: XcmV2MultiassetAssetId;
      readonly fun: XcmV2MultiassetWildFungibility;
    } & Struct;
    readonly type: 'All' | 'AllOf';
  }

  /** @name XcmV2MultiassetWildFungibility (340) */
  interface XcmV2MultiassetWildFungibility extends Enum {
    readonly isFungible: boolean;
    readonly isNonFungible: boolean;
    readonly type: 'Fungible' | 'NonFungible';
  }

  /** @name XcmV2WeightLimit (341) */
  interface XcmV2WeightLimit extends Enum {
    readonly isUnlimited: boolean;
    readonly isLimited: boolean;
    readonly asLimited: Compact<u64>;
    readonly type: 'Unlimited' | 'Limited';
  }

  /** @name CumulusPalletXcmCall (350) */
  type CumulusPalletXcmCall = Null;

  /** @name CumulusPalletDmpQueueCall (351) */
  interface CumulusPalletDmpQueueCall extends Enum {
    readonly isServiceOverweight: boolean;
    readonly asServiceOverweight: {
      readonly index: u64;
      readonly weightLimit: SpWeightsWeightV2Weight;
    } & Struct;
    readonly type: 'ServiceOverweight';
  }

  /** @name OrmlXtokensModuleCall (352) */
  interface OrmlXtokensModuleCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly currencyId: RuntimeCommonXcmImplCurrencyId;
      readonly amount: u128;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isTransferMultiasset: boolean;
    readonly asTransferMultiasset: {
      readonly asset: XcmVersionedMultiAsset;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isTransferWithFee: boolean;
    readonly asTransferWithFee: {
      readonly currencyId: RuntimeCommonXcmImplCurrencyId;
      readonly amount: u128;
      readonly fee: u128;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isTransferMultiassetWithFee: boolean;
    readonly asTransferMultiassetWithFee: {
      readonly asset: XcmVersionedMultiAsset;
      readonly fee: XcmVersionedMultiAsset;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isTransferMulticurrencies: boolean;
    readonly asTransferMulticurrencies: {
      readonly currencies: Vec<ITuple<[RuntimeCommonXcmImplCurrencyId, u128]>>;
      readonly feeItem: u32;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly isTransferMultiassets: boolean;
    readonly asTransferMultiassets: {
      readonly assets: XcmVersionedMultiAssets;
      readonly feeItem: u32;
      readonly dest: XcmVersionedMultiLocation;
      readonly destWeightLimit: XcmV3WeightLimit;
    } & Struct;
    readonly type: 'Transfer' | 'TransferMultiasset' | 'TransferWithFee' | 'TransferMultiassetWithFee' | 'TransferMulticurrencies' | 'TransferMultiassets';
  }

  /** @name XcmVersionedMultiAsset (353) */
  interface XcmVersionedMultiAsset extends Enum {
    readonly isV2: boolean;
    readonly asV2: XcmV2MultiAsset;
    readonly isV3: boolean;
    readonly asV3: XcmV3MultiAsset;
    readonly type: 'V2' | 'V3';
  }

  /** @name OrmlTokensModuleCall (356) */
  interface OrmlTokensModuleCall extends Enum {
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly dest: MultiAddress;
      readonly currencyId: u128;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isTransferAll: boolean;
    readonly asTransferAll: {
      readonly dest: MultiAddress;
      readonly currencyId: u128;
      readonly keepAlive: bool;
    } & Struct;
    readonly isTransferKeepAlive: boolean;
    readonly asTransferKeepAlive: {
      readonly dest: MultiAddress;
      readonly currencyId: u128;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isForceTransfer: boolean;
    readonly asForceTransfer: {
      readonly source: MultiAddress;
      readonly dest: MultiAddress;
      readonly currencyId: u128;
      readonly amount: Compact<u128>;
    } & Struct;
    readonly isSetBalance: boolean;
    readonly asSetBalance: {
      readonly who: MultiAddress;
      readonly currencyId: u128;
      readonly newFree: Compact<u128>;
      readonly newReserved: Compact<u128>;
    } & Struct;
    readonly type: 'Transfer' | 'TransferAll' | 'TransferKeepAlive' | 'ForceTransfer' | 'SetBalance';
  }

  /** @name PalletBridgeCall (357) */
  interface PalletBridgeCall extends Enum {
    readonly isSetThreshold: boolean;
    readonly asSetThreshold: {
      readonly threshold: u32;
    } & Struct;
    readonly isSetResource: boolean;
    readonly asSetResource: {
      readonly id: U8aFixed;
      readonly method: Bytes;
    } & Struct;
    readonly isRemoveResource: boolean;
    readonly asRemoveResource: {
      readonly id: U8aFixed;
    } & Struct;
    readonly isWhitelistChain: boolean;
    readonly asWhitelistChain: {
      readonly id: u8;
    } & Struct;
    readonly isAddRelayer: boolean;
    readonly asAddRelayer: {
      readonly v: AccountId32;
    } & Struct;
    readonly isRemoveRelayer: boolean;
    readonly asRemoveRelayer: {
      readonly v: AccountId32;
    } & Struct;
    readonly isUpdateFee: boolean;
    readonly asUpdateFee: {
      readonly destId: u8;
      readonly fee: u128;
    } & Struct;
    readonly isAcknowledgeProposal: boolean;
    readonly asAcknowledgeProposal: {
      readonly nonce: u64;
      readonly srcId: u8;
      readonly rId: U8aFixed;
      readonly call: Call;
    } & Struct;
    readonly isRejectProposal: boolean;
    readonly asRejectProposal: {
      readonly nonce: u64;
      readonly srcId: u8;
      readonly rId: U8aFixed;
      readonly call: Call;
    } & Struct;
    readonly isEvalVoteState: boolean;
    readonly asEvalVoteState: {
      readonly nonce: u64;
      readonly srcId: u8;
      readonly prop: Call;
    } & Struct;
    readonly type: 'SetThreshold' | 'SetResource' | 'RemoveResource' | 'WhitelistChain' | 'AddRelayer' | 'RemoveRelayer' | 'UpdateFee' | 'AcknowledgeProposal' | 'RejectProposal' | 'EvalVoteState';
  }

  /** @name PalletBridgeTransferCall (358) */
  interface PalletBridgeTransferCall extends Enum {
    readonly isTransferNative: boolean;
    readonly asTransferNative: {
      readonly amount: u128;
      readonly recipient: Bytes;
      readonly destId: u8;
    } & Struct;
    readonly isTransfer: boolean;
    readonly asTransfer: {
      readonly to: AccountId32;
      readonly amount: u128;
      readonly rid: U8aFixed;
    } & Struct;
    readonly isSetMaximumIssuance: boolean;
    readonly asSetMaximumIssuance: {
      readonly maximumIssuance: u128;
    } & Struct;
    readonly isSetExternalBalances: boolean;
    readonly asSetExternalBalances: {
      readonly externalBalances: u128;
    } & Struct;
    readonly type: 'TransferNative' | 'Transfer' | 'SetMaximumIssuance' | 'SetExternalBalances';
  }

  /** @name PalletExtrinsicFilterCall (359) */
  interface PalletExtrinsicFilterCall extends Enum {
    readonly isSetMode: boolean;
    readonly asSetMode: {
      readonly mode: PalletExtrinsicFilterOperationalMode;
    } & Struct;
    readonly isBlockExtrinsics: boolean;
    readonly asBlockExtrinsics: {
      readonly palletNameBytes: Bytes;
      readonly functionNameBytes: Option<Bytes>;
    } & Struct;
    readonly isUnblockExtrinsics: boolean;
    readonly asUnblockExtrinsics: {
      readonly palletNameBytes: Bytes;
      readonly functionNameBytes: Option<Bytes>;
    } & Struct;
    readonly type: 'SetMode' | 'BlockExtrinsics' | 'UnblockExtrinsics';
  }

  /** @name PalletIdentityManagementCall (360) */
  interface PalletIdentityManagementCall extends Enum {
    readonly isAddDelegatee: boolean;
    readonly asAddDelegatee: {
      readonly account: AccountId32;
    } & Struct;
    readonly isRemoveDelegatee: boolean;
    readonly asRemoveDelegatee: {
      readonly account: AccountId32;
    } & Struct;
    readonly isLinkIdentity: boolean;
    readonly asLinkIdentity: {
      readonly shard: H256;
      readonly user: AccountId32;
      readonly encryptedIdentity: Bytes;
      readonly encryptedValidationData: Bytes;
      readonly encryptedWeb3networks: Bytes;
    } & Struct;
    readonly isDeactivateIdentity: boolean;
    readonly asDeactivateIdentity: {
      readonly shard: H256;
      readonly encryptedIdentity: Bytes;
    } & Struct;
    readonly isActivateIdentity: boolean;
    readonly asActivateIdentity: {
      readonly shard: H256;
      readonly encryptedIdentity: Bytes;
    } & Struct;
    readonly isIdentityLinked: boolean;
    readonly asIdentityLinked: {
      readonly primeIdentity: CorePrimitivesIdentity;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isIdentityDeactivated: boolean;
    readonly asIdentityDeactivated: {
      readonly primeIdentity: CorePrimitivesIdentity;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isIdentityActivated: boolean;
    readonly asIdentityActivated: {
      readonly primeIdentity: CorePrimitivesIdentity;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isIdentityNetworksSet: boolean;
    readonly asIdentityNetworksSet: {
      readonly primeIdentity: CorePrimitivesIdentity;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isSomeError: boolean;
    readonly asSomeError: {
      readonly primeIdentity: Option<CorePrimitivesIdentity>;
      readonly error: CorePrimitivesErrorImpError;
      readonly reqExtHash: H256;
    } & Struct;
    readonly type: 'AddDelegatee' | 'RemoveDelegatee' | 'LinkIdentity' | 'DeactivateIdentity' | 'ActivateIdentity' | 'IdentityLinked' | 'IdentityDeactivated' | 'IdentityActivated' | 'IdentityNetworksSet' | 'SomeError';
  }

  /** @name CorePrimitivesErrorImpError (361) */
  interface CorePrimitivesErrorImpError extends Enum {
    readonly isLinkIdentityFailed: boolean;
    readonly asLinkIdentityFailed: CorePrimitivesErrorErrorDetail;
    readonly isDeactivateIdentityFailed: boolean;
    readonly asDeactivateIdentityFailed: CorePrimitivesErrorErrorDetail;
    readonly isActivateIdentityFailed: boolean;
    readonly asActivateIdentityFailed: CorePrimitivesErrorErrorDetail;
    readonly isUnclassifiedError: boolean;
    readonly asUnclassifiedError: CorePrimitivesErrorErrorDetail;
    readonly type: 'LinkIdentityFailed' | 'DeactivateIdentityFailed' | 'ActivateIdentityFailed' | 'UnclassifiedError';
  }

  /** @name PalletAssetManagerCall (362) */
  interface PalletAssetManagerCall extends Enum {
    readonly isRegisterForeignAssetType: boolean;
    readonly asRegisterForeignAssetType: {
      readonly assetType: RuntimeCommonXcmImplCurrencyId;
      readonly metadata: PalletAssetManagerAssetMetadata;
    } & Struct;
    readonly isUpdateForeignAssetMetadata: boolean;
    readonly asUpdateForeignAssetMetadata: {
      readonly assetId: u128;
      readonly metadata: PalletAssetManagerAssetMetadata;
    } & Struct;
    readonly isSetAssetUnitsPerSecond: boolean;
    readonly asSetAssetUnitsPerSecond: {
      readonly assetId: u128;
      readonly unitsPerSecond: u128;
    } & Struct;
    readonly isAddAssetType: boolean;
    readonly asAddAssetType: {
      readonly assetId: u128;
      readonly newAssetType: RuntimeCommonXcmImplCurrencyId;
    } & Struct;
    readonly isRemoveAssetType: boolean;
    readonly asRemoveAssetType: {
      readonly assetType: RuntimeCommonXcmImplCurrencyId;
      readonly newDefaultAssetType: Option<RuntimeCommonXcmImplCurrencyId>;
    } & Struct;
    readonly type: 'RegisterForeignAssetType' | 'UpdateForeignAssetMetadata' | 'SetAssetUnitsPerSecond' | 'AddAssetType' | 'RemoveAssetType';
  }

  /** @name PalletVcManagementCall (364) */
  interface PalletVcManagementCall extends Enum {
    readonly isAddDelegatee: boolean;
    readonly asAddDelegatee: {
      readonly account: AccountId32;
    } & Struct;
    readonly isRemoveDelegatee: boolean;
    readonly asRemoveDelegatee: {
      readonly account: AccountId32;
    } & Struct;
    readonly isRequestVc: boolean;
    readonly asRequestVc: {
      readonly shard: H256;
      readonly assertion: CorePrimitivesAssertion;
    } & Struct;
    readonly isSetAdmin: boolean;
    readonly asSetAdmin: {
      readonly new_: AccountId32;
    } & Struct;
    readonly isAddSchema: boolean;
    readonly asAddSchema: {
      readonly shard: H256;
      readonly id: Bytes;
      readonly content: Bytes;
    } & Struct;
    readonly isDisableSchema: boolean;
    readonly asDisableSchema: {
      readonly shard: H256;
      readonly index: u64;
    } & Struct;
    readonly isActivateSchema: boolean;
    readonly asActivateSchema: {
      readonly shard: H256;
      readonly index: u64;
    } & Struct;
    readonly isRevokeSchema: boolean;
    readonly asRevokeSchema: {
      readonly shard: H256;
      readonly index: u64;
    } & Struct;
    readonly isVcIssued: boolean;
    readonly asVcIssued: {
      readonly identity: CorePrimitivesIdentity;
      readonly assertion: CorePrimitivesAssertion;
      readonly idGraphHash: H256;
      readonly reqExtHash: H256;
    } & Struct;
    readonly isSomeError: boolean;
    readonly asSomeError: {
      readonly identity: Option<CorePrimitivesIdentity>;
      readonly error: CorePrimitivesErrorVcmpError;
      readonly reqExtHash: H256;
    } & Struct;
    readonly type: 'AddDelegatee' | 'RemoveDelegatee' | 'RequestVc' | 'SetAdmin' | 'AddSchema' | 'DisableSchema' | 'ActivateSchema' | 'RevokeSchema' | 'VcIssued' | 'SomeError';
  }

  /** @name CorePrimitivesErrorVcmpError (365) */
  interface CorePrimitivesErrorVcmpError extends Enum {
    readonly isRequestVCFailed: boolean;
    readonly asRequestVCFailed: ITuple<[CorePrimitivesAssertion, CorePrimitivesErrorErrorDetail]>;
    readonly isUnclassifiedError: boolean;
    readonly asUnclassifiedError: CorePrimitivesErrorErrorDetail;
    readonly type: 'RequestVCFailed' | 'UnclassifiedError';
  }

  /** @name PalletGroupCall (366) */
  interface PalletGroupCall extends Enum {
    readonly isAddGroupMember: boolean;
    readonly asAddGroupMember: {
      readonly v: AccountId32;
    } & Struct;
    readonly isBatchAddGroupMembers: boolean;
    readonly asBatchAddGroupMembers: {
      readonly vs: Vec<AccountId32>;
    } & Struct;
    readonly isRemoveGroupMember: boolean;
    readonly asRemoveGroupMember: {
      readonly v: AccountId32;
    } & Struct;
    readonly isBatchRemoveGroupMembers: boolean;
    readonly asBatchRemoveGroupMembers: {
      readonly vs: Vec<AccountId32>;
    } & Struct;
    readonly isSwitchGroupControlOn: boolean;
    readonly isSwitchGroupControlOff: boolean;
    readonly type: 'AddGroupMember' | 'BatchAddGroupMembers' | 'RemoveGroupMember' | 'BatchRemoveGroupMembers' | 'SwitchGroupControlOn' | 'SwitchGroupControlOff';
  }

  /** @name PalletBitacrossCall (368) */
  interface PalletBitacrossCall extends Enum {
    readonly isSetAdmin: boolean;
    readonly asSetAdmin: {
      readonly newAdmin: AccountId32;
    } & Struct;
    readonly isAddRelayer: boolean;
    readonly asAddRelayer: {
      readonly account: CorePrimitivesIdentity;
    } & Struct;
    readonly isRemoveRelayer: boolean;
    readonly asRemoveRelayer: {
      readonly account: CorePrimitivesIdentity;
    } & Struct;
    readonly isBtcWalletGenerated: boolean;
    readonly asBtcWalletGenerated: {
      readonly pubKey: U8aFixed;
    } & Struct;
    readonly isEthWalletGenerated: boolean;
    readonly asEthWalletGenerated: {
      readonly pubKey: U8aFixed;
    } & Struct;
    readonly isTaskComplete: boolean;
    readonly type: 'SetAdmin' | 'AddRelayer' | 'RemoveRelayer' | 'BtcWalletGenerated' | 'EthWalletGenerated' | 'TaskComplete';
  }

  /** @name PalletBitacrossMimicCall (369) */
  interface PalletBitacrossMimicCall extends Enum {
    readonly isWriteBtcToEth: boolean;
    readonly asWriteBtcToEth: {
      readonly btcSender: U8aFixed;
      readonly data: PalletBitacrossMimicCustodialTypeBtcToEth;
    } & Struct;
    readonly isWriteEthToBtc: boolean;
    readonly asWriteEthToBtc: {
      readonly ethSender: H160;
      readonly data: PalletBitacrossMimicCustodialTypeEthToBtc;
    } & Struct;
    readonly type: 'WriteBtcToEth' | 'WriteEthToBtc';
  }

  /** @name PalletBitacrossMimicCustodialTypeBtcToEth (370) */
  interface PalletBitacrossMimicCustodialTypeBtcToEth extends Struct {
    readonly txIndex: u64;
    readonly btcTxHash: H256;
    readonly ethereumReceiver: H160;
    readonly ethereumTxHash: H256;
    readonly ethTxStatus: bool;
    readonly symbol: Text;
    readonly amount: U256;
    readonly txTimestamp: u64;
  }

  /** @name PalletBitacrossMimicCustodialTypeEthToBtc (373) */
  interface PalletBitacrossMimicCustodialTypeEthToBtc extends Struct {
    readonly txIndex: u64;
    readonly btcTxHash: H256;
    readonly btcReceiver: U8aFixed;
    readonly btcReceiverLength: u32;
    readonly ethereumTxHash: H256;
    readonly ethTxStatus: bool;
    readonly symbol: Text;
    readonly amount: U256;
    readonly txTimestamp: u64;
  }

  /** @name PalletEvmAssertionsCall (374) */
  interface PalletEvmAssertionsCall extends Enum {
    readonly isCreateAssertion: boolean;
    readonly asCreateAssertion: {
      readonly id: H160;
      readonly byteCode: Bytes;
      readonly secrets: Vec<Bytes>;
    } & Struct;
    readonly type: 'CreateAssertion';
  }

  /** @name PalletTeebagCall (377) */
  interface PalletTeebagCall extends Enum {
    readonly isSetAdmin: boolean;
    readonly asSetAdmin: {
      readonly newAdmin: AccountId32;
    } & Struct;
    readonly isSetMode: boolean;
    readonly asSetMode: {
      readonly newMode: PalletTeebagOperationalMode;
    } & Struct;
    readonly isForceAddEnclave: boolean;
    readonly asForceAddEnclave: {
      readonly who: AccountId32;
      readonly enclave: PalletTeebagEnclave;
    } & Struct;
    readonly isForceRemoveEnclave: boolean;
    readonly asForceRemoveEnclave: {
      readonly who: AccountId32;
    } & Struct;
    readonly isForceRemoveEnclaveByMrenclave: boolean;
    readonly asForceRemoveEnclaveByMrenclave: {
      readonly mrenclave: U8aFixed;
    } & Struct;
    readonly isForceRemoveEnclaveByWorkerType: boolean;
    readonly asForceRemoveEnclaveByWorkerType: {
      readonly workerType: PalletTeebagWorkerType;
    } & Struct;
    readonly isForceAddAuthorizedEnclave: boolean;
    readonly asForceAddAuthorizedEnclave: {
      readonly workerType: PalletTeebagWorkerType;
      readonly mrenclave: U8aFixed;
    } & Struct;
    readonly isForceRemoveAuthorizedEnclave: boolean;
    readonly asForceRemoveAuthorizedEnclave: {
      readonly workerType: PalletTeebagWorkerType;
      readonly mrenclave: U8aFixed;
    } & Struct;
    readonly isRegisterEnclave: boolean;
    readonly asRegisterEnclave: {
      readonly workerType: PalletTeebagWorkerType;
      readonly workerMode: PalletTeebagWorkerMode;
      readonly attestation: Bytes;
      readonly workerUrl: Bytes;
      readonly shieldingPubkey: Option<Bytes>;
      readonly vcPubkey: Option<SpCoreEd25519Public>;
      readonly attestationType: PalletTeebagAttestationType;
    } & Struct;
    readonly isUnregisterEnclave: boolean;
    readonly isRegisterQuotingEnclave: boolean;
    readonly asRegisterQuotingEnclave: {
      readonly enclaveIdentity: Bytes;
      readonly signature: Bytes;
      readonly certificateChain: Bytes;
    } & Struct;
    readonly isRegisterTcbInfo: boolean;
    readonly asRegisterTcbInfo: {
      readonly tcbInfo: Bytes;
      readonly signature: Bytes;
      readonly certificateChain: Bytes;
    } & Struct;
    readonly isPostOpaqueTask: boolean;
    readonly asPostOpaqueTask: {
      readonly request: PalletTeebagRsaRequest;
    } & Struct;
    readonly isParentchainBlockProcessed: boolean;
    readonly asParentchainBlockProcessed: {
      readonly blockHash: H256;
      readonly blockNumber: u32;
      readonly taskMerkleRoot: H256;
    } & Struct;
    readonly isSidechainBlockImported: boolean;
    readonly asSidechainBlockImported: {
      readonly shard: H256;
      readonly blockNumber: u64;
      readonly nextFinalizationCandidateBlockNumber: u64;
      readonly blockHeaderHash: H256;
    } & Struct;
    readonly type: 'SetAdmin' | 'SetMode' | 'ForceAddEnclave' | 'ForceRemoveEnclave' | 'ForceRemoveEnclaveByMrenclave' | 'ForceRemoveEnclaveByWorkerType' | 'ForceAddAuthorizedEnclave' | 'ForceRemoveAuthorizedEnclave' | 'RegisterEnclave' | 'UnregisterEnclave' | 'RegisterQuotingEnclave' | 'RegisterTcbInfo' | 'PostOpaqueTask' | 'ParentchainBlockProcessed' | 'SidechainBlockImported';
  }

  /** @name PalletTeebagEnclave (378) */
  interface PalletTeebagEnclave extends Struct {
    readonly workerType: PalletTeebagWorkerType;
    readonly workerMode: PalletTeebagWorkerMode;
    readonly mrenclave: U8aFixed;
    readonly lastSeenTimestamp: u64;
    readonly url: Bytes;
    readonly shieldingPubkey: Option<Bytes>;
    readonly vcPubkey: Option<SpCoreEd25519Public>;
    readonly sgxBuildMode: PalletTeebagSgxBuildMode;
    readonly attestationType: PalletTeebagAttestationType;
  }

  /** @name PalletTeebagWorkerMode (379) */
  interface PalletTeebagWorkerMode extends Enum {
    readonly isOffChainWorker: boolean;
    readonly isSidechain: boolean;
    readonly type: 'OffChainWorker' | 'Sidechain';
  }

  /** @name SpCoreEd25519Public (381) */
  interface SpCoreEd25519Public extends U8aFixed {}

  /** @name PalletTeebagSgxBuildMode (382) */
  interface PalletTeebagSgxBuildMode extends Enum {
    readonly isProduction: boolean;
    readonly isDebug: boolean;
    readonly type: 'Production' | 'Debug';
  }

  /** @name PalletTeebagAttestationType (383) */
  interface PalletTeebagAttestationType extends Enum {
    readonly isIgnore: boolean;
    readonly isIas: boolean;
    readonly isDcap: boolean;
    readonly asDcap: PalletTeebagDcapProvider;
    readonly type: 'Ignore' | 'Ias' | 'Dcap';
  }

  /** @name PalletTeebagDcapProvider (384) */
  interface PalletTeebagDcapProvider extends Enum {
    readonly isMaa: boolean;
    readonly isIntel: boolean;
    readonly isLocal: boolean;
    readonly isIntegritee: boolean;
    readonly type: 'Maa' | 'Intel' | 'Local' | 'Integritee';
  }

  /** @name PalletEvmCall (385) */
  interface PalletEvmCall extends Enum {
    readonly isWithdraw: boolean;
    readonly asWithdraw: {
      readonly address: H160;
      readonly value: u128;
    } & Struct;
    readonly isCall: boolean;
    readonly asCall: {
      readonly source: H160;
      readonly target: H160;
      readonly input: Bytes;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly isCreate: boolean;
    readonly asCreate: {
      readonly source: H160;
      readonly init: Bytes;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly isCreate2: boolean;
    readonly asCreate2: {
      readonly source: H160;
      readonly init: Bytes;
      readonly salt: H256;
      readonly value: U256;
      readonly gasLimit: u64;
      readonly maxFeePerGas: U256;
      readonly maxPriorityFeePerGas: Option<U256>;
      readonly nonce: Option<U256>;
      readonly accessList: Vec<ITuple<[H160, Vec<H256>]>>;
    } & Struct;
    readonly type: 'Withdraw' | 'Call' | 'Create' | 'Create2';
  }

  /** @name PalletEthereumCall (389) */
  interface PalletEthereumCall extends Enum {
    readonly isTransact: boolean;
    readonly asTransact: {
      readonly transaction: EthereumTransactionTransactionV2;
    } & Struct;
    readonly type: 'Transact';
  }

  /** @name EthereumTransactionTransactionV2 (390) */
  interface EthereumTransactionTransactionV2 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumTransactionLegacyTransaction;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumTransactionEip2930Transaction;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumTransactionEip1559Transaction;
    readonly type: 'Legacy' | 'Eip2930' | 'Eip1559';
  }

  /** @name EthereumTransactionLegacyTransaction (391) */
  interface EthereumTransactionLegacyTransaction extends Struct {
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly signature: EthereumTransactionTransactionSignature;
  }

  /** @name EthereumTransactionTransactionAction (392) */
  interface EthereumTransactionTransactionAction extends Enum {
    readonly isCall: boolean;
    readonly asCall: H160;
    readonly isCreate: boolean;
    readonly type: 'Call' | 'Create';
  }

  /** @name EthereumTransactionTransactionSignature (393) */
  interface EthereumTransactionTransactionSignature extends Struct {
    readonly v: u64;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name EthereumTransactionEip2930Transaction (395) */
  interface EthereumTransactionEip2930Transaction extends Struct {
    readonly chainId: u64;
    readonly nonce: U256;
    readonly gasPrice: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Vec<EthereumTransactionAccessListItem>;
    readonly oddYParity: bool;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name EthereumTransactionAccessListItem (397) */
  interface EthereumTransactionAccessListItem extends Struct {
    readonly address: H160;
    readonly storageKeys: Vec<H256>;
  }

  /** @name EthereumTransactionEip1559Transaction (398) */
  interface EthereumTransactionEip1559Transaction extends Struct {
    readonly chainId: u64;
    readonly nonce: U256;
    readonly maxPriorityFeePerGas: U256;
    readonly maxFeePerGas: U256;
    readonly gasLimit: U256;
    readonly action: EthereumTransactionTransactionAction;
    readonly value: U256;
    readonly input: Bytes;
    readonly accessList: Vec<EthereumTransactionAccessListItem>;
    readonly oddYParity: bool;
    readonly r: H256;
    readonly s: H256;
  }

  /** @name PalletAccountFixCall (399) */
  interface PalletAccountFixCall extends Enum {
    readonly isUpgradeAccounts: boolean;
    readonly asUpgradeAccounts: {
      readonly who: Vec<AccountId32>;
    } & Struct;
    readonly isSetBalance: boolean;
    readonly asSetBalance: {
      readonly who: MultiAddress;
      readonly addFree: Compact<u128>;
      readonly addReserved: Compact<u128>;
    } & Struct;
    readonly type: 'UpgradeAccounts' | 'SetBalance';
  }

  /** @name PalletSudoCall (400) */
  interface PalletSudoCall extends Enum {
    readonly isSudo: boolean;
    readonly asSudo: {
      readonly call: Call;
    } & Struct;
    readonly isSudoUncheckedWeight: boolean;
    readonly asSudoUncheckedWeight: {
      readonly call: Call;
      readonly weight: SpWeightsWeightV2Weight;
    } & Struct;
    readonly isSetKey: boolean;
    readonly asSetKey: {
      readonly new_: MultiAddress;
    } & Struct;
    readonly isSudoAs: boolean;
    readonly asSudoAs: {
      readonly who: MultiAddress;
      readonly call: Call;
    } & Struct;
    readonly type: 'Sudo' | 'SudoUncheckedWeight' | 'SetKey' | 'SudoAs';
  }

  /** @name PalletSchedulerError (403) */
  interface PalletSchedulerError extends Enum {
    readonly isFailedToSchedule: boolean;
    readonly isNotFound: boolean;
    readonly isTargetBlockNumberInPast: boolean;
    readonly isRescheduleNoChange: boolean;
    readonly isNamed: boolean;
    readonly type: 'FailedToSchedule' | 'NotFound' | 'TargetBlockNumberInPast' | 'RescheduleNoChange' | 'Named';
  }

  /** @name PalletUtilityError (404) */
  interface PalletUtilityError extends Enum {
    readonly isTooManyCalls: boolean;
    readonly type: 'TooManyCalls';
  }

  /** @name PalletMultisigMultisig (406) */
  interface PalletMultisigMultisig extends Struct {
    readonly when: PalletMultisigTimepoint;
    readonly deposit: u128;
    readonly depositor: AccountId32;
    readonly approvals: Vec<AccountId32>;
  }

  /** @name PalletMultisigError (408) */
  interface PalletMultisigError extends Enum {
    readonly isMinimumThreshold: boolean;
    readonly isAlreadyApproved: boolean;
    readonly isNoApprovalsNeeded: boolean;
    readonly isTooFewSignatories: boolean;
    readonly isTooManySignatories: boolean;
    readonly isSignatoriesOutOfOrder: boolean;
    readonly isSenderInSignatories: boolean;
    readonly isNotFound: boolean;
    readonly isNotOwner: boolean;
    readonly isNoTimepoint: boolean;
    readonly isWrongTimepoint: boolean;
    readonly isUnexpectedTimepoint: boolean;
    readonly isMaxWeightTooLow: boolean;
    readonly isAlreadyStored: boolean;
    readonly type: 'MinimumThreshold' | 'AlreadyApproved' | 'NoApprovalsNeeded' | 'TooFewSignatories' | 'TooManySignatories' | 'SignatoriesOutOfOrder' | 'SenderInSignatories' | 'NotFound' | 'NotOwner' | 'NoTimepoint' | 'WrongTimepoint' | 'UnexpectedTimepoint' | 'MaxWeightTooLow' | 'AlreadyStored';
  }

  /** @name PalletProxyProxyDefinition (411) */
  interface PalletProxyProxyDefinition extends Struct {
    readonly delegate: AccountId32;
    readonly proxyType: RococoParachainRuntimeProxyType;
    readonly delay: u32;
  }

  /** @name PalletProxyAnnouncement (415) */
  interface PalletProxyAnnouncement extends Struct {
    readonly real: AccountId32;
    readonly callHash: H256;
    readonly height: u32;
  }

  /** @name PalletProxyError (417) */
  interface PalletProxyError extends Enum {
    readonly isTooMany: boolean;
    readonly isNotFound: boolean;
    readonly isNotProxy: boolean;
    readonly isUnproxyable: boolean;
    readonly isDuplicate: boolean;
    readonly isNoPermission: boolean;
    readonly isUnannounced: boolean;
    readonly isNoSelfProxy: boolean;
    readonly type: 'TooMany' | 'NotFound' | 'NotProxy' | 'Unproxyable' | 'Duplicate' | 'NoPermission' | 'Unannounced' | 'NoSelfProxy';
  }

  /** @name PalletPreimageRequestStatus (418) */
  interface PalletPreimageRequestStatus extends Enum {
    readonly isUnrequested: boolean;
    readonly asUnrequested: {
      readonly deposit: ITuple<[AccountId32, u128]>;
      readonly len: u32;
    } & Struct;
    readonly isRequested: boolean;
    readonly asRequested: {
      readonly deposit: Option<ITuple<[AccountId32, u128]>>;
      readonly count: u32;
      readonly len: Option<u32>;
    } & Struct;
    readonly type: 'Unrequested' | 'Requested';
  }

  /** @name PalletPreimageError (423) */
  interface PalletPreimageError extends Enum {
    readonly isTooBig: boolean;
    readonly isAlreadyNoted: boolean;
    readonly isNotAuthorized: boolean;
    readonly isNotNoted: boolean;
    readonly isRequested: boolean;
    readonly isNotRequested: boolean;
    readonly type: 'TooBig' | 'AlreadyNoted' | 'NotAuthorized' | 'NotNoted' | 'Requested' | 'NotRequested';
  }

  /** @name PalletBalancesBalanceLock (425) */
  interface PalletBalancesBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
    readonly reasons: PalletBalancesReasons;
  }

  /** @name PalletBalancesReasons (426) */
  interface PalletBalancesReasons extends Enum {
    readonly isFee: boolean;
    readonly isMisc: boolean;
    readonly isAll: boolean;
    readonly type: 'Fee' | 'Misc' | 'All';
  }

  /** @name PalletBalancesReserveData (429) */
  interface PalletBalancesReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name PalletBalancesIdAmount (432) */
  interface PalletBalancesIdAmount extends Struct {
    readonly id: Null;
    readonly amount: u128;
  }

  /** @name PalletBalancesError (434) */
  interface PalletBalancesError extends Enum {
    readonly isVestingBalance: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isExpendability: boolean;
    readonly isExistingVestingSchedule: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly isTooManyHolds: boolean;
    readonly isTooManyFreezes: boolean;
    readonly type: 'VestingBalance' | 'LiquidityRestrictions' | 'InsufficientBalance' | 'ExistentialDeposit' | 'Expendability' | 'ExistingVestingSchedule' | 'DeadAccount' | 'TooManyReserves' | 'TooManyHolds' | 'TooManyFreezes';
  }

  /** @name PalletVestingReleases (437) */
  interface PalletVestingReleases extends Enum {
    readonly isV0: boolean;
    readonly isV1: boolean;
    readonly type: 'V0' | 'V1';
  }

  /** @name PalletVestingError (438) */
  interface PalletVestingError extends Enum {
    readonly isNotVesting: boolean;
    readonly isAtMaxVestingSchedules: boolean;
    readonly isAmountLow: boolean;
    readonly isScheduleIndexOutOfBounds: boolean;
    readonly isInvalidScheduleParams: boolean;
    readonly type: 'NotVesting' | 'AtMaxVestingSchedules' | 'AmountLow' | 'ScheduleIndexOutOfBounds' | 'InvalidScheduleParams';
  }

  /** @name PalletTransactionPaymentReleases (440) */
  interface PalletTransactionPaymentReleases extends Enum {
    readonly isV1Ancient: boolean;
    readonly isV2: boolean;
    readonly type: 'V1Ancient' | 'V2';
  }

  /** @name PalletTreasuryProposal (441) */
  interface PalletTreasuryProposal extends Struct {
    readonly proposer: AccountId32;
    readonly value: u128;
    readonly beneficiary: AccountId32;
    readonly bond: u128;
  }

  /** @name FrameSupportPalletId (446) */
  interface FrameSupportPalletId extends U8aFixed {}

  /** @name PalletTreasuryError (447) */
  interface PalletTreasuryError extends Enum {
    readonly isInsufficientProposersBalance: boolean;
    readonly isInvalidIndex: boolean;
    readonly isTooManyApprovals: boolean;
    readonly isInsufficientPermission: boolean;
    readonly isProposalNotApproved: boolean;
    readonly type: 'InsufficientProposersBalance' | 'InvalidIndex' | 'TooManyApprovals' | 'InsufficientPermission' | 'ProposalNotApproved';
  }

  /** @name PalletDemocracyReferendumInfo (452) */
  interface PalletDemocracyReferendumInfo extends Enum {
    readonly isOngoing: boolean;
    readonly asOngoing: PalletDemocracyReferendumStatus;
    readonly isFinished: boolean;
    readonly asFinished: {
      readonly approved: bool;
      readonly end: u32;
    } & Struct;
    readonly type: 'Ongoing' | 'Finished';
  }

  /** @name PalletDemocracyReferendumStatus (453) */
  interface PalletDemocracyReferendumStatus extends Struct {
    readonly end: u32;
    readonly proposal: FrameSupportPreimagesBounded;
    readonly threshold: PalletDemocracyVoteThreshold;
    readonly delay: u32;
    readonly tally: PalletDemocracyTally;
  }

  /** @name PalletDemocracyTally (454) */
  interface PalletDemocracyTally extends Struct {
    readonly ayes: u128;
    readonly nays: u128;
    readonly turnout: u128;
  }

  /** @name PalletDemocracyVoteVoting (455) */
  interface PalletDemocracyVoteVoting extends Enum {
    readonly isDirect: boolean;
    readonly asDirect: {
      readonly votes: Vec<ITuple<[u32, PalletDemocracyVoteAccountVote]>>;
      readonly delegations: PalletDemocracyDelegations;
      readonly prior: PalletDemocracyVotePriorLock;
    } & Struct;
    readonly isDelegating: boolean;
    readonly asDelegating: {
      readonly balance: u128;
      readonly target: AccountId32;
      readonly conviction: PalletDemocracyConviction;
      readonly delegations: PalletDemocracyDelegations;
      readonly prior: PalletDemocracyVotePriorLock;
    } & Struct;
    readonly type: 'Direct' | 'Delegating';
  }

  /** @name PalletDemocracyDelegations (459) */
  interface PalletDemocracyDelegations extends Struct {
    readonly votes: u128;
    readonly capital: u128;
  }

  /** @name PalletDemocracyVotePriorLock (460) */
  interface PalletDemocracyVotePriorLock extends ITuple<[u32, u128]> {}

  /** @name PalletDemocracyError (463) */
  interface PalletDemocracyError extends Enum {
    readonly isValueLow: boolean;
    readonly isProposalMissing: boolean;
    readonly isAlreadyCanceled: boolean;
    readonly isDuplicateProposal: boolean;
    readonly isProposalBlacklisted: boolean;
    readonly isNotSimpleMajority: boolean;
    readonly isInvalidHash: boolean;
    readonly isNoProposal: boolean;
    readonly isAlreadyVetoed: boolean;
    readonly isReferendumInvalid: boolean;
    readonly isNoneWaiting: boolean;
    readonly isNotVoter: boolean;
    readonly isNoPermission: boolean;
    readonly isAlreadyDelegating: boolean;
    readonly isInsufficientFunds: boolean;
    readonly isNotDelegating: boolean;
    readonly isVotesExist: boolean;
    readonly isInstantNotAllowed: boolean;
    readonly isNonsense: boolean;
    readonly isWrongUpperBound: boolean;
    readonly isMaxVotesReached: boolean;
    readonly isTooMany: boolean;
    readonly isVotingPeriodLow: boolean;
    readonly isPreimageNotExist: boolean;
    readonly type: 'ValueLow' | 'ProposalMissing' | 'AlreadyCanceled' | 'DuplicateProposal' | 'ProposalBlacklisted' | 'NotSimpleMajority' | 'InvalidHash' | 'NoProposal' | 'AlreadyVetoed' | 'ReferendumInvalid' | 'NoneWaiting' | 'NotVoter' | 'NoPermission' | 'AlreadyDelegating' | 'InsufficientFunds' | 'NotDelegating' | 'VotesExist' | 'InstantNotAllowed' | 'Nonsense' | 'WrongUpperBound' | 'MaxVotesReached' | 'TooMany' | 'VotingPeriodLow' | 'PreimageNotExist';
  }

  /** @name PalletCollectiveVotes (465) */
  interface PalletCollectiveVotes extends Struct {
    readonly index: u32;
    readonly threshold: u32;
    readonly ayes: Vec<AccountId32>;
    readonly nays: Vec<AccountId32>;
    readonly end: u32;
  }

  /** @name PalletCollectiveError (466) */
  interface PalletCollectiveError extends Enum {
    readonly isNotMember: boolean;
    readonly isDuplicateProposal: boolean;
    readonly isProposalMissing: boolean;
    readonly isWrongIndex: boolean;
    readonly isDuplicateVote: boolean;
    readonly isAlreadyInitialized: boolean;
    readonly isTooEarly: boolean;
    readonly isTooManyProposals: boolean;
    readonly isWrongProposalWeight: boolean;
    readonly isWrongProposalLength: boolean;
    readonly type: 'NotMember' | 'DuplicateProposal' | 'ProposalMissing' | 'WrongIndex' | 'DuplicateVote' | 'AlreadyInitialized' | 'TooEarly' | 'TooManyProposals' | 'WrongProposalWeight' | 'WrongProposalLength';
  }

  /** @name PalletMembershipError (468) */
  interface PalletMembershipError extends Enum {
    readonly isAlreadyMember: boolean;
    readonly isNotMember: boolean;
    readonly isTooManyMembers: boolean;
    readonly type: 'AlreadyMember' | 'NotMember' | 'TooManyMembers';
  }

  /** @name PalletBountiesBounty (471) */
  interface PalletBountiesBounty extends Struct {
    readonly proposer: AccountId32;
    readonly value: u128;
    readonly fee: u128;
    readonly curatorDeposit: u128;
    readonly bond: u128;
    readonly status: PalletBountiesBountyStatus;
  }

  /** @name PalletBountiesBountyStatus (472) */
  interface PalletBountiesBountyStatus extends Enum {
    readonly isProposed: boolean;
    readonly isApproved: boolean;
    readonly isFunded: boolean;
    readonly isCuratorProposed: boolean;
    readonly asCuratorProposed: {
      readonly curator: AccountId32;
    } & Struct;
    readonly isActive: boolean;
    readonly asActive: {
      readonly curator: AccountId32;
      readonly updateDue: u32;
    } & Struct;
    readonly isPendingPayout: boolean;
    readonly asPendingPayout: {
      readonly curator: AccountId32;
      readonly beneficiary: AccountId32;
      readonly unlockAt: u32;
    } & Struct;
    readonly type: 'Proposed' | 'Approved' | 'Funded' | 'CuratorProposed' | 'Active' | 'PendingPayout';
  }

  /** @name PalletBountiesError (474) */
  interface PalletBountiesError extends Enum {
    readonly isInsufficientProposersBalance: boolean;
    readonly isInvalidIndex: boolean;
    readonly isReasonTooBig: boolean;
    readonly isUnexpectedStatus: boolean;
    readonly isRequireCurator: boolean;
    readonly isInvalidValue: boolean;
    readonly isInvalidFee: boolean;
    readonly isPendingPayout: boolean;
    readonly isPremature: boolean;
    readonly isHasActiveChildBounty: boolean;
    readonly isTooManyQueued: boolean;
    readonly type: 'InsufficientProposersBalance' | 'InvalidIndex' | 'ReasonTooBig' | 'UnexpectedStatus' | 'RequireCurator' | 'InvalidValue' | 'InvalidFee' | 'PendingPayout' | 'Premature' | 'HasActiveChildBounty' | 'TooManyQueued';
  }

  /** @name PalletTipsOpenTip (475) */
  interface PalletTipsOpenTip extends Struct {
    readonly reason: H256;
    readonly who: AccountId32;
    readonly finder: AccountId32;
    readonly deposit: u128;
    readonly closes: Option<u32>;
    readonly tips: Vec<ITuple<[AccountId32, u128]>>;
    readonly findersFee: bool;
  }

  /** @name PalletTipsError (477) */
  interface PalletTipsError extends Enum {
    readonly isReasonTooBig: boolean;
    readonly isAlreadyKnown: boolean;
    readonly isUnknownTip: boolean;
    readonly isNotFinder: boolean;
    readonly isStillOpen: boolean;
    readonly isPremature: boolean;
    readonly type: 'ReasonTooBig' | 'AlreadyKnown' | 'UnknownTip' | 'NotFinder' | 'StillOpen' | 'Premature';
  }

  /** @name PalletIdentityRegistration (478) */
  interface PalletIdentityRegistration extends Struct {
    readonly judgements: Vec<ITuple<[u32, PalletIdentityJudgement]>>;
    readonly deposit: u128;
    readonly info: PalletIdentityIdentityInfo;
  }

  /** @name PalletIdentityRegistrarInfo (485) */
  interface PalletIdentityRegistrarInfo extends Struct {
    readonly account: AccountId32;
    readonly fee: u128;
    readonly fields: PalletIdentityBitFlags;
  }

  /** @name PalletIdentityError (487) */
  interface PalletIdentityError extends Enum {
    readonly isTooManySubAccounts: boolean;
    readonly isNotFound: boolean;
    readonly isNotNamed: boolean;
    readonly isEmptyIndex: boolean;
    readonly isFeeChanged: boolean;
    readonly isNoIdentity: boolean;
    readonly isStickyJudgement: boolean;
    readonly isJudgementGiven: boolean;
    readonly isInvalidJudgement: boolean;
    readonly isInvalidIndex: boolean;
    readonly isInvalidTarget: boolean;
    readonly isTooManyFields: boolean;
    readonly isTooManyRegistrars: boolean;
    readonly isAlreadyClaimed: boolean;
    readonly isNotSub: boolean;
    readonly isNotOwned: boolean;
    readonly isJudgementForDifferentIdentity: boolean;
    readonly isJudgementPaymentFailed: boolean;
    readonly type: 'TooManySubAccounts' | 'NotFound' | 'NotNamed' | 'EmptyIndex' | 'FeeChanged' | 'NoIdentity' | 'StickyJudgement' | 'JudgementGiven' | 'InvalidJudgement' | 'InvalidIndex' | 'InvalidTarget' | 'TooManyFields' | 'TooManyRegistrars' | 'AlreadyClaimed' | 'NotSub' | 'NotOwned' | 'JudgementForDifferentIdentity' | 'JudgementPaymentFailed';
  }

  /** @name PolkadotPrimitivesV4UpgradeRestriction (489) */
  interface PolkadotPrimitivesV4UpgradeRestriction extends Enum {
    readonly isPresent: boolean;
    readonly type: 'Present';
  }

  /** @name CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot (490) */
  interface CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot extends Struct {
    readonly dmqMqcHead: H256;
    readonly relayDispatchQueueSize: ITuple<[u32, u32]>;
    readonly ingressChannels: Vec<ITuple<[u32, PolkadotPrimitivesV4AbridgedHrmpChannel]>>;
    readonly egressChannels: Vec<ITuple<[u32, PolkadotPrimitivesV4AbridgedHrmpChannel]>>;
  }

  /** @name PolkadotPrimitivesV4AbridgedHrmpChannel (493) */
  interface PolkadotPrimitivesV4AbridgedHrmpChannel extends Struct {
    readonly maxCapacity: u32;
    readonly maxTotalSize: u32;
    readonly maxMessageSize: u32;
    readonly msgCount: u32;
    readonly totalSize: u32;
    readonly mqcHead: Option<H256>;
  }

  /** @name PolkadotPrimitivesV4AbridgedHostConfiguration (494) */
  interface PolkadotPrimitivesV4AbridgedHostConfiguration extends Struct {
    readonly maxCodeSize: u32;
    readonly maxHeadDataSize: u32;
    readonly maxUpwardQueueCount: u32;
    readonly maxUpwardQueueSize: u32;
    readonly maxUpwardMessageSize: u32;
    readonly maxUpwardMessageNumPerCandidate: u32;
    readonly hrmpMaxMessageNumPerCandidate: u32;
    readonly validationUpgradeCooldown: u32;
    readonly validationUpgradeDelay: u32;
  }

  /** @name PolkadotCorePrimitivesOutboundHrmpMessage (500) */
  interface PolkadotCorePrimitivesOutboundHrmpMessage extends Struct {
    readonly recipient: u32;
    readonly data: Bytes;
  }

  /** @name CumulusPalletParachainSystemCodeUpgradeAuthorization (501) */
  interface CumulusPalletParachainSystemCodeUpgradeAuthorization extends Struct {
    readonly codeHash: H256;
    readonly checkVersion: bool;
  }

  /** @name CumulusPalletParachainSystemError (502) */
  interface CumulusPalletParachainSystemError extends Enum {
    readonly isOverlappingUpgrades: boolean;
    readonly isProhibitedByPolkadot: boolean;
    readonly isTooBig: boolean;
    readonly isValidationDataNotAvailable: boolean;
    readonly isHostConfigurationNotAvailable: boolean;
    readonly isNotScheduled: boolean;
    readonly isNothingAuthorized: boolean;
    readonly isUnauthorized: boolean;
    readonly type: 'OverlappingUpgrades' | 'ProhibitedByPolkadot' | 'TooBig' | 'ValidationDataNotAvailable' | 'HostConfigurationNotAvailable' | 'NotScheduled' | 'NothingAuthorized' | 'Unauthorized';
  }

  /** @name SpCoreCryptoKeyTypeId (506) */
  interface SpCoreCryptoKeyTypeId extends U8aFixed {}

  /** @name PalletSessionError (507) */
  interface PalletSessionError extends Enum {
    readonly isInvalidProof: boolean;
    readonly isNoAssociatedValidatorId: boolean;
    readonly isDuplicatedKey: boolean;
    readonly isNoKeys: boolean;
    readonly isNoAccount: boolean;
    readonly type: 'InvalidProof' | 'NoAssociatedValidatorId' | 'DuplicatedKey' | 'NoKeys' | 'NoAccount';
  }

  /** @name PalletParachainStakingParachainBondConfig (511) */
  interface PalletParachainStakingParachainBondConfig extends Struct {
    readonly account: AccountId32;
    readonly percent: Percent;
  }

  /** @name PalletParachainStakingRoundInfo (512) */
  interface PalletParachainStakingRoundInfo extends Struct {
    readonly current: u32;
    readonly first: u32;
    readonly length: u32;
  }

  /** @name PalletParachainStakingDelegator (513) */
  interface PalletParachainStakingDelegator extends Struct {
    readonly id: AccountId32;
    readonly delegations: PalletParachainStakingSetOrderedSet;
    readonly total: u128;
    readonly lessTotal: u128;
    readonly status: PalletParachainStakingDelegatorStatus;
  }

  /** @name PalletParachainStakingSetOrderedSet (514) */
  interface PalletParachainStakingSetOrderedSet extends Vec<PalletParachainStakingBond> {}

  /** @name PalletParachainStakingBond (515) */
  interface PalletParachainStakingBond extends Struct {
    readonly owner: AccountId32;
    readonly amount: u128;
  }

  /** @name PalletParachainStakingDelegatorStatus (517) */
  interface PalletParachainStakingDelegatorStatus extends Enum {
    readonly isActive: boolean;
    readonly type: 'Active';
  }

  /** @name PalletParachainStakingCandidateMetadata (518) */
  interface PalletParachainStakingCandidateMetadata extends Struct {
    readonly bond: u128;
    readonly delegationCount: u32;
    readonly totalCounted: u128;
    readonly lowestTopDelegationAmount: u128;
    readonly highestBottomDelegationAmount: u128;
    readonly lowestBottomDelegationAmount: u128;
    readonly topCapacity: PalletParachainStakingCapacityStatus;
    readonly bottomCapacity: PalletParachainStakingCapacityStatus;
    readonly request: Option<PalletParachainStakingCandidateBondLessRequest>;
    readonly status: PalletParachainStakingCollatorStatus;
  }

  /** @name PalletParachainStakingCapacityStatus (519) */
  interface PalletParachainStakingCapacityStatus extends Enum {
    readonly isFull: boolean;
    readonly isEmpty: boolean;
    readonly isPartial: boolean;
    readonly type: 'Full' | 'Empty' | 'Partial';
  }

  /** @name PalletParachainStakingCandidateBondLessRequest (521) */
  interface PalletParachainStakingCandidateBondLessRequest extends Struct {
    readonly amount: u128;
    readonly whenExecutable: u32;
  }

  /** @name PalletParachainStakingCollatorStatus (522) */
  interface PalletParachainStakingCollatorStatus extends Enum {
    readonly isActive: boolean;
    readonly isIdle: boolean;
    readonly isLeaving: boolean;
    readonly asLeaving: u32;
    readonly type: 'Active' | 'Idle' | 'Leaving';
  }

  /** @name PalletParachainStakingDelegationRequestsScheduledRequest (524) */
  interface PalletParachainStakingDelegationRequestsScheduledRequest extends Struct {
    readonly delegator: AccountId32;
    readonly whenExecutable: u32;
    readonly action: PalletParachainStakingDelegationRequestsDelegationAction;
  }

  /** @name PalletParachainStakingAutoCompoundAutoCompoundConfig (526) */
  interface PalletParachainStakingAutoCompoundAutoCompoundConfig extends Struct {
    readonly delegator: AccountId32;
    readonly value: Percent;
  }

  /** @name PalletParachainStakingDelegations (527) */
  interface PalletParachainStakingDelegations extends Struct {
    readonly delegations: Vec<PalletParachainStakingBond>;
    readonly total: u128;
  }

  /** @name PalletParachainStakingCollatorSnapshot (529) */
  interface PalletParachainStakingCollatorSnapshot extends Struct {
    readonly bond: u128;
    readonly delegations: Vec<PalletParachainStakingBondWithAutoCompound>;
    readonly total: u128;
  }

  /** @name PalletParachainStakingBondWithAutoCompound (531) */
  interface PalletParachainStakingBondWithAutoCompound extends Struct {
    readonly owner: AccountId32;
    readonly amount: u128;
    readonly autoCompound: Percent;
  }

  /** @name PalletParachainStakingDelayedPayout (532) */
  interface PalletParachainStakingDelayedPayout extends Struct {
    readonly roundIssuance: u128;
    readonly totalStakingReward: u128;
    readonly collatorCommission: Perbill;
  }

  /** @name PalletParachainStakingInflationInflationInfo (533) */
  interface PalletParachainStakingInflationInflationInfo extends Struct {
    readonly expect: {
      readonly min: u128;
      readonly ideal: u128;
      readonly max: u128;
    } & Struct;
    readonly annual: {
      readonly min: Perbill;
      readonly ideal: Perbill;
      readonly max: Perbill;
    } & Struct;
    readonly round: {
      readonly min: Perbill;
      readonly ideal: Perbill;
      readonly max: Perbill;
    } & Struct;
  }

  /** @name PalletParachainStakingError (534) */
  interface PalletParachainStakingError extends Enum {
    readonly isDelegatorDNE: boolean;
    readonly isDelegatorDNEinTopNorBottom: boolean;
    readonly isDelegatorDNEInDelegatorSet: boolean;
    readonly isCandidateDNE: boolean;
    readonly isDelegationDNE: boolean;
    readonly isDelegatorExists: boolean;
    readonly isCandidateExists: boolean;
    readonly isCandidateBondBelowMin: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isDelegatorBondBelowMin: boolean;
    readonly isDelegationBelowMin: boolean;
    readonly isAlreadyOffline: boolean;
    readonly isAlreadyActive: boolean;
    readonly isDelegatorAlreadyLeaving: boolean;
    readonly isDelegatorNotLeaving: boolean;
    readonly isDelegatorCannotLeaveYet: boolean;
    readonly isCannotDelegateIfLeaving: boolean;
    readonly isCandidateAlreadyLeaving: boolean;
    readonly isCandidateNotLeaving: boolean;
    readonly isCandidateCannotLeaveYet: boolean;
    readonly isCannotGoOnlineIfLeaving: boolean;
    readonly isExceedMaxDelegationsPerDelegator: boolean;
    readonly isAlreadyDelegatedCandidate: boolean;
    readonly isInvalidSchedule: boolean;
    readonly isCannotSetBelowMin: boolean;
    readonly isRoundLengthMustBeGreaterThanTotalSelectedCollators: boolean;
    readonly isNoWritingSameValue: boolean;
    readonly isTooLowCandidateCountWeightHintCancelLeaveCandidates: boolean;
    readonly isTooLowCandidateDelegationCountToLeaveCandidates: boolean;
    readonly isPendingCandidateRequestsDNE: boolean;
    readonly isPendingCandidateRequestAlreadyExists: boolean;
    readonly isPendingCandidateRequestNotDueYet: boolean;
    readonly isPendingDelegationRequestDNE: boolean;
    readonly isPendingDelegationRequestAlreadyExists: boolean;
    readonly isPendingDelegationRequestNotDueYet: boolean;
    readonly isCannotDelegateLessThanOrEqualToLowestBottomWhenFull: boolean;
    readonly isPendingDelegationRevoke: boolean;
    readonly isCandidateUnauthorized: boolean;
    readonly type: 'DelegatorDNE' | 'DelegatorDNEinTopNorBottom' | 'DelegatorDNEInDelegatorSet' | 'CandidateDNE' | 'DelegationDNE' | 'DelegatorExists' | 'CandidateExists' | 'CandidateBondBelowMin' | 'InsufficientBalance' | 'DelegatorBondBelowMin' | 'DelegationBelowMin' | 'AlreadyOffline' | 'AlreadyActive' | 'DelegatorAlreadyLeaving' | 'DelegatorNotLeaving' | 'DelegatorCannotLeaveYet' | 'CannotDelegateIfLeaving' | 'CandidateAlreadyLeaving' | 'CandidateNotLeaving' | 'CandidateCannotLeaveYet' | 'CannotGoOnlineIfLeaving' | 'ExceedMaxDelegationsPerDelegator' | 'AlreadyDelegatedCandidate' | 'InvalidSchedule' | 'CannotSetBelowMin' | 'RoundLengthMustBeGreaterThanTotalSelectedCollators' | 'NoWritingSameValue' | 'TooLowCandidateCountWeightHintCancelLeaveCandidates' | 'TooLowCandidateDelegationCountToLeaveCandidates' | 'PendingCandidateRequestsDNE' | 'PendingCandidateRequestAlreadyExists' | 'PendingCandidateRequestNotDueYet' | 'PendingDelegationRequestDNE' | 'PendingDelegationRequestAlreadyExists' | 'PendingDelegationRequestNotDueYet' | 'CannotDelegateLessThanOrEqualToLowestBottomWhenFull' | 'PendingDelegationRevoke' | 'CandidateUnauthorized';
  }

  /** @name CumulusPalletXcmpQueueInboundChannelDetails (536) */
  interface CumulusPalletXcmpQueueInboundChannelDetails extends Struct {
    readonly sender: u32;
    readonly state: CumulusPalletXcmpQueueInboundState;
    readonly messageMetadata: Vec<ITuple<[u32, PolkadotParachainPrimitivesXcmpMessageFormat]>>;
  }

  /** @name CumulusPalletXcmpQueueInboundState (537) */
  interface CumulusPalletXcmpQueueInboundState extends Enum {
    readonly isOk: boolean;
    readonly isSuspended: boolean;
    readonly type: 'Ok' | 'Suspended';
  }

  /** @name PolkadotParachainPrimitivesXcmpMessageFormat (540) */
  interface PolkadotParachainPrimitivesXcmpMessageFormat extends Enum {
    readonly isConcatenatedVersionedXcm: boolean;
    readonly isConcatenatedEncodedBlob: boolean;
    readonly isSignals: boolean;
    readonly type: 'ConcatenatedVersionedXcm' | 'ConcatenatedEncodedBlob' | 'Signals';
  }

  /** @name CumulusPalletXcmpQueueOutboundChannelDetails (543) */
  interface CumulusPalletXcmpQueueOutboundChannelDetails extends Struct {
    readonly recipient: u32;
    readonly state: CumulusPalletXcmpQueueOutboundState;
    readonly signalsExist: bool;
    readonly firstIndex: u16;
    readonly lastIndex: u16;
  }

  /** @name CumulusPalletXcmpQueueOutboundState (544) */
  interface CumulusPalletXcmpQueueOutboundState extends Enum {
    readonly isOk: boolean;
    readonly isSuspended: boolean;
    readonly type: 'Ok' | 'Suspended';
  }

  /** @name CumulusPalletXcmpQueueQueueConfigData (546) */
  interface CumulusPalletXcmpQueueQueueConfigData extends Struct {
    readonly suspendThreshold: u32;
    readonly dropThreshold: u32;
    readonly resumeThreshold: u32;
    readonly thresholdWeight: SpWeightsWeightV2Weight;
    readonly weightRestrictDecay: SpWeightsWeightV2Weight;
    readonly xcmpMaxIndividualWeight: SpWeightsWeightV2Weight;
  }

  /** @name CumulusPalletXcmpQueueError (548) */
  interface CumulusPalletXcmpQueueError extends Enum {
    readonly isFailedToSend: boolean;
    readonly isBadXcmOrigin: boolean;
    readonly isBadXcm: boolean;
    readonly isBadOverweightIndex: boolean;
    readonly isWeightOverLimit: boolean;
    readonly type: 'FailedToSend' | 'BadXcmOrigin' | 'BadXcm' | 'BadOverweightIndex' | 'WeightOverLimit';
  }

  /** @name PalletXcmQueryStatus (549) */
  interface PalletXcmQueryStatus extends Enum {
    readonly isPending: boolean;
    readonly asPending: {
      readonly responder: XcmVersionedMultiLocation;
      readonly maybeMatchQuerier: Option<XcmVersionedMultiLocation>;
      readonly maybeNotify: Option<ITuple<[u8, u8]>>;
      readonly timeout: u32;
    } & Struct;
    readonly isVersionNotifier: boolean;
    readonly asVersionNotifier: {
      readonly origin: XcmVersionedMultiLocation;
      readonly isActive: bool;
    } & Struct;
    readonly isReady: boolean;
    readonly asReady: {
      readonly response: XcmVersionedResponse;
      readonly at: u32;
    } & Struct;
    readonly type: 'Pending' | 'VersionNotifier' | 'Ready';
  }

  /** @name XcmVersionedResponse (553) */
  interface XcmVersionedResponse extends Enum {
    readonly isV2: boolean;
    readonly asV2: XcmV2Response;
    readonly isV3: boolean;
    readonly asV3: XcmV3Response;
    readonly type: 'V2' | 'V3';
  }

  /** @name PalletXcmVersionMigrationStage (559) */
  interface PalletXcmVersionMigrationStage extends Enum {
    readonly isMigrateSupportedVersion: boolean;
    readonly isMigrateVersionNotifiers: boolean;
    readonly isNotifyCurrentTargets: boolean;
    readonly asNotifyCurrentTargets: Option<Bytes>;
    readonly isMigrateAndNotifyOldTargets: boolean;
    readonly type: 'MigrateSupportedVersion' | 'MigrateVersionNotifiers' | 'NotifyCurrentTargets' | 'MigrateAndNotifyOldTargets';
  }

  /** @name XcmVersionedAssetId (561) */
  interface XcmVersionedAssetId extends Enum {
    readonly isV3: boolean;
    readonly asV3: XcmV3MultiassetAssetId;
    readonly type: 'V3';
  }

  /** @name PalletXcmRemoteLockedFungibleRecord (562) */
  interface PalletXcmRemoteLockedFungibleRecord extends Struct {
    readonly amount: u128;
    readonly owner: XcmVersionedMultiLocation;
    readonly locker: XcmVersionedMultiLocation;
    readonly users: u32;
  }

  /** @name PalletXcmError (566) */
  interface PalletXcmError extends Enum {
    readonly isUnreachable: boolean;
    readonly isSendFailure: boolean;
    readonly isFiltered: boolean;
    readonly isUnweighableMessage: boolean;
    readonly isDestinationNotInvertible: boolean;
    readonly isEmpty: boolean;
    readonly isCannotReanchor: boolean;
    readonly isTooManyAssets: boolean;
    readonly isInvalidOrigin: boolean;
    readonly isBadVersion: boolean;
    readonly isBadLocation: boolean;
    readonly isNoSubscription: boolean;
    readonly isAlreadySubscribed: boolean;
    readonly isInvalidAsset: boolean;
    readonly isLowBalance: boolean;
    readonly isTooManyLocks: boolean;
    readonly isAccountNotSovereign: boolean;
    readonly isFeesNotMet: boolean;
    readonly isLockNotFound: boolean;
    readonly isInUse: boolean;
    readonly type: 'Unreachable' | 'SendFailure' | 'Filtered' | 'UnweighableMessage' | 'DestinationNotInvertible' | 'Empty' | 'CannotReanchor' | 'TooManyAssets' | 'InvalidOrigin' | 'BadVersion' | 'BadLocation' | 'NoSubscription' | 'AlreadySubscribed' | 'InvalidAsset' | 'LowBalance' | 'TooManyLocks' | 'AccountNotSovereign' | 'FeesNotMet' | 'LockNotFound' | 'InUse';
  }

  /** @name CumulusPalletXcmError (567) */
  type CumulusPalletXcmError = Null;

  /** @name CumulusPalletDmpQueueConfigData (568) */
  interface CumulusPalletDmpQueueConfigData extends Struct {
    readonly maxIndividual: SpWeightsWeightV2Weight;
  }

  /** @name CumulusPalletDmpQueuePageIndexData (569) */
  interface CumulusPalletDmpQueuePageIndexData extends Struct {
    readonly beginUsed: u32;
    readonly endUsed: u32;
    readonly overweightCount: u64;
  }

  /** @name CumulusPalletDmpQueueError (572) */
  interface CumulusPalletDmpQueueError extends Enum {
    readonly isUnknown: boolean;
    readonly isOverLimit: boolean;
    readonly type: 'Unknown' | 'OverLimit';
  }

  /** @name OrmlXtokensModuleError (573) */
  interface OrmlXtokensModuleError extends Enum {
    readonly isAssetHasNoReserve: boolean;
    readonly isNotCrossChainTransfer: boolean;
    readonly isInvalidDest: boolean;
    readonly isNotCrossChainTransferableCurrency: boolean;
    readonly isUnweighableMessage: boolean;
    readonly isXcmExecutionFailed: boolean;
    readonly isCannotReanchor: boolean;
    readonly isInvalidAncestry: boolean;
    readonly isInvalidAsset: boolean;
    readonly isDestinationNotInvertible: boolean;
    readonly isBadVersion: boolean;
    readonly isDistinctReserveForAssetAndFee: boolean;
    readonly isZeroFee: boolean;
    readonly isZeroAmount: boolean;
    readonly isTooManyAssetsBeingSent: boolean;
    readonly isAssetIndexNonExistent: boolean;
    readonly isFeeNotEnough: boolean;
    readonly isNotSupportedMultiLocation: boolean;
    readonly isMinXcmFeeNotDefined: boolean;
    readonly type: 'AssetHasNoReserve' | 'NotCrossChainTransfer' | 'InvalidDest' | 'NotCrossChainTransferableCurrency' | 'UnweighableMessage' | 'XcmExecutionFailed' | 'CannotReanchor' | 'InvalidAncestry' | 'InvalidAsset' | 'DestinationNotInvertible' | 'BadVersion' | 'DistinctReserveForAssetAndFee' | 'ZeroFee' | 'ZeroAmount' | 'TooManyAssetsBeingSent' | 'AssetIndexNonExistent' | 'FeeNotEnough' | 'NotSupportedMultiLocation' | 'MinXcmFeeNotDefined';
  }

  /** @name OrmlTokensBalanceLock (575) */
  interface OrmlTokensBalanceLock extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name OrmlTokensAccountData (577) */
  interface OrmlTokensAccountData extends Struct {
    readonly free: u128;
    readonly reserved: u128;
    readonly frozen: u128;
  }

  /** @name OrmlTokensReserveData (579) */
  interface OrmlTokensReserveData extends Struct {
    readonly id: U8aFixed;
    readonly amount: u128;
  }

  /** @name OrmlTokensModuleError (581) */
  interface OrmlTokensModuleError extends Enum {
    readonly isBalanceTooLow: boolean;
    readonly isAmountIntoBalanceFailed: boolean;
    readonly isLiquidityRestrictions: boolean;
    readonly isMaxLocksExceeded: boolean;
    readonly isKeepAlive: boolean;
    readonly isExistentialDeposit: boolean;
    readonly isDeadAccount: boolean;
    readonly isTooManyReserves: boolean;
    readonly type: 'BalanceTooLow' | 'AmountIntoBalanceFailed' | 'LiquidityRestrictions' | 'MaxLocksExceeded' | 'KeepAlive' | 'ExistentialDeposit' | 'DeadAccount' | 'TooManyReserves';
  }

  /** @name PalletBridgeProposalVotes (584) */
  interface PalletBridgeProposalVotes extends Struct {
    readonly votesFor: Vec<AccountId32>;
    readonly votesAgainst: Vec<AccountId32>;
    readonly status: PalletBridgeProposalStatus;
    readonly expiry: u32;
  }

  /** @name PalletBridgeProposalStatus (585) */
  interface PalletBridgeProposalStatus extends Enum {
    readonly isInitiated: boolean;
    readonly isApproved: boolean;
    readonly isRejected: boolean;
    readonly type: 'Initiated' | 'Approved' | 'Rejected';
  }

  /** @name PalletBridgeBridgeEvent (587) */
  interface PalletBridgeBridgeEvent extends Enum {
    readonly isFungibleTransfer: boolean;
    readonly asFungibleTransfer: ITuple<[u8, u64, U8aFixed, u128, Bytes]>;
    readonly isNonFungibleTransfer: boolean;
    readonly asNonFungibleTransfer: ITuple<[u8, u64, U8aFixed, Bytes, Bytes, Bytes]>;
    readonly isGenericTransfer: boolean;
    readonly asGenericTransfer: ITuple<[u8, u64, U8aFixed, Bytes]>;
    readonly type: 'FungibleTransfer' | 'NonFungibleTransfer' | 'GenericTransfer';
  }

  /** @name PalletBridgeError (588) */
  interface PalletBridgeError extends Enum {
    readonly isThresholdNotSet: boolean;
    readonly isInvalidChainId: boolean;
    readonly isInvalidThreshold: boolean;
    readonly isChainNotWhitelisted: boolean;
    readonly isChainAlreadyWhitelisted: boolean;
    readonly isResourceDoesNotExist: boolean;
    readonly isRelayerAlreadyExists: boolean;
    readonly isRelayerInvalid: boolean;
    readonly isMustBeRelayer: boolean;
    readonly isRelayerAlreadyVoted: boolean;
    readonly isProposalAlreadyExists: boolean;
    readonly isProposalDoesNotExist: boolean;
    readonly isProposalNotComplete: boolean;
    readonly isProposalAlreadyComplete: boolean;
    readonly isProposalExpired: boolean;
    readonly isFeeTooExpensive: boolean;
    readonly isFeeDoesNotExist: boolean;
    readonly isInsufficientBalance: boolean;
    readonly isCannotPayAsFee: boolean;
    readonly isNonceOverFlow: boolean;
    readonly type: 'ThresholdNotSet' | 'InvalidChainId' | 'InvalidThreshold' | 'ChainNotWhitelisted' | 'ChainAlreadyWhitelisted' | 'ResourceDoesNotExist' | 'RelayerAlreadyExists' | 'RelayerInvalid' | 'MustBeRelayer' | 'RelayerAlreadyVoted' | 'ProposalAlreadyExists' | 'ProposalDoesNotExist' | 'ProposalNotComplete' | 'ProposalAlreadyComplete' | 'ProposalExpired' | 'FeeTooExpensive' | 'FeeDoesNotExist' | 'InsufficientBalance' | 'CannotPayAsFee' | 'NonceOverFlow';
  }

  /** @name PalletBridgeTransferError (590) */
  interface PalletBridgeTransferError extends Enum {
    readonly isInvalidCommand: boolean;
    readonly isInvalidResourceId: boolean;
    readonly isReachMaximumSupply: boolean;
    readonly isOverFlow: boolean;
    readonly type: 'InvalidCommand' | 'InvalidResourceId' | 'ReachMaximumSupply' | 'OverFlow';
  }

  /** @name PalletExtrinsicFilterError (591) */
  interface PalletExtrinsicFilterError extends Enum {
    readonly isCannotBlock: boolean;
    readonly isCannotConvertToString: boolean;
    readonly isExtrinsicAlreadyBlocked: boolean;
    readonly isExtrinsicNotBlocked: boolean;
    readonly type: 'CannotBlock' | 'CannotConvertToString' | 'ExtrinsicAlreadyBlocked' | 'ExtrinsicNotBlocked';
  }

  /** @name PalletIdentityManagementError (592) */
  interface PalletIdentityManagementError extends Enum {
    readonly isDelegateeNotExist: boolean;
    readonly isUnauthorizedUser: boolean;
    readonly type: 'DelegateeNotExist' | 'UnauthorizedUser';
  }

  /** @name PalletAssetManagerError (593) */
  interface PalletAssetManagerError extends Enum {
    readonly isAssetAlreadyExists: boolean;
    readonly isAssetTypeDoesNotExist: boolean;
    readonly isAssetIdDoesNotExist: boolean;
    readonly isDefaultAssetTypeRemoved: boolean;
    readonly isAssetIdLimitReached: boolean;
    readonly type: 'AssetAlreadyExists' | 'AssetTypeDoesNotExist' | 'AssetIdDoesNotExist' | 'DefaultAssetTypeRemoved' | 'AssetIdLimitReached';
  }

  /** @name PalletVcManagementSchemaVcSchema (594) */
  interface PalletVcManagementSchemaVcSchema extends Struct {
    readonly id: Bytes;
    readonly author: AccountId32;
    readonly content: Bytes;
    readonly status: PalletVcManagementSchemaStatus;
  }

  /** @name PalletVcManagementSchemaStatus (597) */
  interface PalletVcManagementSchemaStatus extends Enum {
    readonly isActive: boolean;
    readonly isDisabled: boolean;
    readonly type: 'Active' | 'Disabled';
  }

  /** @name PalletVcManagementError (598) */
  interface PalletVcManagementError extends Enum {
    readonly isDelegateeNotExist: boolean;
    readonly isUnauthorizedUser: boolean;
    readonly isVcAlreadyExists: boolean;
    readonly isVcNotExist: boolean;
    readonly isVcSubjectMismatch: boolean;
    readonly isVcAlreadyDisabled: boolean;
    readonly isRequireAdmin: boolean;
    readonly isSchemaNotExists: boolean;
    readonly isSchemaAlreadyDisabled: boolean;
    readonly isSchemaAlreadyActivated: boolean;
    readonly isSchemaIndexOverFlow: boolean;
    readonly isLengthMismatch: boolean;
    readonly type: 'DelegateeNotExist' | 'UnauthorizedUser' | 'VcAlreadyExists' | 'VcNotExist' | 'VcSubjectMismatch' | 'VcAlreadyDisabled' | 'RequireAdmin' | 'SchemaNotExists' | 'SchemaAlreadyDisabled' | 'SchemaAlreadyActivated' | 'SchemaIndexOverFlow' | 'LengthMismatch';
  }

  /** @name PalletGroupError (599) */
  interface PalletGroupError extends Enum {
    readonly isGroupMemberAlreadyExists: boolean;
    readonly isGroupMemberInvalid: boolean;
    readonly type: 'GroupMemberAlreadyExists' | 'GroupMemberInvalid';
  }

  /** @name PalletBitacrossCustodialWallet (601) */
  interface PalletBitacrossCustodialWallet extends Struct {
    readonly btc: Option<U8aFixed>;
    readonly eth: Option<U8aFixed>;
  }

  /** @name PalletBitacrossError (603) */
  interface PalletBitacrossError extends Enum {
    readonly isRequireAdminOrRoot: boolean;
    readonly isRelayerNotExist: boolean;
    readonly isUnsupportedRelayerType: boolean;
    readonly isBtcWalletAlreadyExist: boolean;
    readonly isEthWalletAlreadyExist: boolean;
    readonly type: 'RequireAdminOrRoot' | 'RelayerNotExist' | 'UnsupportedRelayerType' | 'BtcWalletAlreadyExist' | 'EthWalletAlreadyExist';
  }

  /** @name PalletEvmAssertionsAssertion (606) */
  interface PalletEvmAssertionsAssertion extends Struct {
    readonly byteCode: Bytes;
    readonly secrets: Vec<Bytes>;
  }

  /** @name PalletEvmAssertionsError (607) */
  interface PalletEvmAssertionsError extends Enum {
    readonly isAssertionExists: boolean;
    readonly type: 'AssertionExists';
  }

  /** @name PalletTeebagQuotingEnclave (611) */
  interface PalletTeebagQuotingEnclave extends Struct {
    readonly issueDate: u64;
    readonly nextUpdate: u64;
    readonly miscselect: U8aFixed;
    readonly miscselectMask: U8aFixed;
    readonly attributes: U8aFixed;
    readonly attributesMask: U8aFixed;
    readonly mrsigner: U8aFixed;
    readonly isvprodid: u16;
    readonly tcb: Vec<PalletTeebagTcbQeTcb>;
  }

  /** @name PalletTeebagTcbQeTcb (613) */
  interface PalletTeebagTcbQeTcb extends Struct {
    readonly isvsvn: u16;
  }

  /** @name PalletTeebagTcbTcbInfoOnChain (614) */
  interface PalletTeebagTcbTcbInfoOnChain extends Struct {
    readonly issueDate: u64;
    readonly nextUpdate: u64;
    readonly tcbLevels: Vec<PalletTeebagTcbTcbVersionStatus>;
  }

  /** @name PalletTeebagTcbTcbVersionStatus (616) */
  interface PalletTeebagTcbTcbVersionStatus extends Struct {
    readonly cpusvn: U8aFixed;
    readonly pcesvn: u16;
  }

  /** @name PalletTeebagSidechainBlockConfirmation (620) */
  interface PalletTeebagSidechainBlockConfirmation extends Struct {
    readonly blockNumber: u64;
    readonly blockHeaderHash: H256;
  }

  /** @name PalletTeebagError (621) */
  interface PalletTeebagError extends Enum {
    readonly isRequireAdminOrRoot: boolean;
    readonly isEnclaveSignerDecodeError: boolean;
    readonly isSenderIsNotAttestedEnclave: boolean;
    readonly isRemoteAttestationVerificationFailed: boolean;
    readonly isRemoteAttestationTooOld: boolean;
    readonly isInvalidAttestationType: boolean;
    readonly isInvalidSgxMode: boolean;
    readonly isEnclaveNotExist: boolean;
    readonly isEnclaveIdentifierNotExist: boolean;
    readonly isEnclaveIdentifierAlreadyExist: boolean;
    readonly isUnexpectedWorkerType: boolean;
    readonly isWrongMrenclaveForShard: boolean;
    readonly isEnclaveUrlTooLong: boolean;
    readonly isAttestationTooLong: boolean;
    readonly isUnexpectedWorkerMode: boolean;
    readonly isAuthorizedEnclaveNotExist: boolean;
    readonly isAuthorizedEnclaveAlreadyExist: boolean;
    readonly isEnclaveNotAuthorized: boolean;
    readonly isCollateralInvalid: boolean;
    readonly isMaxEnclaveIdentifierOverflow: boolean;
    readonly isMaxAuthorizedEnclaveOverflow: boolean;
    readonly isReceivedUnexpectedSidechainBlock: boolean;
    readonly isInvalidNextFinalizationCandidateBlockNumber: boolean;
    readonly type: 'RequireAdminOrRoot' | 'EnclaveSignerDecodeError' | 'SenderIsNotAttestedEnclave' | 'RemoteAttestationVerificationFailed' | 'RemoteAttestationTooOld' | 'InvalidAttestationType' | 'InvalidSgxMode' | 'EnclaveNotExist' | 'EnclaveIdentifierNotExist' | 'EnclaveIdentifierAlreadyExist' | 'UnexpectedWorkerType' | 'WrongMrenclaveForShard' | 'EnclaveUrlTooLong' | 'AttestationTooLong' | 'UnexpectedWorkerMode' | 'AuthorizedEnclaveNotExist' | 'AuthorizedEnclaveAlreadyExist' | 'EnclaveNotAuthorized' | 'CollateralInvalid' | 'MaxEnclaveIdentifierOverflow' | 'MaxAuthorizedEnclaveOverflow' | 'ReceivedUnexpectedSidechainBlock' | 'InvalidNextFinalizationCandidateBlockNumber';
  }

  /** @name PalletEvmCodeMetadata (622) */
  interface PalletEvmCodeMetadata extends Struct {
    readonly size_: u64;
    readonly hash_: H256;
  }

  /** @name PalletEvmError (624) */
  interface PalletEvmError extends Enum {
    readonly isBalanceLow: boolean;
    readonly isFeeOverflow: boolean;
    readonly isPaymentOverflow: boolean;
    readonly isWithdrawFailed: boolean;
    readonly isGasPriceTooLow: boolean;
    readonly isInvalidNonce: boolean;
    readonly isGasLimitTooLow: boolean;
    readonly isGasLimitTooHigh: boolean;
    readonly isUndefined: boolean;
    readonly isReentrancy: boolean;
    readonly isTransactionMustComeFromEOA: boolean;
    readonly type: 'BalanceLow' | 'FeeOverflow' | 'PaymentOverflow' | 'WithdrawFailed' | 'GasPriceTooLow' | 'InvalidNonce' | 'GasLimitTooLow' | 'GasLimitTooHigh' | 'Undefined' | 'Reentrancy' | 'TransactionMustComeFromEOA';
  }

  /** @name FpRpcTransactionStatus (627) */
  interface FpRpcTransactionStatus extends Struct {
    readonly transactionHash: H256;
    readonly transactionIndex: u32;
    readonly from: H160;
    readonly to: Option<H160>;
    readonly contractAddress: Option<H160>;
    readonly logs: Vec<EthereumLog>;
    readonly logsBloom: EthbloomBloom;
  }

  /** @name EthbloomBloom (630) */
  interface EthbloomBloom extends U8aFixed {}

  /** @name EthereumReceiptReceiptV3 (632) */
  interface EthereumReceiptReceiptV3 extends Enum {
    readonly isLegacy: boolean;
    readonly asLegacy: EthereumReceiptEip658ReceiptData;
    readonly isEip2930: boolean;
    readonly asEip2930: EthereumReceiptEip658ReceiptData;
    readonly isEip1559: boolean;
    readonly asEip1559: EthereumReceiptEip658ReceiptData;
    readonly type: 'Legacy' | 'Eip2930' | 'Eip1559';
  }

  /** @name EthereumReceiptEip658ReceiptData (633) */
  interface EthereumReceiptEip658ReceiptData extends Struct {
    readonly statusCode: u8;
    readonly usedGas: U256;
    readonly logsBloom: EthbloomBloom;
    readonly logs: Vec<EthereumLog>;
  }

  /** @name EthereumBlock (634) */
  interface EthereumBlock extends Struct {
    readonly header: EthereumHeader;
    readonly transactions: Vec<EthereumTransactionTransactionV2>;
    readonly ommers: Vec<EthereumHeader>;
  }

  /** @name EthereumHeader (635) */
  interface EthereumHeader extends Struct {
    readonly parentHash: H256;
    readonly ommersHash: H256;
    readonly beneficiary: H160;
    readonly stateRoot: H256;
    readonly transactionsRoot: H256;
    readonly receiptsRoot: H256;
    readonly logsBloom: EthbloomBloom;
    readonly difficulty: U256;
    readonly number: U256;
    readonly gasLimit: U256;
    readonly gasUsed: U256;
    readonly timestamp: u64;
    readonly extraData: Bytes;
    readonly mixHash: H256;
    readonly nonce: EthereumTypesHashH64;
  }

  /** @name EthereumTypesHashH64 (636) */
  interface EthereumTypesHashH64 extends U8aFixed {}

  /** @name PalletEthereumError (641) */
  interface PalletEthereumError extends Enum {
    readonly isInvalidSignature: boolean;
    readonly isPreLogExists: boolean;
    readonly type: 'InvalidSignature' | 'PreLogExists';
  }

  /** @name PalletSudoError (642) */
  interface PalletSudoError extends Enum {
    readonly isRequireSudo: boolean;
    readonly type: 'RequireSudo';
  }

  /** @name SpRuntimeMultiSignature (644) */
  interface SpRuntimeMultiSignature extends Enum {
    readonly isEd25519: boolean;
    readonly asEd25519: SpCoreEd25519Signature;
    readonly isSr25519: boolean;
    readonly asSr25519: SpCoreSr25519Signature;
    readonly isEcdsa: boolean;
    readonly asEcdsa: SpCoreEcdsaSignature;
    readonly type: 'Ed25519' | 'Sr25519' | 'Ecdsa';
  }

  /** @name SpCoreEd25519Signature (645) */
  interface SpCoreEd25519Signature extends U8aFixed {}

  /** @name SpCoreSr25519Signature (647) */
  interface SpCoreSr25519Signature extends U8aFixed {}

  /** @name SpCoreEcdsaSignature (648) */
  interface SpCoreEcdsaSignature extends U8aFixed {}

  /** @name FrameSystemExtensionsCheckNonZeroSender (651) */
  type FrameSystemExtensionsCheckNonZeroSender = Null;

  /** @name FrameSystemExtensionsCheckSpecVersion (652) */
  type FrameSystemExtensionsCheckSpecVersion = Null;

  /** @name FrameSystemExtensionsCheckTxVersion (653) */
  type FrameSystemExtensionsCheckTxVersion = Null;

  /** @name FrameSystemExtensionsCheckGenesis (654) */
  type FrameSystemExtensionsCheckGenesis = Null;

  /** @name FrameSystemExtensionsCheckNonce (657) */
  interface FrameSystemExtensionsCheckNonce extends Compact<u32> {}

  /** @name FrameSystemExtensionsCheckWeight (658) */
  type FrameSystemExtensionsCheckWeight = Null;

  /** @name PalletTransactionPaymentChargeTransactionPayment (659) */
  interface PalletTransactionPaymentChargeTransactionPayment extends Compact<u128> {}

} // declare module
