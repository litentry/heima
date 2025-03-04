export default {
    types: {
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
            signature: "LitentryMultiSignature",
        },
        TrustedGetterSigned: {
            getter: "TrustedGetter",
            signature: "LitentryMultiSignature",
        },

        TCAuthentication: {
            _enum: {
                Web3: "LitentryMultiSignature",
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
                free_balance: "(LitentryIdentity)",
                reserved_balance: "(LitentryIdentity)",
                __Unused_evm_nonce: "Null",
                __Unused_evm_account_codes: "Null",
                __Unused_evm_account_storages: "Null",
                id_graph: "(LitentryIdentity)",
            },
        },

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
                create_account_store: "(LitentryIdentity)",
                add_account: "(LitentryIdentity, LitentryIdentity, LitentryValidationData, bool)",
                remove_accounts: "(LitentryIdentity, Vec<LitentryIdentity>)",
                publicize_account: "(LitentryIdentity, LitentryIdentity)",
                request_auth_token: "(LitentryIdentity, AuthOptions)",
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

        /**
         * Native tasks (OmniAccount) results
         * @see tee-worker/identity/litentry/core/native-task/receiver/src/lib.rs
         */
        TrustedCallResult: "Result<TrustedCallOk, TrustedCallError>",
        TrustedCallOk: {
            _enum: {
                ExtrinsicReport: "ExtrinsicReport",
                RequestVcResult: "RequestVcResultOrError",
                AuthToken: "Text",
            },
        },
        ExtrinsicReport: {
            // Hash of the extrinsic.
            extrinsic_hash: "H256",
            // Block hash of the block the extrinsic was included in.
            // Only available if watched until at least `InBlock`.
            block_hash: "Option<H256>",
            // Last known Transaction Status.
            status: "TransactionStatus",
        },
        TransactionStatus: {
            _enum: {
                /// Transaction is part of the future queue.
                Future: "Null",
                /// Transaction is part of the ready queue.
                Ready: "Null",
                /// The transaction has been broadcast to the given peers.
                Broadcasted: "Null",
                /// Transaction has been included in block with given hash.
                InBlock: "H256",
                /// The block this transaction was included in has been retracted.
                Retracted: "H256",
                /// Maximum number of finality watchers has been reached,
                /// old watchers are being removed.
                FinalityTimeout: "H256",
                /// Transaction has been finalized by a finality-gadget, e.g GRANDPA
                Finalized: "H256",
                /// Transaction has been replaced in the pool, by another transaction
                /// that provides the same tags. (e.g. same (sender, nonce)).
                Usurped: "H256",
                /// Transaction has been dropped from the pool because of the limit.
                Dropped: "Null",
                /// Transaction is no longer valid in the current state.
                Invalid: "Null",
            },
        },
        TrustedCallError: {
            _enum: {
                UnexpectedCall: "Text",
                ShieldingKeyRetrievalFailed: "Text", // Stringified itp_sgx_crypto::Error
                RequestPayloadDecodingFailed: "Null",
                ParentchainDataRetrievalFailed: "Text", // Stringified itp_stf_state_handler::Error
                InvalidSignerAccount: "Null",
                UnauthorizedSigner: "Null",
                InvalidMemberIdentity: "Null",
                MissingAesKey: "Null",
                MrEnclaveRetrievalFailed: "Null",
                EnclaveSignerRetrievalFailed: "Null",
                AuthenticationVerificationFailed: "Null",
                ValidationDataVerificationFailed: "Null",
                ConnectionHashNotFound: "Text",
                MetadataRetrievalFailed: "Text", // Stringified itp_node_api_metadata_provider::Error
                InvalidMetadata: "Text", // Stringified itp_node_api_metadata::Error
                TrustedCallSendingFailed: "Text", // Stringified mpsc::SendError<(H256, TrustedCall)>
                CallSendingFailed: "Text",
                ExtrinsicConstructionFailed: "Text", // Stringified itp_extrinsics_factory::Error
                ExtrinsicSendingFailed: "Text", // Stringified sgx_status_t
                InvalidRequest: "Null",
            },
        },
    },
};
