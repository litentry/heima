// @todo move to a better place, and make it more generic, at least define the type
export const mockBatchAssertion = [
    {
        description: 'request_batch_vc trial test',
        assertion: [{ A7: '5' }, { A8: ['Litentry'] }, { A20: 'A20' }],
    },
    {
        description: 'Have identified at least one account/address in both Web2 and Web3.',
        assertion: {
            A1: 'A1',
        },
    },
    {
        description: 'The user is a member of Litentry Discord.',
        assertion: {
            A2: '807161594245152800',
        },
    },

    {
        description:
            'Have commented in Litentry Discord #🪂id-hubber channel. Channel link: https://discord.com/channels/807161594245152800/1093886939746291882',
        assertion: {
            A3: ['A3', 'A3', 'A3'],
        },
    },
    {
        description: 'The length of time a user continues to hold LIT token',
        assertion: {
            A4: '10',
        },
    },
];

// https://github.com/litentry/heima/tree/dev/tee-worker/litentry/core/assertion-build/src
export const mockAssertions = [
    {
        description: 'Have identified at least one account/address in both Web2 and Web3.',
        assertion: {
            A1: 'A1',
        },
    },
    {
        description: 'The user is a member of Litentry Discord.',
        assertion: {
            A2: '807161594245152800',
        },
    },

    {
        description:
            'Have commented in Litentry Discord #🪂id-hubber channel. Channel link: https://discord.com/channels/807161594245152800/1093886939746291882',
        assertion: {
            A3: ['A3', 'A3', 'A3'],
        },
    },

    // litentry-archive
    {
        description:
            'The user is an early bird user of the IdentityHub EVM version and has generated at least 1 credential during 2023 Aug 14th ~ Aug 21st.',
        assertion: {
            A20: 'A20',
        },
    },

    // Achainable
    {
        description: `A trader or liquidity provider of Uniswap V2 or V3
                      Uniswap V2 Factory Contract: 0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f
                      Uniswap V3 Factory Contract: 0x1f98431c8ad98523631ae4a59f267346ea31f984`,
        assertion: {
            Achainable: {
                Basic: {
                    name: 'Uniswap V2/V3 user',
                    chain: ['Ethereum'],
                },
            },
        },
    },
    {
        description: 'The number of ETH tokens hold > 0',
        assertion: {
            Achainable: {
                Amount: {
                    name: 'Balance over {amount}',
                    chain: ['Ethereum'],
                    amount: '0',
                },
            },
        },
    },
    {
        description: 'The class of year that Ethereum account was created (must have on-chain records)',
        assertion: {
            Achainable: {
                ClassOfYear: {
                    name: 'Account created between {dates}',
                    chain: ['Ethereum'],
                },
            },
        },
    },
    {
        description: 'A deployer of a smart contract on Ethereum',
        assertion: {
            Achainable: {
                Amount: {
                    name: 'Created over {amount} contracts',
                    chain: ['Ethereum'],
                    amount: '0',
                },
            },
        },
    },
    {
        description: 'A deployer of a smart contract on Ethereum',
        assertion: {
            Achainable: {
                AmountToken: {
                    name: 'LIT Holding Amount',
                    chain: ['Litentry'],
                    amount: '0',
                },
            },
        },
    },

    {
        description: 'The length of time a user continues to hold LIT token',
        assertion: {
            A4: '10',
        },
    },
    {
        description: "The range of the user's Twitter follower count",
        assertion: {
            A6: [],
        },
    },
    {
        description: 'The length of time a user continues to hold DOT token',
        assertion: {
            A7: '5',
        },
    },
    {
        description:
            'The range of number of transactions a user has made for a specific token on all supported networks(Litentry)',
        assertion: {
            A8: ['Litentry'],
        },
    },
    {
        description: 'The user has participated in any Polkadot on-chain governance events',
        assertion: {
            A14: [],
        },
    },

    // SORA
    {
        description:
            'Congratulations on your participation in our first quiz in collaboration with our partner, SORA. You have embarked on an exciting educational journey, exploring the world of DeFi & Web3 Identity, we truly appreciate your curiosity and dedication.',
        assertion: {
            GenericDiscordRole: {
                SoraQuiz: 'Attendee',
            },
        },
    },
    // VIP3
    {
        description: 'VIP3 Silver Card Holder',
        assertion: {
            VIP3MembershipCard: 'Silver',
        },
    },
    // BNB domain-nodereal
    {
        description: 'Holding a certain amount of bnb domain names',
        assertion: {
            BnbDomainHolding: 'BnbDomainHolding',
        },
    },
    {
        description: 'Holding a certain amount of 000-999.bnb domain names',
        assertion: {
            BnbDigitDomainClub: 'Bnb999ClubMember',
        },
    },
    // OneBlock
    {
        description: 'A participant to the course co-created by OneBlock+ and Parity',
        assertion: {
            Oneblock: 'CourseCompletion',
        },
    },
    // Geniidata
    {
        description: 'NFT holder',
        assertion: {
            Brc20AmountHolder: [],
        },
    },

    // NftHolder
    {
        description: 'You are a holder of a certain kind of NFT',
        assertion: {
            NftHolder: 'WeirdoGhostGang',
        },
    },
    {
        description: 'You are a holder of a certain kind of NFT',
        assertion: {
            NftHolder: 'Club3Sbt',
        },
    },
    {
        description: 'You are a holder of a certain kind of NFT',
        assertion: {
            NftHolder: 'MFan',
        },
    },
    {
        description: 'You are a holder of a certain kind of NFT',
        assertion: {
            NftHolder: 'Mvp',
        },
    },

    // TokenHoldingAmount
    {
        description: 'The amount of TRX you are holding',
        assertion: {
            TokenHoldingAmount: 'TRX',
        },
    },
    {
        description: 'The amount of BNB you are holding',
        assertion: {
            TokenHoldingAmount: 'BNB',
        },
    },
    {
        description: 'The amount of ETH you are holding',
        assertion: {
            TokenHoldingAmount: 'ETH',
        },
    },
    {
        description: 'The amount of LIT you are holding',
        assertion: {
            TokenHoldingAmount: 'LIT',
        },
    },
    {
        description: 'The amount of SOL you are holding',
        assertion: {
            TokenHoldingAmount: 'SOL',
        },
    },
    {
        description: 'The amount of NFP you are holding',
        assertion: {
            TokenHoldingAmount: 'NFP',
        },
    },
    {
        description: 'The amount of BTC you are holding',
        assertion: {
            TokenHoldingAmount: 'BTC',
        },
    },
    {
        description: 'The amount of SHIB you are holding',
        assertion: {
            TokenHoldingAmount: 'SHIB',
        },
    },
    {
        description: 'The amount of BEAN you are holding',
        assertion: {
            TokenHoldingAmount: 'BEAN',
        },
    },
    {
        description: 'The amount of AN you are holding',
        assertion: {
            TokenHoldingAmount: 'AN',
        },
    },
    {
        description: 'The amount of TUNA you are holding',
        assertion: {
            TokenHoldingAmount: 'TUNA',
        },
    },

    {
        description: 'The amount of LIT you are staking',
        assertion: {
            LITStaking: 'LITStaking',
        },
    },
    {
        description: 'The amount of a Ton you are holding',
        assertion: {
            EVMAmountHolding: 'Ton',
        },
    },

    // PlatformUser
    {
        description: 'You are a user of platform KaratDao',
        assertion: {
            PlatformUser: 'KaratDao',
        },
    },
    {
        description: 'You are a user of platform MagicCraft',
        assertion: {
            PlatformUser: 'MagicCraftStaking',
        },
    },
    {
        description: 'You are a user of platform DarenMarket',
        assertion: {
            PlatformUser: 'DarenMarket',
        },
    },

    // CryptoSummary
    {
        description: 'Generate a summary of your on-chain identity',
        assertion: {
            CryptoSummary: [],
        },
    },

    // WeirdoGhostGangHolder
    {
        description: 'You are WeirdoGhostGang NFT holder',
        assertion: {
            WeirdoGhostGangHolder: [],
        },
    },

    // LinkedIdentities
    {
        description: 'All web2 and web3 identities you linked',
        assertion: {
            LinkedIdentities: [],
        },
    },
];
