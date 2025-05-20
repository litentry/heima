export default {
  types: {
    RawTask: {
      _enum: {
        Plain: "NativeTaskWrapper",
        Aes: "AesTask",
      },
    },
    NativeTaskWrapper: {
      task: "NativeTask",
      nonce: "Option<Nonce>",
      auth: "Option<OmniAuth>",
    },
    MrEnclave: "H256",
    NativeTask: {
      _enum: {
        RequestAuthToken: "(Identity)",
        RequestIntent: "(Identity, u32, Intent)",
        CreateAccountStore: "(Identity)",
        AddAccount:
          "(Identity, Identity, ValidationData, bool, Option<Vec<OmniAccountPermission>>)",
        RemoveAccounts: "(Identity, Vec<Identity>)",
        PublicizeAccount: "(Identity, Identity)",
        SetPermissions: "(Identity, Identity, Vec<OmniAccountPermission>)",
        __Unused7: "Null",
        __Unused8: "Null",
        __Unused9: "Null",
        __Unused10: "Null",
        __Unused11: "Null",
        __Unused12: "Null",
        __Unused13: "Null",
        __Unused14: "Null",
        __Unused15: "Null",
        __Unused16: "Null",
        __Unused17: "Null",
        __Unused18: "Null",
        __Unused19: "Null",
        PumpxRequestJwt:
          "(Identity, String, Option<String>, Option<String>, Option<String>)",
      },
    },
    OmniAuth: {
      _enum: {
        Web3: "(HeimaMultiSignature)",
        Email: "(Text, Text)",
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
    NativeTaskResponse: "Result<NativeTaskOk, NativeTaskError>",
    NativeTaskOk: {
      _enum: {
        ExtrinsicReport: "XtReport",
        AuthToken: "Text",
        PumpxRequestJwt: "PumpxRequestJwt",
        RequestIntentResult: "RequestIntentResult",
      },
    },
    RequestIntentResult: {
      intent_id: "u32",
      success: "bool",
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
    PumpxRequestJwt: {
      access_token: "Text",
      id_token: "Text",
      backend_response: "PumpxUserConnectResponse",
    },
    PumpxUserConnectResponse: {
      code: "u32",
      message: "Text",
      data: "PumpxConnectedUser",
    },
    PumpxConnectedUser: {
      userId: "Text",
      googleAuthCheck: "bool",
    },
    NativeTaskError: {
      _enum: {
        UnauthorizedSender: "Null",
        AuthTokenCreationFailed: "Null",
        InternalError: "Null",
        InvalidMemberIdentity: "Null",
        ValidationDataVerificationFailed: "Null",
        UnsupportedIdentityType: "Null",
        PumpxApiError: "Null",
        IntentNonceMismatch: "Null",
      },
    },
    Nonce: "u32",
    AesTask: {
      key: "Vec<u8>",
      payload: "AesOutput",
    },
  },
};
