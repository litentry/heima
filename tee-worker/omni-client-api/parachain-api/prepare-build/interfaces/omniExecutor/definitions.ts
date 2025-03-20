export default {
    types: {
        AesRequest: {
            mrenclave: "MrEnclave",
            key: "Vec<u8>",
            payload: "AesOutput",
        },
        PlainRequest: {
            mrenclave: "MrEnclave",
            payload: "Vec<u8>",
        },
        AesOutput: {
            ciphertext: "Vec<u8>",
            aad: "Vec<u8>",
            nonce: "[u8; 12]",
        },
        MrEnclave: "H256",
        NativeCall: {
            _enum: {
                request_auth_token: "(Identity, AuthOptions)",
                request_intent: "(Identity, Intent)",
                create_account_store: "(Identity)",
                add_account: "(Identity, Identity, ValidationData, bool, Option<Vec<OmniAccountPermission>>)",
                remove_accounts: "(Identity, Vec<Identity>)",
                publicize_account: "(Identity, Identity)",
                set_permissions: "(Identity, Identity, Vec<OmniAccountPermission>)",
            },
        },
        AuthOptions: {
            expires_at: "u32",
        },
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
        NativeCallAuthenticatedOperation: {
            operation: "NativeCall",
            nonce: "Index",
            authentication: "Authentication",
        },
        NativeQueryAuthenticatedOperation: {
            operation: "NativeQuery",
            nonce: "Index",
            authentication: "Authentication",
        },
        Authentication: {
            _enum: {
                Web3: "(MultiSignature)",
                Email: "(Text)",
                AuthToken: "(Text)",
                OAuth2: "(OAuth2Data)",
            },
        },
        NativeQuery: {
            _enum: {
                get_account_store: "(Identity)",
            },
        },
        OAuth2Data: {
            provider: "OAuth2Provider",
            code: "Text",
            state: "Text",
            redirect_uri: "Text",
        },
        OAuth2Provider: {
            _enum: ["Google"],
        },
        NativeOperationResponse: "Result<NativeOperationOk, NativeOperationError>",
        NativeOperationOk: {
            _enum: {
                CallResponse: "CallResponse",
                QueryResponse: "QueryResponse",
            },
        },
        CallResponse: {
            _enum: {
                ExtrinsicReport: "XtReport",
                AuthToken: "Text",
            },
        },
        QueryResponse: {
            _enum: {
                AccountStore: "Vec<Identity>",
            },
        },
        XtReport: {
            // Hash of the extrinsic.
            extrinsic_hash: "H256",
            // Block hash of the block the extrinsic was included in.
            // Only available if watched until at least `InBlock`.
            block_hash: "Option<H256>",
            // Last known Transaction Status.
            status: "TxStatus",
        },
        TxStatus: {
            _enum: {
                // Transaction is part of the future queue.
                Future: "Null",
                // Transaction is part of the ready queue.
                Ready: "Null",
                // The transaction has been broadcast to the given peers.
                Broadcast: "Vec<Text>",
                // Transaction has been included in block with given hash.
                InBlock: "H256",
                // The block this transaction was included in has been retracted.
                Retracted: "H256",
                // Maximum number of finality watchers has been reached,
                // old watchers are being removed.
                FinalityTimeout: "H256",
                // Transaction has been finalized by a finality-gadget, e.g GRANDPA
                Finalized: "H256",
                // Transaction has been replaced in the pool, by another transaction
                // that provides the same tags. (e.g. same (sender, nonce)).
                Usurped: "H256",
                // Transaction has been dropped from the pool because of the limit.
                Dropped: "Null",
                // Transaction is no longer valid in the current state.
                Invalid: "Null",
            },
        },
        NativeOperationError: {
            _enum: {
                UnauthorizedSender: "Null",
                AuthTokenCreationFailed: "Null",
                InternalError: "Null",
                InvalidMemberIdentity: "Null",
                ValidationDataVerificationFailed: "Null",
                AuthTokenExpirationTooLong: "Null",
            },
        },
    },
};
