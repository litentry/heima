# Omni JSON-RPC Client

简洁的 TypeScript JSON-RPC 客户端，使用单例模式。

## 特性

- 🚀 **类型安全**: TypeScript 类型定义
- 🔐 **按需认证**: 根据方法需要传入 token
- 🛡️ **错误处理**: 专门的错误类型
- 🎯 **单例模式**: 全局共享实例
- ⚡ **简洁 API**: 最小化代码

## 快速开始

```typescript
import { omniApi } from './utils/omni-api.js';

// 需要 token 的方法
const result1 = await omniApi.addWallet('your_jwt_token');

// 不需要 token 的方法
const result2 = await omniApi.getHealth();
```

## 使用方式

### 1. 需要认证的方法

```typescript
import { omniApi } from './utils/omni-api.js';

const token = 'your_jwt_token';

// 添加钱包
const wallet = await omniApi.addWallet(token);

// 导出钱包
const export = await omniApi.exportWallet(token);

// 提交订单
const order = await omniApi.submitSwapOrder({ amount: '1000' }, token);
```

### 2. 不需要认证的方法

```typescript
// 健康检查
const health = await omniApi.getHealth();
```

### 3. 通用方法调用

```typescript
// 不需要 token 的方法
const result1 = await omniApi.call('omni_getHealth', []);

// 需要 token 的方法
const result2 = await omniApi.call('omni_addWallet', [], token);
```

## API 方法

| 方法 | 参数 | 描述 | 需要 Token |
|------|------|------|-----------|
| `addWallet(token)` | `token: string` | 添加钱包 | ✅ |
| `exportWallet(token)` | `token: string` | 导出钱包 | ✅ |
| `getHealth()` | - | 获取健康状态 | ❌ |
| `submitSwapOrder(params, token)` | `params: any, token: string` | 提交交换订单 | ✅ |
| `transferWithdraw(params, token)` | `params: any, token: string` | 转账提现 | ✅ |
| `signLimitOrder(params, token)` | `params: any[], token: string` | 签名限价订单 | ✅ |
| `notifyLimitOrderResult(params, token)` | `params: any[], token: string` | 通知限价订单结果 | ✅ |
| `call(method, params?, token?)` | `method: string, params?: any[], token?: string` | 通用方法调用 | 按需 |

## 完整示例

```typescript
import { omniApi } from './utils/omni-api.js';

async function example() {
  const token = 'your_jwt_token';

  try {
    // 健康检查（不需要 token）
    const health = await omniApi.getHealth();
    console.log('状态:', health);

    // 添加钱包（需要 token）
    const wallet = await omniApi.addWallet(token);
    console.log('钱包:', wallet);

    // 提交交换订单（需要 token）
    const swap = await omniApi.submitSwapOrder({
      fromToken: 'USDT',
      toToken: 'ETH',
      amount: '1000'
    }, token);
    console.log('交换:', swap);

    // 通用调用
    const custom = await omniApi.call('omni_customMethod', ['param'], token);
    console.log('自定义:', custom);

  } catch (error) {
    console.error('错误:', error);
  }
}
```

## 错误处理

```typescript
import { JsonRpcError } from './utils/jsonrpc-client.js';

try {
  const result = await omniApi.addWallet(token);
} catch (error) {
  if (error instanceof JsonRpcError) {
    console.error('RPC 错误:', error.code, error.message);
  } else {
    console.error('网络错误:', error.message);
  }
}
```

## 运行测试

```bash
pnpm test
```

## 原理

- **单例模式**: 全局只有一个客户端实例
- **固定端点**: `https://staging-dex-worker.heima.network/`
- **按需认证**: 只有需要的方法才传入 token
- **自动错误处理**: JSON-RPC 错误自动转换为 `JsonRpcError` 