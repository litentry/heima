import type { Registry } from '@polkadot/types-codec/types';
import type { LitentryIdentity, MemberAccount } from '@litentry/parachain-api';

/**
 * Creates a MemberAccount chain type.
 *
 * @param registry - The type registry
 * @param data - Either a LitentryIdentity for public accounts, or an encrypted payload with hash for private accounts
 * @returns A MemberAccount instance
 *
 * @example
 * ```ts
 * // Public account
 * const publicAccount = createMemberAccountType(registry, {
 *   type: 'Public',
 *   identity: litentryIdentity
 * });
 *
 * // Private account
 * const privateAccount = createMemberAccountType(registry, {
 *   type: 'Private',
 *   encryptedPayload: new Uint8Array([...]),
 *   hash: '0x...'
 * });
 * ```
 */
export function createMemberAccountType(
    registry: Registry,
    data: {
        type: 'Public';
        identity: LitentryIdentity;
    } | {
        type: 'Private';
        encryptedPayload: Uint8Array;
        hash: `0x${string}`;
    }
): MemberAccount {
    if (data.type === 'Public') {
        return registry.createType('MemberAccount', { Public: data.identity });
    } else {
        return registry.createType('MemberAccount', {
            Private: [data.encryptedPayload, data.hash]
        });
    }
}
