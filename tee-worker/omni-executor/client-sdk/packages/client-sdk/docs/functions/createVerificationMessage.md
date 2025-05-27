[**@heima-network/client-sdk**](../README.md)

***

[@heima-network/client-sdk](../README.md) / createVerificationMessage

# Function: createVerificationMessage()

> **createVerificationMessage**(`registry`, `args`): `` `0x${string}` ``

Defined in: [utils/create-verification-message.ts:25](https://github.com/litentry/heima/blob/dev/utils/create-verification-message.ts#L25)

Creates a verification message for adding a new member to an Omni account

## Parameters

### registry

`Registry`

The Polkadot registry instance used for type encoding

### args

The arguments object containing:

#### member

`Identity`

The identity of the existing member

#### memberToAdd

`Identity`

The identity of the new member to be added

#### omniAccountNonce

`Index`

The current nonce of the Omni account

## Returns

`` `0x${string}` ``

A hex-encoded string containing the verification message prefixed with "Token: "

The verification message is created by:
1. Converting the nonce to U64 format
2. Concatenating the encoded nonce, member identity, and new member identity
3. Computing the BLAKE2 hash of the concatenated data
4. Prefixing the hash with "Token: " and converting to hex
