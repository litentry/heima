export default {
    types: {
        TrustedOperation: {
            _enum: {
                indirect_call: "(TrustedCallSigned)",
                direct_call: "(TrustedCallSigned)",
                get: "(Getter)",
            },
        },
        TrustedCallSigned: {
            call: "TrustedCall",
            index: "u32",
            signature: "LitentryMultiSignature",
        },
        TrustedGetterSigned: {
            getter: "TrustedGetter",
            signature: "LitentryMultiSignature",
        },

        //important
        TrustedGetter: {
            _enum: {
                free_balance: "(LitentryIdentity)",
                reserved_balance: "(LitentryIdentity)",
                __Unused_evm_nonce: "Null",
                __Unused_evm_account_codes: "Null",
                __Unused_evm_account_storages: "Null",
                id_graph: "(LitentryIdentity)",
            },
        },
        //important
        TrustedCall: {
            _enum: {
                link_identity:
                    "(LitentryIdentity, LitentryIdentity, LitentryIdentity, LitentryValidationData, Vec<Web3Network>, Option<RequestAesKey>, H256)",
                deactivate_identity: "(LitentryIdentity, LitentryIdentity, LitentryIdentity, Option<RequestAesKey>, H256)",
                activate_identity: "(LitentryIdentity, LitentryIdentity, LitentryIdentity, Option<RequestAesKey>, H256)",
                request_vc: "(LitentryIdentity, LitentryIdentity, Assertion, Option<RequestAesKey>, H256)",
                set_identity_networks:
                    "(LitentryIdentity, LitentryIdentity, LitentryIdentity, Vec<Web3Network>, Option<RequestAesKey>, H256)",
                __Unused_remove_identity: "Null",
                request_batch_vc: "(LitentryIdentity, LitentryIdentity, BoundedVec<Assertion, ConstU32<32>>, Option<RequestAesKey>, H256)",

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
                    "(LitentryIdentity, LitentryIdentity, LitentryIdentity, Vec<Web3Network>, Option<RequestAesKey>, H256)",

                __Unused_21: "Null",
                __Unused_22: "Null",
                __Unused_23: "Null",
                __Unused_24: "Null",

                clean_id_graphs: "(LitentryIdentity)",
                request_intent: "(LitentryIdentity, Intent)",
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
