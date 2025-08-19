# Omni Executor Stress Test Framework

A professional stress testing framework for the Omni Executor, featuring comprehensive performance monitoring, real-time dashboards, and detailed reporting.

## Features

### 🚀 Core Capabilities
- **High-performance stress testing** with configurable concurrency and duration
- **Web3 login integration** based on existing test patterns
- **Real-time monitoring** with live dashboard
- **Comprehensive metrics collection** (response time, throughput, error rates)
- **System resource monitoring** (CPU, memory, disk, network)

### 📊 Performance Metrics

#### Response Time Metrics
- Average, minimum, maximum response times
- 50th, 90th, 95th, and 99th percentile response times

#### Throughput Metrics
- Requests per second (RPS)
- Transactions per second (TPS)
- Data transfer rates (KB/s, MB/s)
- Peak concurrency measurements

#### Error Rate Metrics
- HTTP error rates (4xx, 5xx)
- Timeout and connection error rates
- Error categorization and distribution

#### System Resource Metrics
- **CPU**: Usage percentage, core-level usage, load average, context switches
- **Memory**: Usage percentage, available memory, cache utilization
- **Disk I/O**: Read/write rates, I/O wait times, queue lengths
- **Network**: Bandwidth usage, latency, packet loss, connection counts

### 📈 Data Export & Visualization
- **JSON exports** for programmatic analysis
- **Excel reports** with charts and detailed breakdowns
- **Real-time web dashboard** with live charts
- **Console output** with formatted tables

## Installation

```bash
# Install dependencies
pnpm install

# Build the project
pnpm build
```

## Usage

### Command Line Interface

#### Basic Stress Test
```bash
# Run a basic 60-second test with 10 concurrent users
pnpm stress -- run -n "Basic Load Test" -d 60 -c 10

# Run with custom parameters
pnpm stress -- run \
  --name "High Load Test" \
  --duration 300 \
  --concurrency 100 \
  --ramp-up 30 \
  --url "http://localhost:2100/" \
  --timeout 30000
```

#### Dashboard Mode
```bash
# Run test with real-time dashboard
pnpm stress -- run --dashboard -d 300 -c 50

# Start dashboard server only (for external test runner)
pnpm dashboard
# Then visit http://localhost:3000
```

#### Session Management
```bash
# List previous test sessions
pnpm stress -- list

# Export sessions to JSON
pnpm stress -- export -s session-id-1 session-id-2

# Show example usage
pnpm stress -- example
```

### Programmatic Usage

```typescript
import { StressTestEngine, StressTestConfig } from '@heima-network/stress-tests';

const config: StressTestConfig = {
  testName: 'API Load Test',
  duration: 120,
  concurrency: 50,
  rampUpTime: 20,
  rampDownTime: 10,
  targetUrl: 'http://localhost:2100/',
  requestTimeout: 30000,
  retries: 3,
  outputFormat: ['json', 'excel'],
  outputPath: './results',
  enableDashboard: true,
  dashboardPort: 3000
};

const engine = new StressTestEngine();

// Setup event listeners
engine.on('testCompleted', ({ summary }) => {
  console.log('Test completed:', summary);
});

engine.on('requestCompleted', (result) => {
  console.log('Request completed:', result.responseTime);
});

// Start test
const sessionId = await engine.startTest(config);
```

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `testName` | string | 'Default Test' | Name for the test session |
| `duration` | number | 60 | Test duration in seconds |
| `concurrency` | number | 10 | Number of concurrent users |
| `rampUpTime` | number | 10 | Time to ramp up to full concurrency |
| `rampDownTime` | number | 10 | Time to ramp down from full concurrency |
| `targetUrl` | string | 'http://localhost:2100/' | Target server URL |
| `requestTimeout` | number | 30000 | Request timeout in milliseconds |
| `retries` | number | 3 | Number of retries for failed requests |
| `outputFormat` | array | ['json', 'console'] | Output formats |
| `outputPath` | string | './results' | Directory for result files |
| `enableDashboard` | boolean | false | Enable real-time dashboard |
| `dashboardPort` | number | 3000 | Port for dashboard server |

## Dashboard Features

The web dashboard provides real-time monitoring with:

- **Live metrics display**: Current RPS, error rates, response times
- **Interactive charts**: Response time trends, throughput graphs, system resources
- **System monitoring**: CPU, memory, disk, and network usage
- **Error analysis**: Error distribution and categorization
- **Session management**: Start, stop, and monitor tests
- **Historical data**: View past test results

## Test Request Types

The framework includes several request types based on the existing Omni Executor test patterns:

1. **Web3 Login Flow**
   - Get sign-in message
   - Sign message with wallet
   - Perform authentication

2. **Shielding Key Requests**
   - Retrieve encryption keys

3. **Intent ID Requests**
   - Get next available intent ID

4. **Mixed Request Patterns**
   - Random selection of request types for realistic load

## Output Files

Results are automatically saved to the specified output directory:

```
./stress-test-results/
├── stress-test-{timestamp}-{id}/
│   ├── session.json          # Complete session data
│   ├── summary.json          # Performance summary
│   └── report.xlsx          # Detailed Excel report
└── export-{timestamp}.json  # Exported session data
```

## Performance Tuning

### For High Load Testing

```bash
# Increase system limits
ulimit -n 65536

# Run with high concurrency
pnpm stress -- run -c 1000 -d 600 --timeout 10000
```

### Resource Monitoring

The framework automatically monitors:
- Process-level CPU and memory usage
- System-wide resource utilization
- Network I/O and disk operations
- Real-time performance metrics

## Examples

### Quick Load Test
```bash
pnpm stress -- run -d 30 -c 5
```

### Production Simulation
```bash
pnpm stress -- run \
  --name "Production Load Simulation" \
  --duration 1800 \
  --concurrency 200 \
  --ramp-up 120 \
  --dashboard \
  --output ./prod-test-results
```

### Continuous Monitoring
```bash
# Terminal 1: Start dashboard
pnpm dashboard

# Terminal 2: Run multiple tests
pnpm stress -- run -c 50 -d 300
pnpm stress -- run -c 100 -d 300
pnpm stress -- run -c 200 -d 300
```

## Development

### Project Structure

```
src/
├── core/                 # Core testing engine
├── metrics/             # Performance and system monitoring
├── storage/             # Data persistence and export
├── dashboard/           # Web dashboard and real-time UI
├── utils/              # Utilities and client code
└── types/              # TypeScript type definitions
```

### Building and Testing

```bash
# Development mode with hot reload
pnpm dev

# Build for production
pnpm build

# Run tests
pnpm test

# Code formatting and linting
pnpm format
pnpm lint
```

## Architecture

The framework follows a modular architecture:

- **StressTestEngine**: Orchestrates test execution
- **PerformanceCollector**: Collects and analyzes performance metrics
- **SystemMonitor**: Monitors system resources
- **DataStorage**: Handles data persistence and export
- **DashboardServer**: Provides real-time web interface
- **Web3LoginClient**: Integrates with Omni Executor APIs

## Contributing

1. Follow the existing code patterns
2. Add comprehensive tests for new features
3. Update documentation for API changes
4. Use TypeScript strict mode
5. Follow the project's ESLint configuration

## License

MIT License - see LICENSE file for details.