export default {
    types: {
        OmniAesRequest: {
            mrenclave: "MrEnclave",
            key: "Vec<u8>",
            payload: "AesOutput",
        },
        PlainRequest: {
            mrenclave: "MrEnclave",
            payload: "Vec<u8>",
        },
        MrEnclave: "H256",
        NativeCall: {
            _enum: {
                request_intent: "(LitentryIdentity, Intent)",
                create_account_store: "(LitentryIdentity)",
                add_account: "(LitentryIdentity, LitentryIdentity, LitentryValidationData, bool, Option<Vec<OmniAccountPermission>>)",
                remove_accounts: "(LitentryIdentity, Vec<LitentryIdentity>)",
                publicize_account: "(LitentryIdentity, LitentryIdentity)",
                set_permissions: "(LitentryIdentity, LitentryIdentity, Vec<OmniAccountPermission>)",
            },
        },
        NativeCallAuthenticated: {
            call: "NativeCall",
            nonce: "Index",
            authentication: "Authentication",
        },
        Authentication: {
            _enum: {
                Web3: "(LitentryMultiSignature)",
                Email: "(Text)",
                AuthToken: "(Text)",
                OAuth2: "(OAuth2Data)",
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
        NativeCallResponse: "Result<NativeCallOk, NativeCallError>",
        NativeCallOk: {
            _enum: {
                AuthToken: "(Text)",
                ExtrinsicReport: "(XtReport)",
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
        NativeCallError: {
            _enum: {
                UnexpectedCall: "(Text)",
                UnauthorizedSender: "Null",
                AuthTokenCreationFailed: "Null",
                InternalError: "Null",
                IInvalidMemberIdentity: "Null",
                ValidationDataVerificationFailed: "Null",
            },
        },
    },
};
