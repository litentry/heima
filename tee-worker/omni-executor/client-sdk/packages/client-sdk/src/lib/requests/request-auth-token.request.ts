import type { ApiPromise } from '@polkadot/api';
import { hexToU8a } from '@polkadot/util';

import type { Identity, NativeOperationResponse } from '@heima/parachain-api';

import { getOmniAccountNonceWithIdentity } from '@requests/get-nonce.request';
import { AuthenticationData } from '@type-creators/authentication';
import { createNativeCallType } from '@type-creators/native-call';
import { createCallRequestType } from '@type-creators/request';

import { createPayloadToSign } from '@utils/create-payload-to-sign';
import { isWeb3 } from '@utils/identity';
import type { JsonRpcRequest } from '@utils/types';

import { enclave } from '@lib/enclave';

/**
 * Requests an authentication token from the Enclave.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @returns {Promise<Object>} A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args The arguments required to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function requestAuthToken(
    api: ApiPromise,
    data: {
        member: Identity;
    },
): Promise<{
    payloadToSign?: string;
    send: (args: { authentication: AuthenticationData }) => Promise<{
        token: string;
    }>;
}> {
    const { member } = data;

    const [nonce, mrEnclave] = await Promise.all([
        getOmniAccountNonceWithIdentity(api, member),
        enclave.getMrEnclave(api),
    ]);

    const { operation } = createNativeCallType(api.registry, {
        method: 'request_auth_token',
        params: {
            member,
        },
    });

    const mrEnclaveU8 = hexToU8a(mrEnclave);

    const send = async (args: {
        authentication: AuthenticationData;
    }): Promise<{
        token: string;
    }> => {
        // prepare and encrypt request
        const request = await createCallRequestType(api, {
            authentication: args.authentication,
            operation,
            nonce,
            mrEnclave: mrEnclaveU8,
        });

        // send the request to the Enclave
        const rpcRequest: JsonRpcRequest = {
            jsonrpc: '2.0',
            method: 'native_submitCallAesRequest',
            params: [request.toHex()],
        };

        const data = await enclave.send(rpcRequest);

        const result = api.createType<NativeOperationResponse>('NativeOperationResponse', data);

        if (result.isErr) {
            throw new Error(result.asErr.toString());
        }

        if (!result.asOk.isCallResponse) {
            throw new Error('Unexpected response type');
        }

        const callResponse = result.asOk.asCallResponse;
        if (!callResponse.isAuthToken) {
            throw new Error('Unexpected call response type');
        }

        const token = callResponse.asAuthToken.toString();

        return { token };
    };

    if (isWeb3(member)) {
        const payloadToSign = createPayloadToSign({
            who: member,
            operation,
            nonce,
            mrEnclave: mrEnclaveU8,
        });

        return {
            payloadToSign,
            send,
        };
    }

    return { send };
}
