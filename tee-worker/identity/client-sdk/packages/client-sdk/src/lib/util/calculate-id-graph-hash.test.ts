import { TypeRegistry, Metadata } from '@polkadot/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import metadataRpc from '@litentry/sidechain-api/prepare-build/litentry-sidechain-metadata.json';
import { identity } from '@litentry/parachain-api';

import { calculateIdGraphHash } from './calculate-id-graph-hash';
import { createIdGraphType } from '../type-creators/id-graph';
import { createLitentryIdentityType } from '../type-creators/litentry-identity';

const types = {
  ...identity.types, // LitentryIdentity is defined here
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

test('it deterministically hashes a graph', () => {
  const idGraphJson = [
    [
      createLitentryIdentityType(registry, {
        addressOrHandle: '0xd82308cE12A1383e8380D522Cce61d414899E199',
        type: 'Evm',
      }),
      {
        link_block: 1,
        web3networks: ['Ethereum', 'Bsc', 'Polygon', 'Arbitrum', 'Combo'],
        status: 'Active',
      },
    ],
    [
      createLitentryIdentityType(registry, {
        addressOrHandle: '5DDTRjdrwGLw7v6HJv2SoTD5oqegbTQ4sQTcKeReCekHdjZa',
        type: 'Substrate',
      }),
      {
        link_block: 6634,
        web3networks: [
          'Polkadot',
          'Kusama',
          'Litentry',
          'LitentryRococo',
          'Khala',
        ],
        status: 'Active',
      },
    ],
    [
      createLitentryIdentityType(registry, {
        addressOrHandle:
          '0x03d784811c4eb55030d238d30a2d9c08735912ebd03b99a43774d3a4ba8765038b',
        type: 'Bitcoin',
      }),
      {
        link_block: 6645,
        web3networks: ['BitcoinP2tr'],
        status: 'Active',
      },
    ],
  ];

  const idGraph = createIdGraphType(registry, randomizeIdGraph(idGraphJson));

  const hash = calculateIdGraphHash(idGraph);

  const expected = `0x47fb588c1aa067c32087d8ad0c57f8abe6d68d23e6013ad83b10740a1751c697`;

  expect(hash).toEqual(expected);
});

function randomizeIdGraph(json: Array<Array<unknown>>): Array<Array<unknown>> {
  // change the order of element in `json`
  return json.sort(() => Math.random() - 0.5);
}
