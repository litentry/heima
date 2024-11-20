import { hexToU8a, u8aConcat } from '@polkadot/util';
import { blake2AsHex, randomAsHex } from '@polkadot/util-crypto';

import { enclave } from '../enclave';
import { createTrustedCallType } from '../type-creators/trusted-call';
import { createRequestType } from '../type-creators/request';
import { createPayloadToSign } from '../util/create-payload-to-sign';
import { type IdGraph, createIdGraphType } from '../type-creators/id-graph';

import * as shieldingKeyUtils from '../util/shielding-key';

import type {
  LitentryIdentity,
  LitentryValidationData,
  Web3Network,
  WorkerRpcReturnValue,
} from '@litentry/parachain-api';
import type { JsonRpcRequest } from '../util/types';
import type { ApiPromise } from '@polkadot/api';

import { getEnclaveNonce } from './get-enclave-nonce';

/**
 * Generates the challenge code to link an identity.
 *
 * The challenge code is calculated from:
 *
 * ```
 * blake2_256(<enclaveNonce> + <primaryAccount> + <identityToLink>)
 * ```
 *
 * When `options.prettify` is set to true, the challenge code will be prefixed
 * with `Token: ` for utf-8 signatures support.
 * Otherwise, it will be returned as a hex string.
 *
 * `options.prettify` feature is web3-specific. Ignored for web2.
 *
 */
export async function createChallengeCode(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  args: {
    /** The user's account. Use `createCorePrimitivesIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** Identity to be linked. Use `createCorePrimitivesIdentityType` helper to create this struct */
    identity: LitentryIdentity;
    /** */
  },
  options: { prettify?: boolean } = { prettify: false }
): Promise<string> {
  const { who, identity } = args;
  const nonce = await getEnclaveNonce(api, { who });

  const message = u8aConcat(nonce.toU8a(), who.toU8a(), identity.toU8a());
  const challengeCode = blake2AsHex(message, 256);

  const isWeb2 =
    identity.isTwitter ||
    identity.isDiscord ||
    identity.isGithub ||
    identity.isEmail;

  // support prettify for web3 identities only
  if (!isWeb2 && options?.prettify) {
    return `Token: ${challengeCode}`;
  }

  // bitcoin wallets need to sign the challenge code without the 0x prefix.
  return args.identity.isBitcoin ? challengeCode.substring(2) : challengeCode;
}

/**
 * Link an identity to the user's account.
 */
export async function linkIdentity(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The prime identity. Use `createCorePrimitivesIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** Identity to be linked. Use `createCorePrimitivesIdentityType` helper to create this struct */
    identity: LitentryIdentity;
    /** The ownership proof. Use `createLitentryValidationDataType` helper to create this struct */
    validation: LitentryValidationData;
    /** The networks to link the identity to, for web3 accounts */
    networks: Array<Web3Network['type']>;
  }
): Promise<{
  payloadToSign: string;
  txHash: string;
  send: (args: { signedPayload: string }) => Promise<{
    mutatedIdentities: IdGraph;
    idGraphHash: `0x${string}`;
    response: WorkerRpcReturnValue;
    txHash: string;
  }>;
}> {
  const { who, identity, validation, networks } = data;

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);
  const nonce = await getEnclaveNonce(api, { who });

  const txHash = randomAsHex();

  const { call, key } = await createTrustedCallType(api.registry, {
    method: 'link_identity',
    params: {
      who,
      identity,
      validation,
      networks,
      hash: txHash,
    },
  });

  const payloadToSign = createPayloadToSign({
    who,
    call,
    nonce,
    shard: shardU8,
  });

  const send = async (args: {
    signedPayload: string;
  }): Promise<{
    response: WorkerRpcReturnValue;
    mutatedIdentities: IdGraph;
    idGraphHash: `0x${string}`;
    txHash: string;
  }> => {
    // prepare and encrypt request
    const request = await createRequestType(api, {
      sender: who,
      authentication: args.signedPayload,
      call,
      nonce,
      shard: shardU8,
    });

    // send the request to the Enclave
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'author_submitAndWatchAesRequest',
      params: [request.toHex()],
      // note: it should be number and less than 10
      id: 1,
    };
    const [workerResponse] = await enclave.send(api, rpcRequest);

    // process the encrypted response
    const { mutated_id_graph, id_graph_hash } = api.createType(
      'LinkIdentityResult',
      workerResponse.value
    );

    const { cleartext: decryptedIdGraphRaw } = await shieldingKeyUtils.decrypt(
      {
        ciphertext: mutated_id_graph.ciphertext,
        nonce: mutated_id_graph.nonce,
      },
      key
    );

    const mutatedIdentities = createIdGraphType(
      api.registry,
      decryptedIdGraphRaw
    );

    return {
      mutatedIdentities,
      idGraphHash: id_graph_hash.toHex(),
      txHash,
      response: workerResponse,
    };
  };

  return {
    txHash,
    payloadToSign,
    send,
  };
}
