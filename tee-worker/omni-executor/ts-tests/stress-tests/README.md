# Omni Executor Stress Tests

Professional stress testing suite for Omni Executor APIs with QPS scaling and performance analysis.

## 🚀 Quick Start

### 1. Install Dependencies
```bash
npm install
```

### 2. Configure Environment
```bash
# Option 1: Environment variable
export OMNI_RPC_URL="http://your-target-server:2100/"

# Option 2: Command line parameter
OMNI_RPC_URL="http://your-target-server:2100/" npm run stress

# Option 3: .env file
echo 'OMNI_RPC_URL="http://your-target-server:2100/"' > .env
```

### 3. Run Tests
```bash
# Run stress test with dashboard
npm run stress

# Simple API test
npx tsx src/simple-stress-test.ts
```

## 📊 Testing Model

### Test Strategy
- **Target APIs**: `omni_getNextIntentId`, `omni_getWeb3SignInMessage`
- **QPS Scaling**: Linear increase (1 → 3 → 5 → 7 → 9 → 11 → 13 → 15 → 18 → 21...)
- **Step Duration**: 30 seconds per QPS level
- **Safety Limit**: 70% of maximum sustainable QPS
- **Wallet Pool**: 500 pre-generated test wallets

### Performance Metrics
- **Throughput**: Requests per second (QPS)
- **Latency**: Response time distribution (P50, P95, P99)
- **Success Rate**: Percentage of successful requests
- **Error Analysis**: Error categorization and patterns

### Test Flow
1. **Connectivity Test**: Verify all API endpoints
2. **QPS Scaling**: Gradually increase load until performance degrades
3. **Performance Analysis**: Calculate sustainable QPS and recommendations
4. **Data Storage**: Save detailed results for dashboard analysis

### Dashboard
Access real-time monitoring at: `http://localhost:3001`

Features:
- Live QPS and latency charts
- Error rate monitoring
- Session comparison
- Export capabilities

## 🎯 Expected Results

```
QPS    1.0:   30/  30 requests,  0.0% errors,  245ms avg ✅ Excellent
QPS    3.0:   90/  90 requests,  0.0% errors,  251ms avg ✅ Excellent
QPS   15.0:  450/ 450 requests,  0.0% errors,  312ms avg ✅ Good
QPS   21.0:  630/ 630 requests,  8.9% errors,  356ms avg ❌ Poor

Max Sustainable QPS: 15
Recommended Production QPS (70%): 10
```

## 📁 Output Structure

```
stress-test-results/
├── stress_test_[timestamp]/
│   ├── session-result.json     # Complete test results
│   ├── requests/               # Individual request logs
│   └── analysis/               # Performance analysis
```