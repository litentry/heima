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

## 📊 Dashboard

### 🎯 Prerequisites
**You must run stress tests first to generate data before the dashboard will display content**

```bash
# 1. Run stress tests to generate test data
pnpm run stress

# 2. Install dashboard dependencies (first time only)
pnpm run dashboard:install

# 3. Start dashboard
pnpm run dashboard
```

### 📱 Access
Open browser and visit: `http://localhost:3000`

### ⚙️ Tech Stack
- **Framework**: Next.js 14 + TypeScript
- **Package Manager**: pnpm (recommended)
- **Charts**: Chart.js + react-chartjs-2
- **Styling**: CSS-in-JS

### 📋 Features
- **Real-time Data Visualization**: QPS performance charts, latency analysis
- **Success Rate Monitoring**: Error rate statistics and trend analysis
- **Session Comparison**: Historical test session comparison
- **Data Analysis**: Detailed performance metrics and recommendations

### 🚨 Important Notes
1. **Data Dependency**: Dashboard reads data from `stress-test-results/` directory
2. **Port Availability**: Ensure port 3000 is not occupied
3. **Package Manager**: Use pnpm instead of npm to avoid version conflicts
4. **Data Refresh**: Page automatically refreshes data every 30 seconds, or manually refresh browser

### 🔧 Development Mode
```bash
cd src/dashboard
pnpm install
pnpm run dev
```

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