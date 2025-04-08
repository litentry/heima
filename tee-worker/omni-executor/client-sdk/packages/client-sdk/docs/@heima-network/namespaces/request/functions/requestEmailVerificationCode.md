[**@heima-network/client-sdk**](../../../../README.md)

***

[@heima-network/client-sdk](../../../../README.md) / [request](../README.md) / requestEmailVerificationCode

# Function: requestEmailVerificationCode()

> **requestEmailVerificationCode**(`args`, `enclaveInstance`?): `Promise`\<`void`\>

Defined in: [requests/request-email-verification-code.request.ts:16](https://github.com/litentry/heima/blob/dev/requests/request-email-verification-code.request.ts#L16)

Requests an email verification code to be sent to the specified email address.

## Parameters

### args

The args object containing the following properties:

#### email

`string`

The email address to send the verification code to.

### enclaveInstance?

[`Enclave`](../../../../classes/Enclave.md) = `enclave`

The enclave instance to use for the request.

## Returns

`Promise`\<`void`\>

A promise that resolves when the request is sent.

## Throws

Throws an error if the email is empty.
