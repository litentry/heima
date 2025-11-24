# Chopsticks Runtime Upgrade Investigation

## Summary

Runtime upgrades cannot be fully tested in Chopsticks due to two bugs:

1. **Scheduler doesn't execute scheduled calls** - `Dispatched` event fires but calls never run
2. **`applyAuthorizedUpgrade` fails with `FailedToExtractRuntimeVersion`** (error `0x02000000`) when `checkVersion: true`

## Test Results

| Approach | Result | Error | Notes |
|----------|--------|-------|-------|
| 2MB `remarkWithEvent` | ✅ Works | - | Proves size NOT the issue |
| Scheduler injection | ❌ Fails | Call not executed | Scheduler bug |
| `applyAuthorizedUpgrade` (checkVersion: true) | ❌ Fails | `FailedToExtractRuntimeVersion` | Version extraction bug |
| `applyAuthorizedUpgrade` (checkVersion: false) | ⚠️ Partial | - | Triggers parachain validation flow |

## Core Test Snippets

### Proof: 2MB Extrinsics Work
```typescript
// Proves Chopsticks CAN handle 2MB payloads
const wasm = fs.readFileSync('/tmp/runtime.wasm'); // 1.9MB
const wasmHex = '0x' + wasm.toString('hex');

const remarkTx = api.tx.system.remarkWithEvent(wasmHex);
await remarkTx.signAndSend(alice, ({ status, events }) => {
    // Result: ✅ ExtrinsicSuccess - 2MB extrinsic works fine!
});
```

### Bug: Version Extraction Fails
```typescript
// Authorize upgrade
await api.rpc('dev_setStorage', {
    System: {
        AuthorizedUpgrade: { codeHash: codeHash, checkVersion: true }
    }
});

// Apply upgrade
const applyTx = api.tx.system.applyAuthorizedUpgrade(wasmHex);
await applyTx.signAndSend(alice, ({ events }) => {
    // Result: ❌ ExtrinsicFailed
    // Error: Module { index: 0, error: 0x02000000 }
    // Decoded: System.FailedToExtractRuntimeVersion
    //
    // The WASM is valid (verified with subwasm), but Chopsticks
    // cannot extract the runtime version during upgrade execution.
});
```

### Workaround: Disable Version Check
```typescript
// Use checkVersion: false to bypass version extraction
await api.rpc('dev_setStorage', {
    System: {
        AuthorizedUpgrade: { codeHash: codeHash, checkVersion: false }
    }
});

const applyTx = api.tx.system.applyAuthorizedUpgrade(wasmHex);
await applyTx.signAndSend(alice, ({ events }) => {
    // Result: ✅ ExtrinsicSuccess
    // Events: parachainSystem.ValidationFunctionStored
    //
    // BUT: Runtime not immediately updated - uses parachain validation
    // flow which requires relay chain coordination and multiple blocks.
});
```

## Root Causes

### Issue 1: Scheduler Bug
- `scheduler.schedule()` and storage-injected agendas both fail
- `scheduler.Dispatched` event fires but calls don't execute
- Affects all scheduled operations, not just runtime upgrades

### Issue 2: Version Extraction Bug
- Occurs when Substrate calls `Core_version` on new WASM
- Chopsticks fails to extract version even though WASM is valid
- Verified with `subwasm info runtime.wasm` - version metadata exists
- Only happens during `applyAuthorizedUpgrade` execution

## Recommendation

**For CI testing**: Use `wasm-override` in Chopsticks config to test that new runtime launches successfully:

```yaml
# chopsticks/heima.yml
endpoint: wss://rpc.heima-parachain.heima.network
wasm-override: /path/to/new-runtime.wasm
mock-signature-host: true

import-storage:
  System:
    Account:
      - [["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
         {providers: 1, data: {free: "1000000000000000000"}}]
```

Then test that the upgraded runtime works:
```typescript
test('New runtime launches and works', async () => {
    // Connect to Chopsticks with new runtime
    const api = await ApiPromise.create({ provider: new WsProvider('ws://localhost:9944') });

    // Verify version bumped
    const version = await api.rpc.state.getRuntimeVersion();
    expect(version.specVersion.toNumber()).toBe(9251); // New version

    // Test basic operations work
    const tx = api.tx.system.remark('test');
    await tx.signAndSend(alice, ({ status }) => {
        expect(status.isInBlock).toBe(true);
    });

    // Runtime upgrade successful! ✅
});
```

This approach:
- ✅ Tests new runtime can initialize
- ✅ Tests basic operations work with new runtime
- ✅ Bypasses Chopsticks upgrade bugs
- ✅ Validates what matters: runtime compatibility

Democracy governance flow should be tested separately (if needed) up to authorization only.
