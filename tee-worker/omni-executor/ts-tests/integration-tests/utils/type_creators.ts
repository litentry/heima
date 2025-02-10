import { HexString } from '@polkadot/util/types';
import { Codec } from '@polkadot/types-codec/types';
import {
    ApiPromise,
    CorePrimitivesIdentity,
    LitentryValidationData,
    NativeCall,
    NativeCallAuthenticated,
} from 'parachain-api';
import { Signer } from './signer';

export async function createIdentityType(
    api: ApiPromise,
    address: HexString | string,
    type: CorePrimitivesIdentity['type']
): Promise<CorePrimitivesIdentity> {
    const identity = {
        [type]: address,
    };
    return api.createType('CorePrimitivesIdentity', identity);
}

export async function createNativeCallAuthenticated(
    api: ApiPromise,
    call: [string, string],
    signer: Signer,
    mrenclave: string,
    nonce: Codec,
    params: unknown
): Promise<NativeCallAuthenticated> {
    const [variant, argType] = call;
    const nativeCall: NativeCall = api.createType('NativeCall', {
        [variant]: api.createType(argType, params),
    });
}
