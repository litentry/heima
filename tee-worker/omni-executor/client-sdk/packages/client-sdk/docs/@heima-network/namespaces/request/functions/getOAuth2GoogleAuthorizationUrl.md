[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / getOAuth2GoogleAuthorizationUrl

# Function: getOAuth2GoogleAuthorizationUrl()

> **getOAuth2GoogleAuthorizationUrl**(`args`, `enclaveInstance`): `Promise`\<`string`\>

Defined in: [requests/get-oauth2-google-authorization-url.request.ts:17](https://github.com/litentry/heima/blob/dev/requests/get-oauth2-google-authorization-url.request.ts#L17)

Generates an OAuth2 authorization URL for Google authentication.

## Parameters

### args

The arguments object

#### googleAccount

`string`

The Google account email address

#### redirectUri

`string`

The URI where Google will redirect after authentication

### enclaveInstance

[`Enclave`](../../../../classes/Enclave.md) = `enclave`

The enclave instance use to interact with Enclave.

## Returns

`Promise`\<`string`\>

A Promise that resolves to the Google OAuth2 authorization URL

## Throws

If googleAccount or redirectUri is empty or undefined
