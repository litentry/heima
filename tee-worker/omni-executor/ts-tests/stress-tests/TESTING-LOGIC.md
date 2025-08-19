# 🔍 Omni UserLogin 压力测试逻辑详解

## 📋 测试服务概述

本压力测试系统专门针对 **Omni Executor** 的 `OmniUserLogin` 接口进行压力测试，该服务运行在 **2100 端口**。

### 🎯 测试目标接口
- **主要接口**: `omni_userLogin` - Web3 用户登录流程
- **辅助接口**: `omni_getWeb3SignInMessage` - 获取签名消息
- **服务地址**: `http://localhost:2100/` (可配置)

## 🔄 完整测试流程

### 1. **钱包预生成阶段**
```typescript
// 在测试开始前，预生成 100 个测试钱包以提升性能
initializeWalletPool(size = 100): void {
  for (let i = 0; i < size; i++) {
    const privateKey = generatePrivateKey();
    const account = privateKeyToAccount(privateKey);
    this.walletPool.push({
      privateKey,
      address: account.address
    });
  }
}
```

### 2. **单次登录测试流程**
每个并发用户执行以下步骤：

#### Step 1: 获取 Web3 签名消息
```typescript
const messageResponse = await rpcClient.call('omni_getWeb3SignInMessage', [{
  client_id: 'wildmeta',
  omni_account: wallet.address.toLowerCase()
}]);
```

#### Step 2: 签名消息
```typescript
const messageString = JSON.stringify(messageResponse);
const signature = await signMessage({ 
  message: messageString, 
  privateKey: wallet.privateKey
});
```

#### Step 3: 执行登录
```typescript
const loginResponse = await rpcClient.call('omni_userLogin', [{
  user_id: {
    type: 'evm',
    value: wallet.address,
  },
  user_auth: {
    type: 'evm',
    value: signature,
  },
  client_id: 'wildmeta',
  client_auth: {
    type: 'wildmeta',
    value: {
      google_code: '',
      invite_code: '',
    },
  },
}]);
```

#### Step 4: 验证响应
```typescript
const success = loginResponse?.backend_response?.code === 10000;
```

## 🛡️ 错误处理机制

### ✅ 核心原则：**绝不中断测试**

测试系统采用**多层错误捕获**机制，确保任何错误都不会导致测试中断：

### 1. **分步错误捕获**
```typescript
async performLogin(): Promise<LoginResult> {
  let stepError = '';
  
  try {
    // Step 1: Get message (独立捕获)
    try {
      messageResponse = await rpcClient.call('omni_getWeb3SignInMessage', [...]);
    } catch (error) {
      stepError = `Step 1 - Get message failed: ${error.message}`;
      throw new Error(stepError);
    }
    
    // Step 2: Sign message (独立捕获)
    try {
      signature = await signMessage(...);
    } catch (error) {
      stepError = `Step 2 - Sign message failed: ${error.message}`;
      throw new Error(stepError);
    }
    
    // Step 3: Login (独立捕获)
    try {
      loginResponse = await rpcClient.call('omni_userLogin', [...]);
    } catch (error) {
      stepError = `Step 3 - Login failed: ${error.message}`;
      throw new Error(stepError);
    }
    
  } catch (error) {
    // ✅ 始终返回结果，绝不抛出异常
    return {
      success: false,
      responseTime: Date.now() - startTime,
      requestSize,
      responseSize,
      error: stepError || error.message
    };
  }
}
```

### 2. **工作线程错误隔离**
```typescript
private async createWorker(workerId: number, endTime: number): Promise<void> {
  while (this.running && Date.now() < endTime) {
    // ✅ 每个请求都被安全包装，绝不会因错误停止循环
    const result = await this.executeRequestSafely(workerId);
    
    // ✅ 成功和失败都记录到结果中
    if (this.session) {
      this.session.results.push(result);
    }
    
    if (result.success) {
      this.emit('requestCompleted', result);
    } else {
      this.emit('requestFailed', result);  // 错误事件，但不停止
    }
    
    // ✅ 继续下一个请求
    await new Promise(resolve => setTimeout(resolve, 10));
  }
}
```

### 3. **错误分类和记录**
系统详细记录每种错误类型：

```typescript
// 网络错误
"Step 1 - Get message failed: fetch failed"
"Step 3 - Login failed: HTTP 500: Internal Server Error"

// 业务逻辑错误  
"Login validation failed: Invalid credentials"
"Login validation failed: User not found"

// 系统错误
"Step 2 - Sign message failed: Invalid private key"
"Unexpected error in performLogin: Memory allocation failed"
```

## 📊 测试数据收集

### 🎯 每个请求记录的数据
```typescript
interface RequestResult {
  timestamp: number;          // 请求时间戳
  requestId: string;          // 唯一请求 ID
  method: 'POST';            // 请求方法
  url: string;               // 目标 URL
  statusCode: number;        // HTTP 状态码 (成功=200, 失败=500/0)
  responseTime: number;      // 响应时间 (毫秒)
  success: boolean;          // 是否成功
  error?: string;            // 错误信息 (如果失败)
  requestSize: number;       // 请求大小 (字节)
  responseSize: number;      // 响应大小 (字节)
}
```

### 📈 聚合统计指标
```typescript
// 响应时间分布
responseTime: {
  average: number;     // 平均响应时间
  min: number;         // 最小响应时间
  max: number;         // 最大响应时间
  p50: number;         // 50th 百分位
  p90: number;         // 90th 百分位
  p95: number;         // 95th 百分位
  p99: number;         // 99th 百分位
}

// 吞吐量指标
throughput: {
  rps: number;                // 每秒请求数
  tps: number;                // 每秒成功事务数
  dataTransferRate: number;   // 数据传输速率
  peakConcurrency: number;    // 峰值并发数
}

// 错误率分析
errorRate: {
  total: number;              // 总错误率 %
  http4xx: number;            // 4xx 错误率 %
  http5xx: number;            // 5xx 错误率 %
  timeout: number;            // 超时错误率 %
  connection: number;         // 连接错误率 %
}
```

## 🚀 并发控制策略

### 1. **渐进式加载 (Ramp-up)**
```typescript
// 示例：10 秒内从 0 逐步增加到 50 并发
const rampUpIntervalMs = (10 * 1000) / 50;  // 每 200ms 启动一个用户

for (let i = 0; i < 50; i++) {
  setTimeout(() => {
    if (this.running) {
      const worker = this.createWorker(i, testEndTime);
      this.workers.push(worker);
    }
  }, i * rampUpIntervalMs);
}
```

### 2. **稳定期测试**
- 所有用户同时运行，维持目标并发数
- 每个用户独立循环发送请求
- 实时监控系统资源使用情况

### 3. **渐进式卸载 (Ramp-down)**
- 测试结束前逐步减少并发用户
- 等待所有进行中的请求完成
- 确保数据完整性

## 🌐 分布式测试协调

### 1. **机器标识**
```typescript
private generateMachineId(): string {
  const hostName = hostname();
  const timestamp = Date.now();
  const random = Math.random().toString(36).substr(2, 6);
  return `${hostName}-${timestamp}-${random}`;
}
```

### 2. **结果文件格式**
```json
{
  "machineId": "server1-1703123456789-abc123",
  "hostname": "server1",
  "testConfig": {
    "testName": "High Load Test",
    "targetUrl": "http://target:2100/",
    "duration": 300,
    "concurrency": 50
  },
  "startTime": 1703123456789,
  "endTime": 1703123756789,
  "results": [...],  // 所有请求结果
  "summary": {...}   // 统计摘要
}
```

### 3. **结果聚合逻辑**
```typescript
// 合并多台机器的结果
const allResults = machineResults.flatMap(mr => mr.results);
const totalRequests = allResults.length;
const totalDuration = (latestEnd - earliestStart) / 1000;
const combinedThroughput = totalRequests / totalDuration;
```

## 🎯 性能基准和阈值

### 📊 生产级性能目标
```typescript
const PRODUCTION_THRESHOLDS = {
  responseTime: {
    target: 300,      // 300ms 目标响应时间
    acceptable: 500,  // 500ms 可接受
    critical: 1000,   // 1s 关键警告线
  },
  throughput: {
    target: 200,      // 200 RPS 目标
    minimum: 50,      // 50 RPS 最低要求
  },
  errorRate: {
    warning: 0.5,     // 0.5% 警告线
    critical: 2,      // 2% 关键警告线
  }
};
```

### 🔍 自动性能分析
1. **最大可持续 RPS 计算**
2. **瓶颈识别算法**
3. **风险等级评估**
4. **优化建议生成**

## 🛠️ 关键特性总结

### ✅ **绝不中断的测试逻辑**
- 所有错误都被捕获并记录
- 测试线程永不因异常停止
- 完整的错误分类和统计

### 📊 **完整的数据收集**
- 每个请求的详细指标
- 多维度的性能统计
- 实时和历史数据分析

### 🌐 **分布式架构支持**
- 跨机器协调测试
- 自动结果聚合
- 统一的性能报告

### 🎯 **专业的性能评估**
- 基于生产标准的阈值
- 自动化的瓶颈分析
- 实用的优化建议

这个测试系统确保了在任何情况下（网络故障、服务异常、资源不足等）都能完整执行测试并收集有价值的性能数据，为系统优化提供可靠的依据。