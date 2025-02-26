import type { ApiPromise } from '@polkadot/api';
import type { AccountId32 } from '@polkadot/types/interfaces';
import type { Option } from '@polkadot/types-codec';
import type { CorePrimitivesTeebagTypesWorkerType, CorePrimitivesTeebagTypesEnclave } from '@litentry/parachain-api';

/**
 * Return the Enclave registry information of the latest registered TEE worker.
 */
export async function getLastRegisteredEnclave(
  api: ApiPromise,
  workerType: CorePrimitivesTeebagTypesWorkerType['type'] = 'OmniExecutor',
): Promise<{
  account: AccountId32;
  enclave: CorePrimitivesTeebagTypesEnclave;
}> {
  const identifiers = (await api.query.teebag.enclaveIdentifier(workerType)) as unknown as AccountId32[];
  const latestEnclaveId = identifiers[identifiers.length - 1];

  if (latestEnclaveId.isEmpty) {
    throw new Error('[omni-sdk] No Enclave identifiers found');
  }

  const maybeEnclave = (await api.query.teebag.enclaveRegistry(latestEnclaveId)) as unknown as Option<any>;

  if (maybeEnclave.isEmpty) {
    throw new Error(`[omni-sdk] No Enclave Worker with ${latestEnclaveId.toHuman()} found`);
  }

  const enclave = maybeEnclave.unwrap() as CorePrimitivesTeebagTypesEnclave;

  return { enclave, account: latestEnclaveId };
}
