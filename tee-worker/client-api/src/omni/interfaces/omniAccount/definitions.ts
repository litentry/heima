import * as identity from "../identity/definitions";

const { Address20, Address32, Address33 } = identity.default.types;

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
      _enum: [
        "All",
        "AccountManagement",
        "RequestNativeIntent",
        "RequestEthereumIntent",
        "RequestSolanaIntent",
      ],
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
