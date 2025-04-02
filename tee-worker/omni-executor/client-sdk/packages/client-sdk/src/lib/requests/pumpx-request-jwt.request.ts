import type { ApiPromise } from '@polkadot/api';
import { hexToU8a } from '@polkadot/util';

import type { Identity, NativeTaskResponse, PumpxJwt } from '@heima-network/parachain-api';

import { getOmniAccountNonceWithIdentity } from '@requests/get-nonce.request';
import { OmniAuthData } from '@type-creators/omni-auth';
import { createNativeTaskType } from '@type-creators/native-task';
import { createRawTaskType } from '@type-creators/raw-task';

import { createPayloadToSign } from '@utils/create-payload-to-sign';
import { isWeb3 } from '@utils/identity';
import type { JsonRpcRequest } from '@utils/types';

import { enclave } from '@lib/enclave';

/**
 * Requests a pumpx JWT from the Enclave.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @returns {Promise<Object>} A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {Function} getPayloadToSign - A function to get the payload that needs to be signed (only for Web3 identities)
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {OmniAuthData} send.args.authData - The authentication data.
 * @returns {PumpxJwt} send.return.jwt - The generated Pumpx JWT.
 */
export async function requestPumpxJwt(
    api: ApiPromise,
    data: {
        member: Identity;
        inviteCode?: string;
        googleCode?: string;
        lang?: string;
    },
): Promise<{
    getPayloadToSign?: () => Promise<string>;
    send: (args: { authData: OmniAuthData }) => Promise<{
        jwt: PumpxJwt;
    }>;
}> {
    const { member, inviteCode, googleCode, lang } = data;

    const nonce = await getOmniAccountNonceWithIdentity(api, member);

    const { task } = createNativeTaskType(api.registry, {
        method: 'PumpxRequestJwt',
        params: {
            member,
            inviteCode,
            googleCode,
            lang,
        },
    });

    const send = async (args: {
        authData: OmniAuthData;
    }): Promise<{
        jwt: PumpxJwt;
    }> => {
        // prepare and encrypt task
        const rawTask = await createRawTaskType(api, {
            task,
            nonce,
            authData: args.authData,
        });

        // send the request to the Enclave
        const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            method: 'omni_submitNativeTask',
            params: [rawTask.toHex()],
        };

        const data = await enclave.send(request);

        const response = api.createType<NativeTaskResponse>('NativeTaskResponse', data);

        if (response.isErr) {
            throw new Error(response.asErr.toString());
        }

        const okResponse = response.asOk;
        if (!okResponse.isPumpxJwt) {
            throw new Error('Unexpected response type');
        }

        const jwt = okResponse.asPumpxJwt;

        return { jwt };
    };

    if (isWeb3(member)) {
        const getPayloadToSign = async () => {
            const mrEnclave = await enclave.getMrEnclave(api);
            return createPayloadToSign({
                who: member,
                task,
                nonce,
                mrEnclave: hexToU8a(mrEnclave),
            });
        }

        return {
            getPayloadToSign,
            send,
        };
    }

    return { send };
}
