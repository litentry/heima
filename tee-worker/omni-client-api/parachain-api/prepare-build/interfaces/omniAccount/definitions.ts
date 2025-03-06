export default {
    types: {
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
    },
};
