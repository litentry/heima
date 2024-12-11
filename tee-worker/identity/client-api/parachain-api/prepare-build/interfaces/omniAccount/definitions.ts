export default {
    types: {
        Intent: {
            _enum: {
                TransferEthereum: "IntentTransferEthereum",
                CallEthereum: "IntentCallEthereum",
                SystemRemark: "Bytes",
                TransferNative: "IntentTransferNative",
                TransferSolana: "IntentTransferSolana",
            },
        },
        IntentTransferEthereum: {
            to: "H160",
            value: "[u8;32]",
        },
        IntentCallEthereum: {
            address: "H160",
            input: "Bytes",
        },
        IntentTransferNative: {
            to: "AccountId32",
            value: "u128",
        },
        IntentTransferSolana: {
            to: "[u8; 32]",
            value: "u64",
        },

        /**
         * AccountStore
         * @see common/primitives/core/src/omni_account.rs
         */
        MemberAccount: {
            _enum: {
                Public: "LitentryIdentity",
                Private: "(Bytes,H256)",
            },
        },
    },
};
