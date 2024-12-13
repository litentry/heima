import { TypeRegistry } from '@polkadot/types';
import {
  base58Encode,
  cryptoWaitReady,
  base64Decode,
} from '@polkadot/util-crypto';

import { identity, trusted_operations } from '@litentry/parachain-api';

import { createLitentryValidationDataType } from './validation-data';

const types = {
  ...identity.types, // LitentryIdentity is defined here
  ...trusted_operations.types, // TrustedCall types are defined here
};

let registry: TypeRegistry;

beforeAll(async () => {
  await cryptoWaitReady();

  registry = new TypeRegistry();
  registry.register(types);
});

describe('Bitcoin', () => {
  test('0x prefix is restored for hex encoded challenge code', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle:
          '036cb496b29f281f34b52c49e5dd56f4f921db3f9353cbe254040f581662a924d7',
        type: 'Bitcoin',
      },
      {
        message:
          'a7960fc417cc24e9346c36cba74bf43a1fac66d1c2eacb474e56343f09e8340ac',
        signature:
          'H7+Fq/aqJsFSbKQvwalp9+Nju7+eTnvxgqV47QGOdlDWMiqJpDPSa2zYSdpeIXcqWclaNPUYtcxviCJ/StMN6vQ=',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb3Validation).toBeTruthy();
    expect(validationData.asWeb3Validation.isBitcoin).toBeTruthy();

    const data = validationData.asWeb3Validation.asBitcoin;
    expect(data.message.toHuman()).toEqual(
      '0xa7960fc417cc24e9346c36cba74bf43a1fac66d1c2eacb474e56343f09e8340ac'
    );
    expect(data.signature.isBitcoin).toBeTruthy();
    expect(
      data.signature.asBitcoin.eq(
        base64Decode(
          'H7+Fq/aqJsFSbKQvwalp9+Nju7+eTnvxgqV47QGOdlDWMiqJpDPSa2zYSdpeIXcqWclaNPUYtcxviCJ/StMN6vQ='
        )
      )
    ).toBeTruthy();
  });

  test('utf-8 challenge codes are preserved as is', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle:
          '036cb496b29f281f34b52c49e5dd56f4f921db3f9353cbe254040f581662a924d7',
        type: 'Bitcoin',
      },
      {
        message:
          'Token: 0xa7960fc417cc24e9346c36cba74bf43a1fac66d1c2eacb474e56343f09e8340ac',
        signature:
          'H7+Fq/aqJsFSbKQvwalp9+Nju7+eTnvxgqV47QGOdlDWMiqJpDPSa2zYSdpeIXcqWclaNPUYtcxviCJ/StMN6vQ=',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb3Validation).toBeTruthy();
    expect(validationData.asWeb3Validation.isBitcoin).toBeTruthy();

    const data = validationData.asWeb3Validation.asBitcoin;
    expect(data.message.toHuman()).toEqual(
      'Token: 0xa7960fc417cc24e9346c36cba74bf43a1fac66d1c2eacb474e56343f09e8340ac'
    );
  });
});

describe('Substrate', () => {
  test('it works', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        type: 'Substrate',
      },
      {
        message: 'hello world',
        signature:
          '0xa6192b739c5fc196dedec99b1a0adec7182c57a79529f1b3236a6246f65ed51562dba8b1311a826cd6604129fae297c2c71da5ba69667ffda023e37c5403138b',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb3Validation).toBeTruthy();
    expect(validationData.asWeb3Validation.isSubstrate).toBeTruthy();

    const data = validationData.asWeb3Validation.asSubstrate;
    expect(data.message.toHuman()).toEqual('hello world');
    expect(data.signature.isSr25519).toBeTruthy();
    expect(data.signature.asSr25519.toHex()).toEqual(
      '0xa6192b739c5fc196dedec99b1a0adec7182c57a79529f1b3236a6246f65ed51562dba8b1311a826cd6604129fae297c2c71da5ba69667ffda023e37c5403138b'
    );
  });
});

describe('Solana', () => {
  test('it works', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: 'F8dr7WYcvkdU9YkQ84Hh8JvZwjZo81BzX2vMHbZBtPd4',
        type: 'Solana',
      },
      {
        message: 'hello world',
        signature:
          '2Z6GoT1BTNSfEj8WgtY8cozEVzwBFYqbcyCDStVs1AV3HTmQrD2k92VTiqVRC1rPcMBkaBwL9ytkbe21RS18ycjn',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb3Validation).toBeTruthy();
    expect(validationData.asWeb3Validation.isSolana).toBeTruthy();

    const data = validationData.asWeb3Validation.asSolana;

    expect(data.message.toHuman()).toEqual('hello world');
    expect(data.signature.isEd25519).toBeTruthy();
    expect(base58Encode(data.signature.asEd25519)).toEqual(
      '2Z6GoT1BTNSfEj8WgtY8cozEVzwBFYqbcyCDStVs1AV3HTmQrD2k92VTiqVRC1rPcMBkaBwL9ytkbe21RS18ycjn'
    );
    expect(base58Encode(data.signature.asEd25519)).toEqual(
      '2Z6GoT1BTNSfEj8WgtY8cozEVzwBFYqbcyCDStVs1AV3HTmQrD2k92VTiqVRC1rPcMBkaBwL9ytkbe21RS18ycjn'
    );
  });

  test('it works with hex encoded signatures', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: 'F8dr7WYcvkdU9YkQ84Hh8JvZwjZo81BzX2vMHbZBtPd4',
        type: 'Solana',
      },
      {
        message: 'hello world',
        signature:
          '0x3c5d36a76ed33902bd7b44a0fc6d194ae1599ff17064716c1e3808c4eb69aa84756084ca117641da629cb9159606655f41e5816a10f3e4df8d9019e8bb112009',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb3Validation).toBeTruthy();
    expect(validationData.asWeb3Validation.isSolana).toBeTruthy();
    expect(validationData.asWeb3Validation.asSolana.message.toHuman()).toEqual(
      'hello world'
    );
    expect(
      validationData.asWeb3Validation.asSolana.signature.isEd25519
    ).toBeTruthy();
    expect(
      validationData.asWeb3Validation.asSolana.signature.asEd25519.toHex()
    ).toEqual(
      '0x3c5d36a76ed33902bd7b44a0fc6d194ae1599ff17064716c1e3808c4eb69aa84756084ca117641da629cb9159606655f41e5816a10f3e4df8d9019e8bb112009'
    );
  });
});

describe('Twitter', () => {
  test('it works', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: 'my_twitter_handle',
        type: 'Twitter',
      },
      {
        tweetId: 'https://twitter.com/0x123/status/123',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb2Validation).toBeTruthy();
    expect(validationData.asWeb2Validation.isTwitter).toBeTruthy();
    expect(
      validationData.asWeb2Validation.asTwitter.isPublicTweet
    ).toBeTruthy();

    const data = validationData.asWeb2Validation.asTwitter.asPublicTweet;
    expect(data.tweet_id.toHuman()).toEqual(
      'https://twitter.com/0x123/status/123'
    );
  });

  test('it works oAuth2', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: 'my_twitter_handle',
        type: 'Twitter',
      },
      {
        code: 'my_twitter_code',
        state: 'my_twitter_state',
        redirectUri: 'http://test-redirect-uri',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb2Validation).toBeTruthy();
    expect(validationData.asWeb2Validation.isTwitter).toBeTruthy();
    expect(validationData.asWeb2Validation.asTwitter.isOAuth2).toBeTruthy();

    const data = validationData.asWeb2Validation.asTwitter.asOAuth2;
    expect(data.code.toHuman()).toEqual('my_twitter_code');
    expect(data.state.toHuman()).toEqual('my_twitter_state');
    expect(data.redirect_uri.toHuman()).toEqual('http://test-redirect-uri');
  });
});

describe('Discord', () => {
  test('it works', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: 'my_discord_handle',
        type: 'Discord',
      },
      {
        channelId: '123',
        messageId: '456',
        guildId: '789',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb2Validation).toBeTruthy();
    expect(validationData.asWeb2Validation.isDiscord).toBeTruthy();
    expect(
      validationData.asWeb2Validation.asDiscord.isPublicMessage
    ).toBeTruthy();

    const data = validationData.asWeb2Validation.asDiscord.asPublicMessage;

    expect(data.channel_id.toHuman()).toEqual('123');
    expect(data.message_id.toHuman()).toEqual('456');
    expect(data.guild_id.toHuman()).toEqual('789');
  });

  test('it works oAuth2', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: 'my_discord_handle',
        type: 'Discord',
      },
      {
        code: 'my_discord_code',
        redirectUri: 'http://test-redirect-uri',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb2Validation).toBeTruthy();
    expect(validationData.asWeb2Validation.isDiscord).toBeTruthy();
    expect(validationData.asWeb2Validation.asDiscord.isOAuth2).toBeTruthy();

    const data = validationData.asWeb2Validation.asDiscord.asOAuth2;
    expect(data.code.toHuman()).toEqual('my_discord_code');
    expect(data.redirect_uri.toHuman()).toEqual('http://test-redirect-uri');
  });
});

describe('Email', () => {
  test('it works', () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: 'test@my.wrong.email', // not validated
        type: 'Email',
      },
      {
        email: 'test@my.wrong.email',
        verificationCode: '123',
      }
    );

    expect(validationData).toBeDefined();
    expect(validationData.isWeb2Validation).toBeTruthy();
    expect(validationData.asWeb2Validation.isEmail).toBeTruthy();

    const data = validationData.asWeb2Validation.asEmail;

    expect(data.email.toHuman()).toEqual('test@my.wrong.email');
    expect(data.verification_code.toHuman()).toEqual('123');
  });
});
