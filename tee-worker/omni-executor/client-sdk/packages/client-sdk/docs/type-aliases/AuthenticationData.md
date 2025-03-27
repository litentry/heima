[**@heima/client-sdk**](../README.md)

***

[@heima/client-sdk](../README.md) / AuthenticationData

# Type Alias: AuthenticationData

> **AuthenticationData** = \{ `type`: `"Email"`; `verificationCode`: `string`; \} \| \{ `signature`: `string`; `signer`: `Identity`; `type`: `"Web3"`; \} \| \{ `token`: `string`; `type`: `"AuthToken"`; \} \| \{ `data`: `OAuth2DataType`; `type`: `"OAuth2"`; \}

Defined in: [type-creators/authentication.ts:6](https://github.com/litentry/heima/blob/dev/type-creators/authentication.ts#L6)
