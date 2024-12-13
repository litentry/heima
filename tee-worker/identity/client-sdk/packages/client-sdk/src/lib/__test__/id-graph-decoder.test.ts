import { TypeRegistry, Metadata } from '@polkadot/types';
import { cryptoWaitReady, decodeAddress } from '@polkadot/util-crypto';

import metadataRpc from '@litentry/sidechain-api/prepare-build/litentry-sidechain-metadata.json';
import { identity, sidechain } from '@litentry/parachain-api';

import { createIdGraphType } from '../type-creators/id-graph';

const types = {
  ...identity.types, // LitentryIdentity is defined here
  ...sidechain.types, // LitentryIdentity is defined here
};

let registry: TypeRegistry;

beforeAll(async () => {
  await cryptoWaitReady();

  registry = new TypeRegistry();
  registry.register(types);
  // Needed for id_graph types
  const metadata = new Metadata(registry, metadataRpc.result as `0x${string}`);
  registry.setMetadata(metadata);
});

describe('IdGraph decoder', () => {
  // help keep backward compatibility
  it('correctly decodes an old IdGraph', async () => {
    // old id_graph before the BTC support on LitentryIdentity
    // it as an evm account and a substrate account
    const idGraph =
      '0x0804edb94607428de256494a8e7987d0f6f77e97e0084d64000008070800033a2647fe0ecd1fe0d450fd8650cd8bf3536ecac77b6e8ec2df1b5f5fc9d62931436400001c0001020304050600';

    const decoded = createIdGraphType(registry, idGraph);

    expect(decoded.isEmpty).toBeFalsy();
    expect(decoded.length).toBe(2);
    expect(decoded[0][0].isEvm).toBeTruthy();
    expect(
      decoded[0][0].asEvm.eq('0xedb94607428de256494a8e7987d0f6f77e97e008')
    ).toBeTruthy();
    expect(decoded[1][0].isSubstrate).toBeTruthy();
    expect(
      decoded[1][0].asSubstrate.eq(
        decodeAddress('5DNx1Kgis2u2SQq7EJrBdnV49PoZCxV3NqER4vV5VqjqZcat')
      )
    ).toBeTruthy();
  });
});
