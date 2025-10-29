import { encodePacked, keccak256, encodeAbiParameters, parseAbiParameters } from "viem";

// Hyperliquid CoreWriter contract address (on HyperEVM)
export const CORE_WRITER_ADDRESS = "0x3333333333333333333333333333333333333333" as const;

/**
 * Generate a unique client order ID based on current timestamp
 */
export function generateCloid(): bigint {
    return BigInt(Date.now());
}

/**
 * Encode a limit order action for CoreWriter
 * @param assetId - The asset ID (e.g., for PURR token)
 * @param isBuy - true for buy, false for sell
 * @param limitPx - Limit price in CoreWriter units (price * 10^8)
 * @param sz - Size in CoreWriter units (size * 10^8)
 * @param cloid - Client order ID
 * @returns Encoded action bytes
 */
export function encodeLimitOrderAction(
    assetId: number,
    isBuy: boolean,
    limitPx: bigint,
    sz: bigint,
    cloid: bigint
): `0x${string}` {
    const reduceOnly = false;
    const encodedTif = 2; // Gtc (Good-Til-Cancel)

    // Encode the parameters according to the CoreWriter ABI
    const encoded = encodeAbiParameters(
        parseAbiParameters("uint32, bool, uint64, uint64, bool, uint8, uint128"),
        [assetId, isBuy, limitPx, sz, reduceOnly, encodedTif, cloid]
    );

    // Prepend version byte (0x01) and action_id (0x000001 for limit order)
    const version = "0x01";
    const actionId = "0x000001";

    return (version + actionId.slice(2) + encoded.slice(2)) as `0x${string}`;
}

/**
 * Build a spot buy order action
 * @param assetId - The spot asset ID
 * @param size - Size in CoreWriter units (size * 10^8)
 * @param price - Price in CoreWriter units (price * 10^8)
 * @param cloid - Client order ID
 * @returns Encoded action bytes
 */
export function buildSpotBuyOrder(
    assetId: number,
    size: bigint,
    price: bigint,
    cloid: bigint
): `0x${string}` {
    const isBuy = true;
    return encodeLimitOrderAction(assetId, isBuy, price, size, cloid);
}

/**
 * Build a spot sell order action
 * @param assetId - The spot asset ID
 * @param size - Size in CoreWriter units (size * 10^8)
 * @param price - Price in CoreWriter units (price * 10^8)
 * @param cloid - Client order ID
 * @returns Encoded action bytes
 */
export function buildSpotSellOrder(
    assetId: number,
    size: bigint,
    price: bigint,
    cloid: bigint
): `0x${string}` {
    const isBuy = false;
    return encodeLimitOrderAction(assetId, isBuy, price, size, cloid);
}

/**
 * Encode sendRawAction call for CoreWriter
 * @param actionData - The action data bytes
 * @returns Encoded function call
 */
export function encodeSendRawAction(actionData: `0x${string}`): `0x${string}` {
    const SEND_RAW_ACTION_SELECTOR = "0x17938e13"; // bytes4(keccak256("sendRawAction(bytes)"))

    const encoded = encodeAbiParameters(
        parseAbiParameters("bytes"),
        [actionData]
    );

    return (SEND_RAW_ACTION_SELECTOR + encoded.slice(2)) as `0x${string}`;
}

/**
 * Encode OmniAccount execute call
 * @param target - The target contract address
 * @param callData - The call data to execute
 * @returns Encoded execute call
 */
export function encodeOmniAccountExecute(
    target: `0x${string}`,
    callData: `0x${string}`
): `0x${string}` {
    const EXECUTE_SELECTOR = "0xb61d27f6"; // bytes4(keccak256("execute(address,uint256,bytes)"))

    const encoded = encodeAbiParameters(
        parseAbiParameters("address, uint256, bytes"),
        [target, BigInt(0), callData]
    );

    return (EXECUTE_SELECTOR + encoded.slice(2)) as `0x${string}`;
}

/**
 * Convert human-readable price to CoreWriter units
 * CoreWriter uses 8 decimals for prices
 * @param price - Human-readable price (e.g., 5.2)
 * @returns Price in CoreWriter units
 */
export function priceToUnits(price: number): bigint {
    return BigInt(Math.round(price * 100_000_000));
}

/**
 * Convert human-readable size to CoreWriter units
 * CoreWriter uses 8 decimals for sizes
 * @param size - Human-readable size (e.g., 100)
 * @returns Size in CoreWriter units
 */
export function sizeToUnits(size: number): bigint {
    return BigInt(Math.round(size * 100_000_000));
}

/**
 * Build complete callData for buying a spot token
 * @param assetId - The spot asset ID
 * @param size - Human-readable size
 * @param price - Human-readable price
 * @returns Complete callData for OmniAccount execute
 */
export function buildSpotBuyCallData(
    assetId: number,
    size: number,
    price: number
): `0x${string}` {
    const cloid = generateCloid();
    const sizeUnits = sizeToUnits(size);
    const priceUnits = priceToUnits(price);

    // Build the spot buy order action
    const spotBuyAction = buildSpotBuyOrder(assetId, sizeUnits, priceUnits, cloid);

    // Encode sendRawAction call
    const coreWriterCallData = encodeSendRawAction(spotBuyAction);

    // Encode OmniAccount execute call
    const callData = encodeOmniAccountExecute(CORE_WRITER_ADDRESS, coreWriterCallData);

    return callData;
}

/**
 * Get asset ID for a token by name
 * This is a simplified mapping - in production, you'd query the Hyperliquid API
 */
export async function getSpotAssetId(tokenName: string): Promise<number | null> {
    try {
        const response = await fetch("https://api.hyperliquid-testnet.xyz/info", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                type: "spotMeta",
            }),
        });

        if (!response.ok) {
            throw new Error(`Failed to fetch spot metadata: ${response.status}`);
        }

        const data = await response.json();

        // Find the token in the tokens array
        const token = data.tokens.find((t: any) => t.name === tokenName);

        if (!token) {
            console.error(`Token ${tokenName} not found in spot metadata`);
            return null;
        }

        return token.index;
    } catch (error) {
        console.error("Error fetching spot asset ID:", error);
        return null;
    }
}
