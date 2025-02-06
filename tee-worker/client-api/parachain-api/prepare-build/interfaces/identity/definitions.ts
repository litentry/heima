export default {
    types: {
        GenericEventWithAccount: {
            account: "AccountId",
        },
        LinkIdentityResult: {
            mutated_id_graph: "AesOutput",
            id_graph_hash: "H256",
        },
        DeactivateIdentityResult: {
            mutated_id_graph: "AesOutput",
            id_graph_hash: "H256",
        },
        ActivateIdentityResult: {
            mutated_id_graph: "AesOutput",
            id_graph_hash: "H256",
        },
        SetIdentityNetworksResult: {
            mutated_id_graph: "AesOutput",
            id_graph_hash: "H256",
        },
        LitentryIdentity: {
            _enum: {
                Twitter: "IdentityString",
                Discord: "IdentityString",
                Github: "IdentityString",
                Substrate: "Address32",
                Evm: "Address20",
                Bitcoin: "Address33",
                Solana: "Address32",
                Email: "IdentityString",
            },
        },
        Address32: "[u8;32]",
        Address20: "[u8;20]",
        Address33: "[u8;33]",
        IdentityString: "Vec<u8>",
        Web3Network: {
            _enum: [
                "Polkadot",
                "Kusama",
                "Litentry",
                "Litmus",
                "LitentryRococo",
                "Khala",
                "SubstrateTestnet",
                "Ethereum",
                "Bsc",
                "BitcoinP2tr",
                "BitcoinP2pkh",
                "BitcoinP2sh",
                "BitcoinP2wpkh",
                "BitcoinP2wsh",
                "Polygon",
                "Arbitrum",
                "Solana",
                "Combo",
            ],
        },
        LitentryValidationData: {
            _enum: {
                Web2Validation: "Web2ValidationData",
                Web3Validation: "Web3ValidationData",
            },
        },
        Web2ValidationData: {
            _enum: {
                Twitter: "TwitterValidationData",
                Discord: "DiscordValidationData",
                Email: "EmailValidationData",
            },
        },
        TwitterValidationData: {
            _enum: {
                PublicTweet: "PublicTweet",
                OAuth2: "TwitterOAuth2",
            },
        },
        PublicTweet: {
            tweet_id: "Vec<u8>",
        },
        TwitterOAuth2: {
            code: "Vec<u8>",
            state: "Vec<u8>",
            redirect_uri: "Vec<u8>",
        },
        DiscordValidationData: {
            _enum: {
                PublicMessage: "PublicMessage",
                OAuth2: "DiscordOAuth2",
            },
        },
        PublicMessage: {
            channel_id: "Vec<u8>",
            message_id: "Vec<u8>",
            guild_id: "Vec<u8>",
        },
        DiscordOAuth2: {
            code: "Vec<u8>",
            redirect_uri: "Vec<u8>",
        },
        EmailValidationData: {
            email: "Text",
            verification_code: "Text",
        },
        Web3ValidationData: {
            _enum: {
                Substrate: "Web3CommonValidationData",
                Evm: "Web3CommonValidationData",
                Bitcoin: "Web3CommonValidationData",
                Solana: "Web3CommonValidationData",
            },
        },
        Web3CommonValidationData: {
            message: "Vec<u8>",
            signature: "LitentryMultiSignature",
        },

        LitentryMultiSignature: {
            _enum: {
                Ed25519: "Ed25519Signature",
                Sr25519: "Sr25519Signature",
                Ecdsa: "EcdsaSignature",
                Ethereum: "EthereumSignature",
                Bitcoin: "BitcoinSignature",
            },
        },
        Ed25519Signature: "([u8; 64])",
        Sr25519Signature: "([u8; 64])",
        EcdsaSignature: "([u8; 65])",
        EthereumSignature: "([u8; 65])",
        BitcoinSignature: "([u8; 65])",

        IdentityGenericEvent: {
            who: "AccountId",
            identity: "LitentryIdentity",
            id_graph: "Vec<(LitentryIdentity, IdentityContext)>",
        },

        IdentityStatus: {
            _enum: ["Active", "Inactive"],
        },

        IdentityContext: {
            link_block: "BlockNumber",
            web3networks: "BoundedWeb3Network",
            status: "IdentityStatus",
        },
        BoundedWeb3Network: "BoundedVec<Web3Network, ConstU32<128>>",
    },
};
