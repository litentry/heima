import { sha256 } from 'js-sha256';
import { Address } from 'viem';
import { ClientId } from './constants';
import { privateKeyToAccount } from 'viem/accounts';

export function calculateOmniAccount(
    address: Address | string,
    clientId: string = ClientId.Wildmeta,
    identityType: 'evm' | 'solana' = 'evm'
): `0x${string}` {
    // Create input array for hashing - order is important: clientId, oaType, sender
    const inputs: Uint8Array[] = [];

    // First: clientId as raw bytes (not padded to 32)
    const clientIdBytes = new TextEncoder().encode(clientId);
    inputs.push(clientIdBytes);

    // Second: identity type ("evm" or "solana")
    inputs.push(new TextEncoder().encode(identityType));

    // Third: address
    if (identityType === 'evm') {
        // For EVM addresses, remove 0x prefix and convert from hex
        const addressHex = address.slice(2).toLowerCase();
        const addressBytes = new Uint8Array(20); // EVM addresses are 20 bytes
        for (let i = 0; i < addressHex.length; i += 2) {
            addressBytes[i / 2] = parseInt(addressHex.substring(i, i + 2), 16);
        }
        inputs.push(addressBytes);
    } else {
        // For Solana, handle as appropriate
        // This is a simplified version - real implementation would handle base58 for Solana
        // TODO
        inputs.push(new TextEncoder().encode(address));
    }

    const totalLength = inputs.reduce((sum, arr) => sum + arr.length, 0);
    const combined = new Uint8Array(totalLength);
    let offset = 0;
    for (const input of inputs) {
        combined.set(input, offset);
        offset += input.length;
    }

    // Calculate SHA256 hash
    const hash = sha256.array(combined);

    // Convert to hex string
    const result = `0x${hash.map((b) => b.toString(16).padStart(2, '0')).join('')}`;

    // console.log("OmniAccount calculation:", {
    // 	address,
    // 	clientId,
    // 	identityType,
    // 	clientIdBytes: Array.from(inputs[0])
    // 		.map((b) => b.toString(16).padStart(2, "0"))
    // 		.join(""),
    // 	identityTypeBytes: Array.from(inputs[1])
    // 		.map((b) => b.toString(16).padStart(2, "0"))
    // 		.join(""),
    // 	addressBytes: Array.from(inputs[2])
    // 		.map((b) => b.toString(16).padStart(2, "0"))
    // 		.join(""),
    // 	combined: Array.from(combined)
    // 		.map((b) => b.toString(16).padStart(2, "0"))
    // 		.join(""),
    // 	result,
    // });

    return result as `0x${string}`;
}

export interface EvmWallet {
    address: Address;
    privateKey: `0x${string}`;
}

export function randomEvmWallet(): EvmWallet {
    const privateKeyBytes = new Uint8Array(32);
    crypto.getRandomValues(privateKeyBytes);

    const privateKey = `0x${Array.from(privateKeyBytes)
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('')}` as `0x${string}`;

    const account = privateKeyToAccount(privateKey);

    return {
        address: account.address,
        privateKey: privateKey,
    };
}
