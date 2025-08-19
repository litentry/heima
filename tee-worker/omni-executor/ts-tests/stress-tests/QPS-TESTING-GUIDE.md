# 📈 QPS 逐步增加测试指南

## 🎯 新增功能概述

已成功为 Omni Executor 压力测试系统添加了三个新的测试接口，并实现了 QPS 逐步增加测试功能：

### 🆕 新增测试接口

#### 1. **omni_getShieldingKey** 
```typescript
// 获取屏蔽密钥 - 无需认证
const response = await rpcClient.call('omni_getShieldingKey', []);
// 验证响应包含 n 和 e 字段
```

#### 2. **omni_addWallet**
```typescript  
// 添加钱包 - 需要认证 token
const response = await rpcClient.call('omni_addWallet', [], authToken);
// 验证 backend_response.code === 10000
```

#### 3. **混合测试模式**
- **mixed**: 随机调用所有 4 个接口（均匀分布）
- **weighted_mixed**: 加权调用（40% UserLogin, 30% ShieldingKey, 20% GetNextIntentId, 10% AddWallet）

### 📊 QPS 逐步增加测试

全新的 QPS Ramp Engine 能够：
- ✅ **逐步增加负载**: 从低 QPS 开始，逐步增加到指定上限
- ✅ **自动识别瓶颈**: 检测系统性能下降点和稳定性边界
- ✅ **详细数据记录**: 每个 QPS 级别的完整性能数据
- ✅ **JSON 格式存储**: 所有测试结果保存为结构化数据

## 🚀 快速使用

### 基础 QPS 测试
```bash
# 默认 UserLogin QPS 测试 (1-50 QPS, 5 QPS 步长)
npm run qps-test run

# 自定义 QPS 范围和步长
npm run qps-test run --start-qps 2 --max-qps 100 --step-size 10 --step-duration 30
```

### 特定接口测试
```bash
# ShieldingKey QPS 测试
npm run qps-test api shielding-key

# AddWallet QPS 测试 
npm run qps-test api add-wallet

# 混合 API 测试
npm run qps-test api mixed
npm run qps-test api weighted-mixed
```

### 快速测试
```bash
# 快速 UserLogin 测试 (1-20 QPS)
npm run qps-test quick-login

# 快速 ShieldingKey 测试 (1-30 QPS)
npm run qps-test quick-shielding
```

## 📊 QPS 测试工作流程

### 1. **逐步加载阶段**
```
QPS 1  → 运行 30 秒 → 收集数据
QPS 6  → 运行 30 秒 → 收集数据  
QPS 11 → 运行 30 秒 → 收集数据
...
QPS 50 → 运行 30 秒 → 收集数据
```

### 2. **实时性能监控**
每个 QPS 级别实时显示：
- ✅ **请求成功率**: `45/50 requests (90% success)`
- ⏱️ **响应时间**: `avg 250ms, P95 400ms, P99 600ms`
- 🚨 **错误率**: `5% errors (timeout: 2, connection: 1)`
- 📊 **系统状态**: `✅ Good / ⚠️ Warning / ❌ Poor`

### 3. **自动瓶颈检测**
- **稳定性检查**: 错误率 < 5% 且平均响应时间 < 2s
- **瓶颈识别**: 性能显著下降的 QPS 点
- **安全阈值**: 自动计算生产环境推荐 QPS

## 📁 测试结果格式

### QPS 专用结果文件
```json
{
  "testConfig": {
    "testName": "ShieldingKey QPS Analysis",
    "targetUrl": "http://localhost:2100/",
    "startQPS": 1,
    "maxQPS": 50,
    "stepSize": 5,
    "stepDuration": 30,
    "testType": "omni_getShieldingKey"
  },
  "steps": [
    {
      "qps": 5,
      "stepStartTime": 1703123456789,
      "stepEndTime": 1703123486789,
      "stepDuration": 30,
      "totalRequests": 150,
      "successfulRequests": 147,
      "failedRequests": 3,
      "averageResponseTime": 245.6,
      "p95ResponseTime": 456.2,
      "p99ResponseTime": 612.8,
      "errorRate": 2.0,
      "detailedErrors": {
        "timeout": 2,
        "connection": 1
      },
      "systemBreakpoint": false
    }
  ],
  "maxSustainableQPS": 35,
  "breakpointQPS": 40,
  "summary": {
    "totalRequests": 2500,
    "totalDuration": 450,
    "overallSuccessRate": 96.8,
    "recommendations": [
      "系统最大可持续 QPS: 35",
      "系统在 40 QPS 开始出现性能下降",
      "建议生产环境控制在 28 QPS 以下以保证稳定性"
    ]
  }
}
```

### 标准压力测试格式
同时自动转换为标准的分布式测试结果格式，便于与现有分析工具兼容。

## 🔍 详细测试逻辑

### AddWallet 测试特殊处理
```typescript
// 预先认证获取 token
await prepareAuthenticatedTokens(5);

// 测试时使用预准备的 token
const authToken = getRandomAuthToken();
const result = await rpcClient.call('omni_addWallet', [], authToken);
```

### 混合测试权重分布
```typescript
// weighted_mixed 模式的真实业务场景分布
if (random < 0.4) {
  return performLogin();        // 40% - 用户登录 (最常见)
} else if (random < 0.7) {
  return performShieldingKey(); // 30% - 获取屏蔽密钥
} else if (random < 0.9) {
  return performGetNextIntentId(); // 20% - 获取意图 ID
} else {
  return performAddWallet();    // 10% - 添加钱包 (较少)
}
```

### 错误处理和分类
```typescript
// 详细的错误分类，便于问题定位
const errorCategories = {
  timeout: "请求超时",
  connection: "连接错误", 
  server_error: "服务器内部错误 (5xx)",
  auth_error: "认证失败",
  validation_error: "参数验证失败",
  other: "其他错误"
};
```

## 📈 实际应用场景

### 1. **API 性能基准测试**
```bash
# 为每个 API 建立性能基准
npm run qps-test api shielding-key    # 预期 60-80 QPS
npm run qps-test api add-wallet       # 预期 15-25 QPS  
npm run qps-test api mixed           # 预期 40-60 QPS
```

### 2. **系统容量规划**
```bash
# 全面的系统容量评估
npm run qps-example comprehensive
```

### 3. **性能回归检测**
```bash
# 每次发布前的性能回归测试
npm run qps-test run --max-qps 30 --step-duration 60
```

### 4. **生产环境监控阈值设定**
测试结果直接提供生产监控建议：
- 🎯 **安全 QPS**: 70% 最大可持续 QPS
- ⚠️ **警告阈值**: 85% 最大可持续 QPS  
- 🚨 **关键阈值**: 100% 最大可持续 QPS

## 🎯 测试结果解读

### 成功指标
- ✅ **Max Sustainable QPS > 目标值**: 系统性能满足需求
- ✅ **Error Rate < 5%**: 系统稳定性良好
- ✅ **P95 Response Time < 500ms**: 用户体验优秀

### 警告信号
- ⚠️ **Breakpoint QPS 接近 Max QPS**: 系统裕量不足
- ⚠️ **Error Rate 5-20%**: 需要关注稳定性
- ⚠️ **Response Time 逐步增长**: 可能存在性能瓶颈

### 需要优化
- 🚨 **Max Sustainable QPS < 需求**: 需要性能优化或扩容
- 🚨 **Error Rate > 20%**: 系统不稳定，需立即处理
- 🚨 **P99 Response Time > 2s**: 存在严重性能问题

## 💡 最佳实践

### 1. **测试环境准备**
- 确保 Omni Executor 服务稳定运行
- 预热系统，运行几分钟低负载测试
- 监控服务器资源使用情况

### 2. **测试参数选择**
- **起始 QPS**: 从 1-2 开始，确保基础功能正常
- **最大 QPS**: 根据预期容量设定，建议不超过预期的 2 倍
- **步长大小**: 10-20% 的增量比较合适
- **步长时间**: 30-60 秒，确保系统稳定

### 3. **结果分析**
- 关注错误率和响应时间的趋势变化
- 识别性能下降的拐点
- 结合系统资源监控分析瓶颈原因

## 🚨 注意事项

### 安全提醒
- ⚠️ **不要对生产环境进行高 QPS 测试**
- ⚠️ **监控目标服务器资源，避免压垮系统**
- ⚠️ **测试使用的 token 仅用于测试目的**

### 技术要求
- **Node.js**: >=20.0.0
- **内存**: 建议 4GB+ 用于高 QPS 测试
- **网络**: 稳定连接到目标系统
- **存储**: 约 10MB/1000 QPS*分钟的存储空间

通过这个完整的 QPS 逐步增加测试系统，你可以精确地了解 Omni Executor 各个接口的性能特征，为生产环境的容量规划和监控提供科学依据！🎯