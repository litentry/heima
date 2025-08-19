# 🚀 Omni UserLogin 压力测试 - 快速开始

## 📋 测试服务概述

本系统专门针对 **Omni Executor** 的 `OmniUserLogin` 接口进行压力测试：

- **目标服务**: Omni Executor RPC 服务 
- **默认端口**: `2100`
- **主要接口**: `omni_userLogin`, `omni_getWeb3SignInMessage`
- **测试类型**: Web3 用户登录完整流程压力测试

## ⚡ 5分钟快速上手

### 1. 安装依赖
```bash
cd tee-worker/omni-executor/ts-tests/stress-tests
pnpm install
```

### 2. 启动你的 Omni Executor 服务
确保 Omni Executor 服务运行在 `http://localhost:2100/`

### 3. 运行快速测试
```bash
# 30秒轻量测试 (5并发)
npm run stress quick

# 标准测试 (60秒，10并发)  
npm run stress run

# 自定义测试
npm run stress run -d 120 -c 20 -u http://your-server:2100/
```

## 🔍 测试逻辑说明

### 核心测试流程
每个虚拟用户执行以下完整的 Web3 登录流程：

1. **生成测试钱包** → 2. **获取签名消息** → 3. **签名消息** → 4. **执行登录** → 5. **验证响应**

```
[钱包池] → [omni_getWeb3SignInMessage] → [签名] → [omni_userLogin] → [验证结果]
   ↓              ↓                      ↓          ↓            ↓
预生成100个      获取challenge          本地签名    提交登录      检查code=10000
测试钱包         消息和参数             消息        请求         业务成功标志
```

### 🛡️ 容错设计

**关键特性：测试过程绝不中断**

- ✅ **分步错误捕获**: 每个步骤独立捕获错误
- ✅ **永不抛出异常**: 所有错误都转换为测试结果记录
- ✅ **详细错误分类**: 网络错误、业务错误、系统错误分别统计
- ✅ **持续测试**: 单个请求失败不影响其他并发请求

```typescript
// 示例错误处理
try {
  result = await performLogin();  // 永不抛出异常
} catch (error) {
  // 这个 catch 永远不会执行，因为 performLogin 内部已全面捕获
}

// 即使所有请求都失败，测试也会完整运行并生成报告
```

## 📊 实时监控

### 启动 Dashboard
```bash
npm run stress dashboard
# 访问 http://localhost:3000
```

实时查看：
- 📈 **当前 RPS**: 每秒请求数
- ⏱️ **响应时间**: 平均/P95/P99 响应时间  
- 🚨 **错误率**: 实时错误百分比
- 💻 **系统资源**: CPU/内存使用情况

## 🌐 分布式测试

### 多机器协同测试
```bash
# 机器 A (轻负载)
npm run stress run -c 15 -d 300 -u http://target:2100/

# 机器 B (中负载)  
npm run stress run -c 25 -d 300 -u http://target:2100/

# 机器 C (重负载)
npm run stress run -c 40 -d 300 -u http://target:2100/
```

### 结果聚合分析
```bash
# 收集各机器的结果文件到一个目录
mkdir combined-results
cp machineA/stress-test-results/* combined-results/
cp machineB/stress-test-results/* combined-results/  
cp machineC/stress-test-results/* combined-results/

# 聚合分析
npm run stress aggregate -d combined-results

# 性能深度分析
npm run analyze capacity -d combined-results
```

## 🎯 常用测试场景

### 开发环境验证
```bash
# 功能验证 (低负载)
npm run stress run -d 60 -c 5 -u http://localhost:2100/
```

### 预发布环境测试
```bash  
# 中等负载验证
npm run stress run -d 180 -c 20 -u http://staging:2100/
```

### 生产容量评估  
```bash
# 高负载压测 (谨慎使用)
npm run stress run -d 300 -c 50 -u http://production:2100/
```

### 性能回归测试
```bash
# 每日基准测试
npm run stress run -d 120 -c 15 --name "每日性能基准"
```

## 📊 结果解读

### 成功指标
- ✅ **成功率 > 99%**: 系统稳定性良好
- ✅ **P95 响应时间 < 500ms**: 用户体验优秀  
- ✅ **平均 RPS > 50**: 吞吐量满足需求

### 警告信号
- ⚠️ **成功率 95-99%**: 存在潜在问题
- ⚠️ **P95 响应时间 500-1000ms**: 性能有待优化
- ⚠️ **错误率 > 1%**: 需要关注稳定性

### 危险信号  
- 🚨 **成功率 < 95%**: 系统不稳定，需要立即优化
- 🚨 **P95 响应时间 > 1000ms**: 严重性能问题
- 🚨 **错误率 > 5%**: 系统接近不可用状态

## 🔧 配置参数

### 基础参数
| 参数 | 说明 | 默认值 | 推荐范围 |
|------|------|--------|----------|
| `-d, --duration` | 测试时长(秒) | 60 | 30-600 |
| `-c, --concurrency` | 并发用户数 | 10 | 1-100 |
| `-r, --ramp-up` | 启动时间(秒) | 10 | 5-60 |
| `-u, --url` | 目标地址 | localhost:2100/ | - |

### 高级参数
```bash
npm run stress run \
  --name "生产压测" \
  --duration 300 \
  --concurrency 50 \
  --ramp-up 30 \
  --url http://production:2100/
```

## 🚨 注意事项

### 安全须知
- ⚠️ **不要对生产环境进行高并发测试** 
- ⚠️ **测试钱包仅用于测试，无真实价值**
- ⚠️ **确保有足够的系统资源运行测试**
- ⚠️ **监控目标服务器的资源使用**

### 最佳实践
1. **渐进式测试**: 从低并发开始逐步增加
2. **多轮验证**: 运行多次测试确保结果一致性  
3. **资源监控**: 同时监控客户端和服务端资源
4. **错误分析**: 详细分析每种错误的根本原因

## 📞 问题排查

### 常见问题
```bash
# 连接失败
Error: fetch failed
→ 检查 Omni Executor 服务是否运行在指定端口

# 认证失败  
Login validation failed: Invalid credentials
→ 检查 omni_userLogin 接口的认证逻辑

# 签名失败
Step 2 - Sign message failed: Invalid private key  
→ 检查钱包生成逻辑和签名算法
```

### 调试模式
```bash
# 启用详细日志
DEBUG=* npm run stress run -d 30 -c 1

# 单用户调试
npm run stress run -d 10 -c 1 -u http://localhost:2100/
```

现在你就可以开始对 Omni Executor 的 UserLogin 接口进行专业的压力测试了！🎯