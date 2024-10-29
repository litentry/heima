export default {
    types: {
        Intent: {
            _enum: {
                TransferEthereum: "IntentTransferEthereum",
                CallEthereum: "IntentCallEthereum",
                SystemRemark: "Bytes",
                TransferNative: "IntentTransferNative",
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
    },
};
