import { u8aToHex, u8aConcat } from '@polkadot/util';
import { xxhashAsU8a } from '@polkadot/util-crypto';
import { StorageEntryMetadataV14, SiLookupTypeId, StorageHasherV14 } from '@polkadot/types/interfaces';
import { sendRequest } from '../call';
import { blake2128Concat, twox64Concat, identity, createJsonRpcRequest, nextRequestId } from '../helpers';
import type { IntegrationTestContext } from '../common-types';
import type { PalletIdentityManagementTeeIdentityContext } from '@heima-network/api-argument/sidechain';
import type { CorePrimitivesIdentity } from '@heima-network/api-argument/identity';
import type { HexString } from '@polkadot/util/types';
import type { Metadata } from '@polkadot/types';

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

import * as base58 from 'micro-base58';

//sidechain storage utils
export function buildStorageEntry(metadata: Metadata, prefix: string, method: string): StorageEntryMetadataV14 | null {
    for (const pallet of metadata.asV14.pallets) {
        if (pallet.name.toString() == prefix) {
            const storage = pallet.storage.unwrap();

            for (const item of storage.items) {
                if (item.name.toString() == method) {
                    return item;
                }
            }
        }
    }
    return null;
}

export function buildStorageKey(
    metadata: Metadata,
    prefix: string,
    method: string,
    keyTypeId?: SiLookupTypeId,
    hashers?: Array<StorageHasherV14>,
    input?: Array<unknown>
): Uint8Array {
    let storageKey = u8aConcat(xxhashAsU8a(prefix, 128), xxhashAsU8a(method, 128));
    if (keyTypeId && hashers && input) {
        const keyTypeIds =
            hashers.length === 1 ? [keyTypeId] : metadata.registry.lookup.getSiType(keyTypeId).def.asTuple;
        for (let i = 0; i < keyTypeIds.length; i++) {
            const theKeyTypeId = keyTypeIds[i];
            const theHasher = hashers[i].toString();
            const theKeyItem = input[i];
            // get the scale encoded input data by encoding the input
            const theKeyType = metadata.registry.createLookupType(theKeyTypeId);
            const theKeyItemEncoded = metadata.registry.createType(theKeyType, theKeyItem).toU8a();
            // apply hasher
            let theKeyItemAppliedHasher;
            if (theHasher == 'Blake2_128Concat') {
                theKeyItemAppliedHasher = blake2128Concat(theKeyItemEncoded);
            } else if (theHasher == 'Twox64Concat') {
                theKeyItemAppliedHasher = twox64Concat(theKeyItemEncoded);
            } else if (theHasher == 'Identity') {
                theKeyItemAppliedHasher = identity(theKeyItemEncoded);
            } else {
                throw new Error(`The hasher ${theHasher} is not support.`);
            }
            storageKey = u8aConcat(storageKey, theKeyItemAppliedHasher);
        }
    }
    return storageKey;
}
export async function buildStorageHelper(
    metadata: Metadata,
    prefix: string,
    method: string,
    ...input: Array<unknown>
): Promise<string | null> {
    const storageEntry = buildStorageEntry(metadata, prefix, method);
    if (!storageEntry) {
        throw new Error('Can not find the storage entry from metadata');
    }
    let storageKey;

    if (storageEntry.type.isPlain) {
        storageKey = buildStorageKey(metadata, prefix, method);
    } else if (storageEntry.type.isMap) {
        const { hashers, key } = storageEntry.type.asMap;
        if (input.length != hashers.length) {
            throw new Error('The `input` param is not correct');
        }
        storageKey = buildStorageKey(metadata, prefix, method, key, hashers, input);
    } else {
        throw new Error('Only support plain and map type');
    }
    console.debug(`storage key: ${u8aToHex(storageKey)}`);
    return u8aToHex(storageKey);
}

export async function checkIdGraph(
    context: IntegrationTestContext,
    pallet: string,
    method: string,
    subject: CorePrimitivesIdentity,
    identity: HexString
): Promise<PalletIdentityManagementTeeIdentityContext> {
    await sleep(6000);
    const storageKey = await buildStorageHelper(context.sidechainMetaData, pallet, method, subject, identity);

    const base58mrEnclave = base58.encode(Buffer.from(context.mrEnclave.slice(2), 'hex'));

    const request = createJsonRpcRequest('state_getStorage', [base58mrEnclave, storageKey], nextRequestId(context));
    const resp = await sendRequest(context.tee, request, context.api);
    const idGraph = context.sidechainRegistry.createType('PalletIdentityManagementTeeIdentityContext', resp.value);
    return idGraph;
}
