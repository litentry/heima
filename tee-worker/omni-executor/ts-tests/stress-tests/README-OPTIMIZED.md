# 🚀 Omni Executor 压力测试系统

专为 Heima Network Omni Executor 的 `OmniUserLogin` 接口设计的高级压力测试框架，支持分布式测试、实时监控和性能分析。

## ✨ 主要特性

### 🎯 专门针对 OmniUserLogin 接口优化
- **智能钱包池管理**: 预生成测试钱包，提升测试性能
- **完整登录流程**: 自动处理 Web3 消息签名和用户登录
- **真实场景模拟**: 基于 `jsonrpc_mock_test.test.ts` 的测试逻辑

### 🌐 分布式测试架构
- **跨机器协调**: 多台机器同时测试，模拟真实负载
- **结果自动聚合**: 收集各机器结果并生成统一报告
- **机器标识**: 每台机器生成独特标识，便于结果追踪

### 📊 实时监控 Dashboard
- **Web 界面**: 直观的实时性能监控
- **多维度图表**: 响应时间、吞吐量、错误率趋势
- **系统资源监控**: CPU、内存、网络使用情况

### 🔍 深度性能分析
- **承载能力计算**: 自动计算最大可持续 RPS
- **瓶颈识别**: 智能分析性能瓶颈
- **风险评估**: 生产环境风险等级评估
- **优化建议**: 针对性的性能优化建议

## 🚀 快速开始

### 安装依赖
```bash
cd tee-worker/omni-executor/ts-tests/stress-tests
pnpm install
```

### 基础测试
```bash
# 快速测试 (30秒，5并发)
npm run stress quick

# 标准测试 (60秒，10并发)
npm run stress run

# 高负载测试 (5分钟，50并发)
npm run stress run -d 300 -c 50 -r 30
```

### 自定义配置
```bash
# 指定目标 RPC 地址
npm run stress run -u http://production-server:2100/

# 自定义测试参数
npm run stress run \\
  --name "生产环境压测" \\
  --duration 180 \\
  --concurrency 25 \\
  --ramp-up 20 \\
  --url http://your-server:2100/
```

## 🌐 分布式测试

### 1. 部署到多台机器
```bash
# 机器 1
OMNI_RPC_URL=http://target:2100/ npm run stress run -c 20 -d 300

# 机器 2  
OMNI_RPC_URL=http://target:2100/ npm run stress run -c 30 -d 300

# 机器 3
OMNI_RPC_URL=http://target:2100/ npm run stress run -c 40 -d 300
```

### 2. 收集结果
```bash
# 将各机器的 stress-test-results/ 目录复制到一台机器上
mkdir combined-results
cp machine1/stress-test-results/* combined-results/
cp machine2/stress-test-results/* combined-results/
cp machine3/stress-test-results/* combined-results/

# 聚合分析
npm run stress aggregate -d combined-results
```

### 3. 性能分析
```bash
# 系统承载能力分析
npm run analyze capacity -d combined-results

# 生成综合报告
npm run analyze report -d combined-results -o performance-report.md
```

## 📊 实时监控

### 启动 Dashboard
```bash
npm run stress dashboard
```

访问 http://localhost:3000 查看实时性能指标：
- 🔥 当前 RPS 和响应时间
- 📈 实时趋势图表
- 🚨 错误率监控
- 💻 系统资源使用情况

## 🔍 性能分析报告

系统自动生成包含以下内容的详细报告：

### 📊 基础指标
- **总请求数**: 测试期间总请求量
- **成功率**: 请求成功百分比
- **平均响应时间**: 所有请求的平均响应时间
- **吞吐量**: 每秒处理的请求数 (RPS)

### 📈 响应时间分布
- **50th 百分位**: 中位数响应时间
- **90th 百分位**: 90% 请求的响应时间
- **95th 百分位**: 95% 请求的响应时间  
- **99th 百分位**: 99% 请求的响应时间

### 🎯 承载能力评估
- **最大可持续 RPS**: 生产环境推荐的最大 RPS
- **最大并发用户数**: 系统能稳定支持的并发数
- **错误阈值 RPS**: 开始出现错误的 RPS 水平
- **不同负载下的响应时间**: 50/100/200 RPS 下的预期响应时间

### 🚧 瓶颈识别
- **响应时间瓶颈**: 高延迟问题分析
- **吞吐量瓶颈**: 处理能力限制分析
- **错误率瓶颈**: 系统稳定性问题
- **系统资源瓶颈**: CPU/内存/网络限制

### ⚠️ 风险评估
- **低风险**: 系统表现良好，有充足裕量
- **中等风险**: 存在潜在问题，需要监控
- **高风险**: 存在严重问题，需要立即优化

### 💡 优化建议
基于测试结果自动生成的优化建议：
- 数据库查询优化
- 缓存策略建议
- 连接池配置优化
- 水平扩容建议
- 监控告警配置

## 🏗️ 项目架构

```
stress-tests/
├── src/
│   ├── core/
│   │   └── stress-engine.ts           # 核心压力测试引擎
│   ├── utils/
│   │   ├── web3-login-client.ts       # OmniUserLogin 专用客户端
│   │   └── json-rpc-client.ts         # JSON-RPC 基础客户端
│   ├── distributed/
│   │   └── result-collector.ts        # 分布式结果收集器
│   ├── dashboard/
│   │   ├── server.ts                  # Dashboard 服务器
│   │   └── public/
│   │       └── index.html             # Web 监控界面
│   ├── analysis/
│   │   └── performance-analyzer.ts    # 性能分析工具
│   ├── metrics/
│   │   ├── performance-collector.ts   # 性能指标收集
│   │   └── system-monitor.ts          # 系统资源监控
│   ├── storage/
│   │   └── data-storage.ts            # 数据存储管理
│   ├── types/
│   │   └── index.ts                   # TypeScript 类型定义
│   └── omni-stress-test.ts            # 优化的 CLI 入口
├── examples/
│   └── distributed-test-example.ts    # 分布式测试示例
└── package.json
```

## 🛠️ 高级功能

### 性能阈值配置
系统内置生产级性能阈值：
```typescript
const THRESHOLDS = {
  responseTime: {
    target: 300,      // 300ms 目标响应时间
    acceptable: 500,  // 500ms 可接受响应时间
    critical: 1000,   // 1s 关键响应时间
  },
  throughput: {
    target: 200,      // 200 RPS 目标吞吐量
    minimum: 50,      // 50 RPS 最低要求
  },
  errorRate: {
    warning: 0.5,     // 0.5% 警告错误率
    critical: 2,      // 2% 关键错误率
  }
};
```

### 自定义测试场景
```typescript
// 混合测试模式
const testConfig = {
  testName: '混合负载测试',
  targetUrl: 'http://localhost:2100/',
  duration: 300,
  concurrency: 50,
  testType: 'mixed', // 混合调用不同接口
};
```

## 📈 使用场景

### 1. 开发环境测试
```bash
# 快速验证功能正确性
npm run stress quick -u http://localhost:2100/
```

### 2. 预发布环境验证
```bash
# 中等负载测试，验证基本性能
npm run stress run -d 120 -c 20 -u http://staging:2100/
```

### 3. 生产环境评估
```bash
# 高负载测试，评估系统极限
npm run stress run -d 600 -c 100 -u http://production:2100/
```

### 4. 性能回归测试
```bash
# 每日自动化性能基准测试
npm run stress run -d 180 -c 30 --name "每日基准测试"
```

## 🔧 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `OMNI_RPC_URL` | 目标 RPC 地址 | `http://localhost:2100/` |
| `STRESS_TEST_OUTPUT_DIR` | 结果输出目录 | `./stress-test-results` |
| `DASHBOARD_PORT` | Dashboard 端口 | `3000` |

## 🚨 注意事项

### 性能测试最佳实践
1. **逐步增加负载**: 从低并发开始，逐步提升到目标负载
2. **监控系统资源**: 测试过程中监控 CPU、内存、网络使用情况
3. **多轮测试验证**: 进行多轮测试确保结果的一致性和可靠性
4. **考虑真实场景**: 测试环境应尽可能接近生产环境配置

### 安全注意事项
- ⚠️ 不要对生产环境进行高负载测试
- ⚠️ 测试使用的钱包为随机生成，仅用于测试目的
- ⚠️ 分布式测试时确保网络连接稳定
- ⚠️ 监控目标系统资源，避免造成服务不可用

### 系统要求
- **Node.js**: >=20.0.0
- **内存**: 建议 4GB+ 用于高并发测试
- **网络**: 稳定的网络连接到目标系统
- **存储**: 足够空间存储测试结果（约 1MB/1000 请求）

## 📞 支持与反馈

如遇到问题或有改进建议，请：
1. 检查测试目标系统是否正常运行
2. 验证网络连接和防火墙设置
3. 查看详细的错误日志和系统监控
4. 参考项目文档和示例代码

---

**Happy Testing!** 🎯 通过科学的压力测试，确保 Omni Executor 在各种负载条件下都能稳定高效地运行。