[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / getOAuth2GoogleAuthorizationUrl

# Function: getOAuth2GoogleAuthorizationUrl()

> **getOAuth2GoogleAuthorizationUrl**(`args`): `Promise`\<`string`\>

Defined in: [requests/get-oauth2-google-authorization-url.request.ts:16](https://github.com/litentry/heima/blob/dev/requests/get-oauth2-google-authorization-url.request.ts#L16)

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

## Returns

`Promise`\<`string`\>

A Promise that resolves to the Google OAuth2 authorization URL

## Throws

If googleAccount or redirectUri is empty or undefined
