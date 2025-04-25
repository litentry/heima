import * as identity from "../identity/definitions";

const { Address20, Address32, Address33 } = identity.default.types;

export default {
    types: {
        Intent: {
            _enum: {
                TransferEthereum: "IntentTransferEthereum",
                CallEthereum: "IntentCallEthereum",
                SystemRemark: "Bytes",
                TransferNative: "IntentTransferNative",
                TransferSolana: "IntentTransferSolana",
                // TODO: add `Swap`
            },
        },
        SwapOrder: {
            from_asset: "ChainAsset",
            from_amount: "u64",
            to_asset: "ChainAsset",
            to_address: "Option<HeimaMultiAddress>",
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
                Public: "Identity",
                Private: "(Bytes,H256)",
            },
        },
        OmniAccountPermission: {
            _enum: ["All", "AccountManagement", "RequestNativeIntent", "RequestEthereumIntent", "RequestSolanaIntent"],
        },
        AuthOptions: {
            expires_at: "u32",
        },

        ChainAsset: {
            _enum: {
                Ethereum: "(u32, EthereumToken)",
                Solana: "SolanaToken",
            },
        },
        EthereumToken: {
            _enum: {
                Native: "Null",
                ERC20: "Address20",
            },
        },
        SolanaToken: {
            _enum: {
                Native: "Null",
                SPL: "Address32",
            },
        },
        HeimaMultiAddress: {
            _enum: {
                Address20: "Address20",
                Address32: "Address32",
                Address33: "Address33",
            },
        },
    },
};
