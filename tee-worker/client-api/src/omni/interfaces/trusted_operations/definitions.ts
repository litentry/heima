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
        TrustedOperation: {
            _enum: {
                indirect_call: "(TrustedCallSigned)",
                direct_call: "(TrustedCallSigned)",
                get: "(Getter)",
            },
        },
        TrustedOperationAuthenticated: {
            _enum: {
                indirect_call: "Null",
                direct_call: "(TrustedCallAuthenticated)",
                get: "(Getter)",
            },
        },
        TrustedCallSigned: {
            call: "TrustedCall",
            index: "Index",
            signature: "HeimaMultiSignature",
        },
        TrustedGetterSigned: {
            getter: "TrustedGetter",
            signature: "HeimaMultiSignature",
        },

        TCAuthentication: {
            _enum: {
                Web3: "HeimaMultiSignature",
                Email: "Text",
                AuthToken: "Text",
            },
        },
        TrustedCallAuthenticated: {
            call: "TrustedCall",
            nonce: "Index",
            authentication: "TCAuthentication",
        },

        //important
        TrustedGetter: {
            _enum: {
                free_balance: "(HeimaIdentity)",
                reserved_balance: "(HeimaIdentity)",
                __Unused_evm_nonce: "Null",
                __Unused_evm_account_codes: "Null",
                __Unused_evm_account_storages: "Null",
                id_graph: "(HeimaIdentity)",
            },
        },

        TrustedCall: {
            _enum: {
                link_identity:
                    "(HeimaIdentity, HeimaIdentity, HeimaIdentity, HeimaValidationData, Vec<Web3Network>, Option<RequestAesKey>, H256)",
                deactivate_identity: "(HeimaIdentity, HeimaIdentity, HeimaIdentity, Option<RequestAesKey>, H256)",
                activate_identity: "(HeimaIdentity, HeimaIdentity, HeimaIdentity, Option<RequestAesKey>, H256)",
                request_vc: "(HeimaIdentity, HeimaIdentity, Assertion, Option<RequestAesKey>, H256)",
                set_identity_networks:
                    "(HeimaIdentity, HeimaIdentity, HeimaIdentity, Vec<Web3Network>, Option<RequestAesKey>, H256)",
                __Unused_remove_identity: "Null",
                request_batch_vc:
                    "(HeimaIdentity, HeimaIdentity, BoundedVec<Assertion, ConstU32<32>>, Option<RequestAesKey>, H256)",

                __Unused_7: "Null",
                __Unused_8: "Null",
                __Unused_9: "Null",
                __Unused_10: "Null",
                __Unused_11: "Null",
                __Unused_12: "Null",
                __Unused_13: "Null",
                __Unused_14: "Null",
                __Unused_15: "Null",
                __Unused_16: "Null",
                __Unused_17: "Null",
                __Unused_18: "Null",
                __Unused_19: "Null",

                // this trusted call can only be requested directly by root or enclave_signer_account
                link_identity_callback:
                    "(HeimaIdentity, HeimaIdentity, HeimaIdentity, Vec<Web3Network>, Option<RequestAesKey>, H256)",

                __Unused_21: "Null",
                __Unused_22: "Null",
                __Unused_23: "Null",
                __Unused_24: "Null",

                clean_id_graphs: "(HeimaIdentity)",
                request_intent: "(HeimaIdentity, Intent)",
                create_account_store: "(HeimaIdentity)",
                add_account: "(HeimaIdentity, HeimaIdentity, HeimaValidationData, bool)",
                remove_accounts: "(HeimaIdentity, Vec<HeimaIdentity>)",
                publicize_account: "(HeimaIdentity, HeimaIdentity)",
                request_auth_token: "(HeimaIdentity, AuthOptions)",
            },
        },
        TrustedOperationStatus: {
            _enum: {
                Submitted: null,
                Future: null,
                Ready: null,
                Broadcast: null,
                InSidechainBlock: "H256",
                Retracted: null,
                FinalityTimeout: null,
                Finalized: null,
                Usurped: null,
                Dropped: null,
                Invalid: null,
                TopExecuted: "Bytes",
            },
        },
    },
};
