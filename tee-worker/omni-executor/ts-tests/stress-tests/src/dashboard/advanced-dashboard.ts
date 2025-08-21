import express from 'express';
import path from 'path';
import fs from 'fs';
import { WebSocketServer } from 'ws';
import http from 'http';
import { fileURLToPath } from 'url';
import { exec } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

interface DashboardConfig {
  port: number;
  host: string;
  outputDir: string;
  enableRealtime: boolean;
  enableExports: boolean;
}

interface SessionSummary {
  sessionId: string;
  startTime: number;
  endTime: number;
  duration: number;
  totalRequests: number;
  successRate: number;
  maxQPS: number;
  avgResponseTime?: number;
  status: 'completed' | 'running' | 'failed';
}

export class Dashboard {
  private app: express.Application;
  private server: http.Server;
  private wss: WebSocketServer | null = null;
  private config: DashboardConfig;
  private activeSessions: Map<string, any> = new Map();

  constructor(outputDir: string, config: Partial<DashboardConfig> = {}) {
    this.config = {
      port: 3000,
      host: '0.0.0.0',
      outputDir,
      enableRealtime: true,
      enableExports: true,
      ...config
    };

    this.app = express();
    this.server = http.createServer(this.app);
    
    if (this.config.enableRealtime) {
      this.wss = new WebSocketServer({ server: this.server });
      this.setupWebSocket();
    }

    this.setupRoutes();
    this.setupStaticFiles();
  }

  private setupWebSocket(): void {
    if (!this.wss) return;

    this.wss.on('connection', (ws) => {
      console.log('Dashboard client connected');
      
      // Send initial data
      ws.send(JSON.stringify({
        type: 'init',
        data: {
          sessions: this.getSessionList(),
          activeSessions: Array.from(this.activeSessions.keys())
        }
      }));

      ws.on('message', (message) => {
        try {
          const data = JSON.parse(message.toString());
          this.handleWebSocketMessage(ws, data);
        } catch (error) {
          console.error('Invalid WebSocket message:', error);
        }
      });

      ws.on('close', () => {
        console.log('Dashboard client disconnected');
      });
    });
  }

  private handleWebSocketMessage(ws: any, data: any): void {
    switch (data.type) {
      case 'subscribe':
        // Subscribe to session updates
        if (data.sessionId) {
          this.subscribeToSession(ws, data.sessionId);
        }
        break;
      case 'unsubscribe':
        // Unsubscribe from session updates
        if (data.sessionId) {
          this.unsubscribeFromSession(ws, data.sessionId);
        }
        break;
      default:
        console.warn('Unknown WebSocket message type:', data.type);
    }
  }

  private subscribeToSession(ws: any, sessionId: string): void {
    // Store subscription (in a real implementation, you'd manage this properly)
    ws.sessionSubscriptions = ws.sessionSubscriptions || new Set();
    ws.sessionSubscriptions.add(sessionId);
  }

  private unsubscribeFromSession(ws: any, sessionId: string): void {
    if (ws.sessionSubscriptions) {
      ws.sessionSubscriptions.delete(sessionId);
    }
  }

  private broadcastToSubscribers(sessionId: string, data: any): void {
    if (!this.wss) return;

    this.wss.clients.forEach((client) => {
      if (client.readyState === 1 && // WebSocket.OPEN
          (client as any).sessionSubscriptions?.has(sessionId)) {
        client.send(JSON.stringify({
          type: 'sessionUpdate',
          sessionId,
          data
        }));
      }
    });
  }

  private setupRoutes(): void {
    // Enable JSON parsing
    this.app.use(express.json());
    
    // Enable CORS for development
    this.app.use((req, res, next) => {
      res.header('Access-Control-Allow-Origin', '*');
      res.header('Access-Control-Allow-Headers', 'Content-Type');
      res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE');
      next();
    });

    // API Routes
    this.app.get('/api/sessions', this.handleGetSessions.bind(this));
    this.app.get('/api/sessions/:sessionId', this.handleGetSession.bind(this));
    this.app.get('/api/sessions/:sessionId/requests', this.handleGetRequests.bind(this));
    this.app.get('/api/sessions/:sessionId/steps', this.handleGetSteps.bind(this));
    this.app.get('/api/sessions/:sessionId/export/:format', this.handleExport.bind(this));
    this.app.delete('/api/sessions/:sessionId', this.handleDeleteSession.bind(this));
    
    // Statistics
    this.app.get('/api/stats/overview', this.handleGetOverviewStats.bind(this));
    
    // Real-time updates
    this.app.post('/api/sessions/:sessionId/update', this.handleSessionUpdate.bind(this));
  }

  private setupStaticFiles(): void {
    // Create dashboard HTML if it doesn't exist
    this.createDashboardHTML();
    
    // Serve static files
    const publicDir = path.join(__dirname, 'public');
    if (!fs.existsSync(publicDir)) {
      fs.mkdirSync(publicDir, { recursive: true });
    }
    
    this.app.use(express.static(publicDir));
    
    // Serve dashboard at root
    this.app.get('/', (req, res) => {
      res.sendFile(path.join(publicDir, 'index.html'));
    });
  }

  private createDashboardHTML(): void {
    const publicDir = path.join(__dirname, 'public');
    const htmlFile = path.join(publicDir, 'index.html');
    
    if (!fs.existsSync(publicDir)) {
      fs.mkdirSync(publicDir, { recursive: true });
    }

    const html = this.generateDashboardHTML();
    fs.writeFileSync(htmlFile, html);

    // Create dashboard JavaScript
    const jsFile = path.join(publicDir, 'dashboard.js');
    const js = this.generateDashboardJS();
    fs.writeFileSync(jsFile, js);

    // Create dashboard CSS
    const cssFile = path.join(publicDir, 'styles.css');
    const css = this.generateDashboardCSS();
    fs.writeFileSync(cssFile, css);
  }

  private generateDashboardHTML(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Omni Stress Test Dashboard</title>
    <link rel="stylesheet" href="styles.css">
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns/dist/chartjs-adapter-date-fns.bundle.min.js"></script>
</head>
<body>
    <div class="dashboard">
        <!-- Header -->
        <header class="header">
            <h1>🚀 Omni Stress Test Dashboard</h1>
            <div class="header-stats">
                <div class="stat-card">
                    <span class="stat-label">Total Sessions</span>
                    <span class="stat-value" id="totalSessions">0</span>
                </div>
                <div class="stat-card">
                    <span class="stat-label">Active Tests</span>
                    <span class="stat-value" id="activeTests">0</span>
                </div>
                <div class="stat-card">
                    <span class="stat-label">Total Requests</span>
                    <span class="stat-value" id="totalRequests">0</span>
                </div>
            </div>
        </header>

        <!-- Navigation -->
        <nav class="nav-tabs">
            <button class="tab-button active" onclick="showTab('overview')">Overview</button>
            <button class="tab-button" onclick="showTab('sessions')">Sessions</button>
        </nav>

        <!-- Main Content -->
        <main class="content">
            <!-- Overview Tab -->
            <div id="overview" class="tab-content active">
                <!-- Key Metrics Cards -->
                <div class="metrics-grid">
                    <div class="metric-card">
                        <div class="metric-value" id="currentQPS">0</div>
                        <div class="metric-label">Current QPS</div>
                        <div class="metric-trend" id="qpsTrend">+0%</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="avgLatency">0ms</div>
                        <div class="metric-label">Avg Response Time</div>
                        <div class="metric-trend" id="latencyTrend">+0%</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="errorRate">0%</div>
                        <div class="metric-label">Error Rate</div>
                        <div class="metric-trend" id="errorTrend">+0%</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value" id="throughput">0</div>
                        <div class="metric-label">Req/Min</div>
                        <div class="metric-trend" id="throughputTrend">+0%</div>
                    </div>
                </div>
                
                <!-- Main Charts -->
                <div class="grid-2">
                    <div class="card">
                        <h3>Response Time Latency</h3>
                        <canvas id="latencyChart"></canvas>
                    </div>
                    <div class="card">
                        <h3>QPS & Error Rate</h3>
                        <canvas id="qpsErrorChart"></canvas>
                    </div>
                </div>
                
                <!-- Detailed Charts -->
                <div class="grid-3">
                    <div class="card">
                        <h3>QPS Progression</h3>
                        <canvas id="qpsProgressionChart"></canvas>
                    </div>
                    <div class="card">
                        <h3>Error Distribution</h3>
                        <canvas id="errorDistributionChart"></canvas>
                    </div>
                    <div class="card">
                        <h3>Endpoint Comparison</h3>
                        <canvas id="endpointComparisonChart"></canvas>
                    </div>
                </div>
                
                <!-- Recent Sessions -->
                <div class="card">
                    <h3>Recent Test Sessions</h3>
                    <div id="recentSessions" class="session-list"></div>
                </div>
            </div>

            <!-- Sessions Tab -->
            <div id="sessions" class="tab-content">
                <div class="card">
                    <div class="card-header">
                        <h3>Test Sessions</h3>
                        <div class="controls">
                            <input type="text" id="sessionFilter" placeholder="Filter sessions..." />
                            <button onclick="refreshSessions()">Refresh</button>
                            <button onclick="exportAllData()">Export All</button>
                        </div>
                    </div>
                    <div class="table-container">
                        <table id="sessionsTable" class="data-table">
                            <thead>
                                <tr>
                                    <th>Session ID</th>
                                    <th>Start Time</th>
                                    <th>Duration</th>
                                    <th>Requests</th>
                                    <th>Max QPS</th>
                                    <th>Success Rate</th>
                                    <th>Status</th>
                                    <th>Actions</th>
                                </tr>
                            </thead>
                            <tbody></tbody>
                        </table>
                    </div>
                </div>
            </div>
        </main>
    </div>

    <!-- Modal for detailed views -->
    <div id="modal" class="modal">
        <div class="modal-content">
            <span class="close" onclick="closeModal()">&times;</span>
            <div id="modalContent"></div>
        </div>
    </div>

    <script src="dashboard.js"></script>
</body>
</html>`;
  }

  private generateDashboardJS(): string {
    return `// Dashboard JavaScript
let currentSession = null;
let charts = {};
let wsConnection = null;

// Initialize dashboard
document.addEventListener('DOMContentLoaded', function() {
    initializeWebSocket();
    loadInitialData();
    setupEventListeners();
});

function initializeWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = protocol + '//' + window.location.host;
    
    wsConnection = new WebSocket(wsUrl);
    
    wsConnection.onopen = function() {
        console.log('WebSocket connected');
    };
    
    wsConnection.onmessage = function(event) {
        const data = JSON.parse(event.data);
        handleWebSocketMessage(data);
    };
    
    wsConnection.onclose = function() {
        console.log('WebSocket disconnected, attempting to reconnect...');
        setTimeout(initializeWebSocket, 5000);
    };
}

function handleWebSocketMessage(data) {
    switch(data.type) {
        case 'init':
            updateOverviewStats(data.data);
            break;
        case 'sessionUpdate':
            updateRealtimeData(data.sessionId, data.data);
            break;
    }
}

async function loadInitialData() {
    try {
        const [sessions, stats] = await Promise.all([
            fetch('/api/sessions').then(r => r.json()),
            fetch('/api/stats/overview').then(r => r.json())
        ]);
        
        updateSessionsTable(sessions);
        updateOverviewStats(stats);
        updateSessionSelects(sessions);
        createOverviewCharts(sessions);
    } catch (error) {
        console.error('Failed to load initial data:', error);
    }
}

function setupEventListeners() {
    document.getElementById('sessionFilter').addEventListener('input', filterSessions);
}

function showTab(tabName) {
    // Hide all tabs
    document.querySelectorAll('.tab-content').forEach(tab => {
        tab.classList.remove('active');
    });
    
    // Show selected tab
    document.getElementById(tabName).classList.add('active');
    
    // Update tab buttons
    document.querySelectorAll('.tab-button').forEach(btn => {
        btn.classList.remove('active');
    });
    event.target.classList.add('active');
    
    // Load tab-specific data
    switch(tabName) {
        case 'sessions':
            refreshSessions();
            break;
    }
}

function updateOverviewStats(stats) {
    document.getElementById('totalSessions').textContent = stats.totalSessions || 0;
    document.getElementById('activeTests').textContent = stats.activeTests || 0;
    document.getElementById('totalRequests').textContent = (stats.totalRequests || 0).toLocaleString();
}

function updateSessionsTable(sessions) {
    const tbody = document.querySelector('#sessionsTable tbody');
    tbody.innerHTML = '';
    
    sessions.forEach(session => {
        const row = document.createElement('tr');
        row.innerHTML = \`
            <td><a href="#" onclick="viewSession('\${session.sessionId}')">\${session.sessionId}</a></td>
            <td>\${new Date(session.startTime).toLocaleString()}</td>
            <td>\${formatDuration(session.duration)}</td>
            <td>\${session.totalRequests.toLocaleString()}</td>
            <td>\${session.maxQPS}</td>
            <td>\${session.successRate.toFixed(1)}%</td>
            <td><span class="status \${session.status}">\${session.status}</span></td>
            <td>
                <button onclick="viewSession('\${session.sessionId}')">View</button>
                <button onclick="exportSession('\${session.sessionId}')">Export</button>
                <button onclick="deleteSession('\${session.sessionId}')" class="danger">Delete</button>
            </td>
        \`;
        tbody.appendChild(row);
    });
}

async function createOverviewCharts(sessions) {
    // Get the latest session for detailed charts
    const latestSession = sessions[0];
    if (!latestSession) return;
    
    // Fetch detailed session data
    const [sessionData, stepsData] = await Promise.all([
        fetch('/api/sessions/' + latestSession.sessionId).then(r => r.json()),
        fetch('/api/sessions/' + latestSession.sessionId + '/steps').then(r => r.json())
    ]).catch(e => [null, []]);
    
    // Update metrics cards
    updateMetricsCards(sessionData, stepsData);
    
    // 1. Response Time Latency Chart
    createLatencyChart(stepsData);
    
    // 2. QPS & Error Rate Combined Chart
    createQPSErrorChart(stepsData);
    
    // 3. QPS Progression Chart
    createQPSProgressionChart(stepsData);
    
    // 4. Error Distribution Chart
    createErrorDistributionChart(stepsData);
    
    // 5. Endpoint Comparison Chart
    createEndpointComparisonChart(stepsData);
}

function updateMetricsCards(sessionData, stepsData) {
    if (!sessionData || !stepsData.length) return;
    
    const latestStep = stepsData[stepsData.length - 1];
    
    // Use responseMetrics if available, otherwise fallback to old calculation
    const avgLatency = latestStep.responseMetrics ? 
        latestStep.responseMetrics.avgLatency : 
        calculateAverageLatency(stepsData);
    
    const p95Latency = latestStep.responseMetrics ? 
        latestStep.responseMetrics.p95Latency : 
        avgLatency * 1.5;
    
    const p99Latency = latestStep.responseMetrics ? 
        latestStep.responseMetrics.p99Latency : 
        avgLatency * 2;
    
    const errorRate = latestStep.failedRequests / latestStep.totalRequests * 100;
    const throughput = latestStep.actualQPS * 60;
    
    document.getElementById('currentQPS').textContent = latestStep.actualQPS.toFixed(1);
    document.getElementById('avgLatency').textContent = Math.round(avgLatency) + 'ms';
    document.getElementById('errorRate').textContent = errorRate.toFixed(2) + '%';
    document.getElementById('throughput').textContent = Math.round(throughput);
    
    // Update latency percentiles if elements exist
    const p95Element = document.getElementById('p95Latency');
    const p99Element = document.getElementById('p99Latency');
    if (p95Element) p95Element.textContent = Math.round(p95Latency) + 'ms';
    if (p99Element) p99Element.textContent = Math.round(p99Latency) + 'ms';
    
    // Calculate trends (simplified)
    const prevStep = stepsData.length > 1 ? stepsData[stepsData.length - 2] : latestStep;
    const qpsTrend = ((latestStep.actualQPS - prevStep.actualQPS) / prevStep.actualQPS * 100);
    document.getElementById('qpsTrend').textContent = (qpsTrend >= 0 ? '+' : '') + qpsTrend.toFixed(1) + '%';
    document.getElementById('qpsTrend').className = 'metric-trend ' + (qpsTrend >= 0 ? 'positive' : 'negative');
    
    // Add latency trend calculation
    const prevAvgLatency = prevStep.responseMetrics ? 
        prevStep.responseMetrics.avgLatency : 
        calculateAverageLatency([prevStep]);
    const latencyTrend = ((avgLatency - prevAvgLatency) / prevAvgLatency * 100);
    const latencyTrendElement = document.getElementById('latencyTrend');
    if (latencyTrendElement) {
        latencyTrendElement.textContent = (latencyTrend >= 0 ? '+' : '') + latencyTrend.toFixed(1) + '%';
        latencyTrendElement.className = 'metric-trend ' + (latencyTrend <= 0 ? 'positive' : 'negative'); // Lower latency is better
    }
}

function createLatencyChart(stepsData) {
    const ctx = document.getElementById('latencyChart').getContext('2d');
    
    const labels = stepsData.map(step => step.qps + ' QPS');
    const avgLatencies = stepsData.map(step => step.responseMetrics ? step.responseMetrics.avgLatency : calculateStepAverageLatency(step));
    const p95Latencies = stepsData.map(step => step.responseMetrics ? step.responseMetrics.p95Latency : calculateStepP95Latency(step));
    const p99Latencies = stepsData.map(step => step.responseMetrics ? step.responseMetrics.p99Latency : calculateStepMaxLatency(step));
    
    charts.latency = new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                {
                    label: 'Average Latency',
                    data: avgLatencies,
                    borderColor: 'rgb(75, 192, 192)',
                    backgroundColor: 'rgba(75, 192, 192, 0.1)',
                    tension: 0.1,
                    fill: false
                },
                {
                    label: 'P95 Latency',
                    data: p95Latencies,
                    borderColor: 'rgb(255, 159, 64)',
                    backgroundColor: 'rgba(255, 159, 64, 0.1)',
                    tension: 0.1,
                    fill: false
                },
                {
                    label: 'P99 Latency',
                    data: p99Latencies,
                    borderColor: 'rgb(255, 99, 132)',
                    backgroundColor: 'rgba(255, 99, 132, 0.1)',
                    tension: 0.1,
                    fill: false
                }
            ]
        },
        options: {
            responsive: true,
            interaction: {
                intersect: false,
                mode: 'index'
            },
            scales: {
                y: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: 'Response Time (ms)'
                    }
                },
                x: {
                    title: {
                        display: true,
                        text: 'QPS Level'
                    }
                }
            },
            plugins: {
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            return context.dataset.label + ': ' + context.parsed.y.toFixed(2) + 'ms';
                        }
                    }
                }
            }
        }
    });
}

function createQPSErrorChart(stepsData) {
    const ctx = document.getElementById('qpsErrorChart').getContext('2d');
    
    const labels = stepsData.map(step => 'Step ' + step.qps);
    const qpsData = stepsData.map(step => step.actualQPS);
    const errorRates = stepsData.map(step => (step.failedRequests / step.totalRequests * 100) || 0);
    
    charts.qpsError = new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [
                {
                    label: 'Actual QPS',
                    data: qpsData,
                    borderColor: 'rgb(54, 162, 235)',
                    backgroundColor: 'rgba(54, 162, 235, 0.1)',
                    yAxisID: 'y',
                    tension: 0.1
                },
                {
                    label: 'Error Rate (%)',
                    data: errorRates,
                    borderColor: 'rgb(255, 99, 132)',
                    backgroundColor: 'rgba(255, 99, 132, 0.1)',
                    yAxisID: 'y1',
                    tension: 0.1
                }
            ]
        },
        options: {
            responsive: true,
            interaction: {
                intersect: false,
                mode: 'index'
            },
            scales: {
                y: {
                    type: 'linear',
                    display: true,
                    position: 'left',
                    title: {
                        display: true,
                        text: 'QPS'
                    },
                    beginAtZero: true
                },
                y1: {
                    type: 'linear',
                    display: true,
                    position: 'right',
                    title: {
                        display: true,
                        text: 'Error Rate (%)'
                    },
                    beginAtZero: true,
                    grid: {
                        drawOnChartArea: false
                    }
                }
            }
        }
    });
}

function createQPSProgressionChart(stepsData) {
    const ctx = document.getElementById('qpsProgressionChart').getContext('2d');
    
    const labels = stepsData.map((step, index) => 'Step ' + (index + 1));
    const targetQPS = stepsData.map(step => step.qps);
    const actualQPS = stepsData.map(step => step.actualQPS);
    
    charts.qpsProgression = new Chart(ctx, {
        type: 'bar',
        data: {
            labels: labels,
            datasets: [
                {
                    label: 'Target QPS',
                    data: targetQPS,
                    backgroundColor: 'rgba(54, 162, 235, 0.6)',
                    borderColor: 'rgb(54, 162, 235)',
                    borderWidth: 1
                },
                {
                    label: 'Actual QPS',
                    data: actualQPS,
                    backgroundColor: 'rgba(75, 192, 192, 0.6)',
                    borderColor: 'rgb(75, 192, 192)',
                    borderWidth: 1
                }
            ]
        },
        options: {
            responsive: true,
            scales: {
                y: {
                    beginAtZero: true,
                    title: {
                        display: true,
                        text: 'Queries Per Second'
                    }
                }
            }
        }
    });
}

function createErrorDistributionChart(stepsData) {
    const ctx = document.getElementById('errorDistributionChart').getContext('2d');
    
    // Aggregate all error types across steps
    const errorTypes = new Map();
    stepsData.forEach(step => {
        Object.entries(step.errorDetails || {}).forEach(([type, count]) => {
            errorTypes.set(type, (errorTypes.get(type) || 0) + count);
        });
    });
    
    const labels = Array.from(errorTypes.keys());
    const data = Array.from(errorTypes.values());
    const colors = [
        '#FF6384', '#36A2EB', '#FFCE56', '#4BC0C0',
        '#9966FF', '#FF9F40', '#FF6384', '#C9CBCF'
    ];
    
    charts.errorDistribution = new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: labels.length ? labels : ['No Errors'],
            datasets: [{
                data: data.length ? data : [1],
                backgroundColor: labels.length ? colors.slice(0, labels.length) : ['#4BC0C0'],
                borderWidth: 2
            }]
        },
        options: {
            responsive: true,
            plugins: {
                legend: {
                    position: 'bottom'
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            const total = context.dataset.data.reduce((a, b) => a + b, 0);
                            const percentage = ((context.parsed / total) * 100).toFixed(1);
                            return context.label + ': ' + context.parsed + ' (' + percentage + '%)';
                        }
                    }
                }
            }
        }
    });
}

function createEndpointComparisonChart(stepsData) {
    const ctx = document.getElementById('endpointComparisonChart').getContext('2d');
    
    // Get endpoint names from the latest step
    const latestStep = stepsData[stepsData.length - 1];
    const endpointNames = Object.keys(latestStep.endpointStats || {});
    
    if (!endpointNames.length) {
        // No endpoint data available
        charts.endpointComparison = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: ['No Data'],
                datasets: [{
                    label: 'No endpoint data available',
                    data: [0],
                    backgroundColor: 'rgba(201, 203, 207, 0.6)'
                }]
            },
            options: { responsive: true }
        });
        return;
    }
    
    const avgResponseTimes = endpointNames.map(endpoint => {
        const responseTimes = stepsData.map(step => 
            step.endpointStats[endpoint]?.avgResponseTime || 0
        );
        return responseTimes.reduce((sum, time) => sum + time, 0) / responseTimes.length;
    });
    
    const successRates = endpointNames.map(endpoint => {
        const rates = stepsData.map(step => {
            const stats = step.endpointStats[endpoint];
            return stats ? (stats.successful / stats.total * 100) : 100;
        });
        return rates.reduce((sum, rate) => sum + rate, 0) / rates.length;
    });
    
    charts.endpointComparison = new Chart(ctx, {
        type: 'bar',
        data: {
            labels: endpointNames,
            datasets: [
                {
                    label: 'Avg Response Time (ms)',
                    data: avgResponseTimes,
                    backgroundColor: 'rgba(75, 192, 192, 0.6)',
                    yAxisID: 'y'
                },
                {
                    label: 'Success Rate (%)',
                    data: successRates,
                    backgroundColor: 'rgba(54, 162, 235, 0.6)',
                    yAxisID: 'y1'
                }
            ]
        },
        options: {
            responsive: true,
            scales: {
                y: {
                    type: 'linear',
                    display: true,
                    position: 'left',
                    title: {
                        display: true,
                        text: 'Response Time (ms)'
                    },
                    beginAtZero: true
                },
                y1: {
                    type: 'linear',
                    display: true,
                    position: 'right',
                    title: {
                        display: true,
                        text: 'Success Rate (%)'
                    },
                    beginAtZero: true,
                    max: 100,
                    grid: {
                        drawOnChartArea: false
                    }
                }
            }
        }
    });
}

// Helper functions for latency calculations
function calculateAverageLatency(stepsData) {
    const allLatencies = stepsData.map(step => calculateStepAverageLatency(step));
    return allLatencies.reduce((sum, latency) => sum + latency, 0) / allLatencies.length;
}

function calculateStepAverageLatency(step) {
    const endpointStats = step.endpointStats || {};
    const latencies = Object.values(endpointStats).map(stat => stat.avgResponseTime || 0);
    return latencies.length ? latencies.reduce((sum, lat) => sum + lat, 0) / latencies.length : 0;
}

function calculateStepP95Latency(step) {
    // Simplified P95 calculation - in real implementation, you'd need actual request data
    return calculateStepAverageLatency(step) * 1.5;
}

function calculateStepMaxLatency(step) {
    const endpointStats = step.endpointStats || {};
    const maxLatencies = Object.values(endpointStats).map(stat => stat.maxResponseTime || 0);
    return Math.max(...maxLatencies, 0);
}

async function viewSession(sessionId) {
    try {
        const session = await fetch('/api/sessions/' + sessionId).then(r => r.json());
        showSessionModal(session);
    } catch (error) {
        console.error('Failed to load session details:', error);
        alert('Failed to load session details');
    }
}

function showSessionModal(session) {
    const content = \`
        <h2>Session Details: \${session.sessionId}</h2>
        <div class="session-details">
            <div class="detail-grid">
                <div><strong>Duration:</strong> \${formatDuration(session.endTime - session.startTime)}</div>
                <div><strong>Total Requests:</strong> \${session.overallStats.totalRequests.toLocaleString()}</div>
                <div><strong>Success Rate:</strong> \${session.overallStats.overallSuccessRate.toFixed(2)}%</div>
                <div><strong>Max Sustainable QPS:</strong> \${session.maxSustainableQPS}</div>
                <div><strong>Recommended QPS:</strong> \${session.recommendedMaxQPS}</div>
            </div>
        </div>
    \`;
    
    document.getElementById('modalContent').innerHTML = content;
    document.getElementById('modal').style.display = 'block';
}

async function exportSession(sessionId, format = 'csv') {
    try {
        const response = await fetch('/api/sessions/' + sessionId + '/export/' + format);
        const blob = await response.blob();
        
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.style.display = 'none';
        a.href = url;
        a.download = 'session-' + sessionId + '.' + format;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
    } catch (error) {
        console.error('Export failed:', error);
        alert('Export failed');
    }
}

async function deleteSession(sessionId) {
    if (!confirm('Are you sure you want to delete session ' + sessionId + '?')) {
        return;
    }
    
    try {
        await fetch('/api/sessions/' + sessionId, { method: 'DELETE' });
        refreshSessions();
        alert('Session deleted successfully');
    } catch (error) {
        console.error('Delete failed:', error);
        alert('Delete failed');
    }
}

function refreshSessions() {
    loadInitialData();
}

function filterSessions() {
    const filter = document.getElementById('sessionFilter').value.toLowerCase();
    const rows = document.querySelectorAll('#sessionsTable tbody tr');
    
    rows.forEach(row => {
        const text = row.textContent.toLowerCase();
        row.style.display = text.includes(filter) ? '' : 'none';
    });
}

function closeModal() {
    document.getElementById('modal').style.display = 'none';
}

function formatDuration(ms) {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
        return hours + 'h ' + (minutes % 60) + 'm ' + (seconds % 60) + 's';
    } else if (minutes > 0) {
        return minutes + 'm ' + (seconds % 60) + 's';
    } else {
        return seconds + 's';
    }
}

// Additional functions for other tabs would go here...

// Close modal when clicking outside
window.onclick = function(event) {
    const modal = document.getElementById('modal');
    if (event.target === modal) {
        modal.style.display = 'none';
    }
}`;
  }

  private generateDashboardCSS(): string {
    return `/* Dashboard Styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: #f5f7fa;
    color: #333;
    line-height: 1.6;
}

.dashboard {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
}

/* Header */
.header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 1rem 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.header h1 {
    font-size: 1.5rem;
    font-weight: 600;
}

.header-stats {
    display: flex;
    gap: 1rem;
}

.stat-card {
    background: rgba(255, 255, 255, 0.1);
    padding: 0.5rem 1rem;
    border-radius: 8px;
    text-align: center;
    backdrop-filter: blur(10px);
}

.stat-label {
    display: block;
    font-size: 0.8rem;
    opacity: 0.8;
}

.stat-value {
    display: block;
    font-size: 1.2rem;
    font-weight: bold;
}

/* Navigation */
.nav-tabs {
    background: white;
    border-bottom: 1px solid #e1e5e9;
    padding: 0 2rem;
    display: flex;
}

.tab-button {
    background: none;
    border: none;
    padding: 1rem 1.5rem;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    color: #6c757d;
    border-bottom: 3px solid transparent;
    transition: all 0.3s ease;
}

.tab-button:hover {
    color: #495057;
    background-color: #f8f9fa;
}

.tab-button.active {
    color: #667eea;
    border-bottom-color: #667eea;
}

/* Content */
.content {
    flex: 1;
    padding: 2rem;
}

.tab-content {
    display: none;
}

.tab-content.active {
    display: block;
}

/* Grid Layouts */
.grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
}

.grid-3 {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
}

/* Metrics Grid */
.metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
}

.metric-card {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 1.5rem;
    border-radius: 12px;
    text-align: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    transition: transform 0.3s ease;
}

.metric-card:hover {
    transform: translateY(-2px);
}

.metric-value {
    font-size: 2rem;
    font-weight: bold;
    margin-bottom: 0.5rem;
    color: white;
}

.metric-label {
    font-size: 0.875rem;
    opacity: 0.9;
    margin-bottom: 0.5rem;
}

.metric-trend {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.2);
    display: inline-block;
}

.metric-trend.positive {
    background: rgba(72, 187, 120, 0.3);
    color: #68D391;
}

.metric-trend.negative {
    background: rgba(245, 101, 101, 0.3);
    color: #FC8181;
}

/* Cards */
.card {
    background: white;
    border-radius: 12px;
    padding: 1.5rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    border: 1px solid #e1e5e9;
}

.card h3 {
    color: #495057;
    margin-bottom: 1rem;
    font-size: 1.1rem;
    font-weight: 600;
}

.card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.controls {
    display: flex;
    gap: 0.5rem;
    align-items: center;
}

/* Tables */
.table-container {
    overflow-x: auto;
}

.data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
}

.data-table th,
.data-table td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid #dee2e6;
}

.data-table th {
    background-color: #f8f9fa;
    font-weight: 600;
    color: #495057;
}

.data-table tr:hover {
    background-color: #f8f9fa;
}

/* Buttons */
button {
    background: #667eea;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background-color 0.3s ease;
}

button:hover {
    background: #5a67d8;
}

button.danger {
    background: #e53e3e;
}

button.danger:hover {
    background: #c53030;
}

/* Inputs */
input[type="text"],
select {
    padding: 0.5rem;
    border: 1px solid #cbd5e0;
    border-radius: 6px;
    font-size: 0.875rem;
}

input[type="text"]:focus,
select:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

/* Status indicators */
.status {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
}

.status.completed {
    background: #c6f6d5;
    color: #22543d;
}

.status.running {
    background: #bee3f8;
    color: #2c5282;
}

.status.failed {
    background: #fed7d7;
    color: #742a2a;
}

/* Modal */
.modal {
    display: none;
    position: fixed;
    z-index: 1000;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
}

.modal-content {
    background-color: white;
    margin: 5% auto;
    padding: 2rem;
    border-radius: 12px;
    width: 80%;
    max-width: 800px;
    max-height: 80vh;
    overflow-y: auto;
    position: relative;
}

.close {
    position: absolute;
    right: 1rem;
    top: 1rem;
    color: #aaa;
    font-size: 28px;
    font-weight: bold;
    cursor: pointer;
}

.close:hover {
    color: #000;
}

/* Session Details */
.session-details {
    margin-top: 1rem;
}

.detail-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin-bottom: 2rem;
}


.session-list {
    max-height: 400px;
    overflow-y: auto;
}

.session-item {
    padding: 0.75rem;
    border-bottom: 1px solid #e1e5e9;
    cursor: pointer;
    transition: background-color 0.3s ease;
}

.session-item:hover {
    background-color: #f8f9fa;
}

/* Charts */
canvas {
    max-height: 350px;
    min-height: 250px;
}

/* Chart containers */
.card canvas {
    margin-top: 1rem;
}

/* Chart titles */
.card h3 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.card h3::before {
    content: '📊';
    font-size: 1rem;
}

/* Special chart styling */
.card:has(#latencyChart) h3::before {
    content: '⚡';
}

.card:has(#qpsErrorChart) h3::before {
    content: '📈';
}

.card:has(#errorDistributionChart) h3::before {
    content: '🎯';
}

/* Responsive Design */
@media (max-width: 1024px) {
    .metrics-grid {
        grid-template-columns: repeat(2, 1fr);
    }
    
    .grid-3 {
        grid-template-columns: 1fr 1fr;
    }
}

@media (max-width: 768px) {
    .grid-2,
    .grid-3 {
        grid-template-columns: 1fr;
    }
    
    .metrics-grid {
        grid-template-columns: 1fr 1fr;
    }
    
    .header {
        flex-direction: column;
        gap: 1rem;
    }
    
    .nav-tabs {
        overflow-x: auto;
    }
    
    .content {
        padding: 1rem;
    }
    
    .controls {
        flex-direction: column;
        align-items: stretch;
    }
    
    .detail-grid {
        grid-template-columns: 1fr;
    }
}

@media (max-width: 480px) {
    .metrics-grid {
        grid-template-columns: 1fr;
    }
    
    .metric-card {
        padding: 1rem;
    }
    
    .metric-value {
        font-size: 1.5rem;
    }
}

/* Loading states */
.loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    color: #6c757d;
}

.loading::after {
    content: '';
    width: 20px;
    height: 20px;
    border: 2px solid #f3f3f3;
    border-top: 2px solid #667eea;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-left: 10px;
}

@keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}

/* Links */
a {
    color: #667eea;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}`;
  }

  // API Route Handlers
  private async handleGetSessions(req: express.Request, res: express.Response): Promise<void> {
    try {
      const sessions = this.getSessionList();
      res.json(sessions);
    } catch (error) {
      res.status(500).json({ error: 'Failed to load sessions' });
    }
  }

  private async handleGetSession(req: express.Request, res: express.Response): Promise<void> {
    try {
      const { sessionId } = req.params;
      const sessionPath = path.join(this.config.outputDir, sessionId, 'session-result.json');
      
      if (!fs.existsSync(sessionPath)) {
        res.status(404).json({ error: 'Session not found' });
        return;
      }

      const sessionData = JSON.parse(fs.readFileSync(sessionPath, 'utf-8'));
      res.json(sessionData);
    } catch (error) {
      res.status(500).json({ error: 'Failed to load session' });
    }
  }


  private async handleGetRequests(req: express.Request, res: express.Response): Promise<void> {
    try {
      const { sessionId } = req.params;
      const requestsDir = path.join(this.config.outputDir, sessionId, 'requests');
      
      if (!fs.existsSync(requestsDir)) {
        res.status(404).json({ error: 'Requests data not found' });
        return;
      }

      // Load and aggregate request data
      const requests: any[] = [];
      const files = fs.readdirSync(requestsDir).filter(file => file.endsWith('.jsonl'));
      
      for (const file of files) {
        const content = fs.readFileSync(path.join(requestsDir, file), 'utf-8');
        const lines = content.trim().split('\n').filter(line => line.trim());
        
        for (const line of lines) {
          try {
            requests.push(JSON.parse(line));
          } catch (error) {
            console.warn('Failed to parse request line:', line);
          }
        }
      }

      res.json(requests.sort((a, b) => a.timestamp - b.timestamp));
    } catch (error) {
      res.status(500).json({ error: 'Failed to load requests' });
    }
  }

  private async handleGetSteps(req: express.Request, res: express.Response): Promise<void> {
    try {
      const { sessionId } = req.params;
      const stepsDir = path.join(this.config.outputDir, sessionId, 'steps');
      
      if (!fs.existsSync(stepsDir)) {
        res.status(404).json({ error: 'Steps data not found' });
        return;
      }

      const steps: any[] = [];
      const files = fs.readdirSync(stepsDir).filter(file => file.endsWith('.json'));
      
      for (const file of files) {
        try {
          const content = fs.readFileSync(path.join(stepsDir, file), 'utf-8');
          steps.push(JSON.parse(content));
        } catch (error) {
          console.warn('Failed to parse step file:', file);
        }
      }

      res.json(steps.sort((a, b) => a.qps - b.qps));
    } catch (error) {
      res.status(500).json({ error: 'Failed to load steps' });
    }
  }

  private async handleExport(req: express.Request, res: express.Response): Promise<void> {
    try {
      const { sessionId, format } = req.params;
      
      if (format !== 'csv' && format !== 'json') {
        res.status(400).json({ error: 'Unsupported export format' });
        return;
      }

      const exportPath = path.join(this.config.outputDir, sessionId, 'exports');
      
      if (!fs.existsSync(exportPath)) {
        fs.mkdirSync(exportPath, { recursive: true });
      }

      // Generate export file
      const filename = format === 'csv' ? 'export.csv' : 'export.json';
      const filePath = path.join(exportPath, filename);
      
      if (format === 'csv') {
        await this.generateCSVExport(sessionId, filePath);
      } else {
        await this.generateJSONExport(sessionId, filePath);
      }

      res.download(filePath, 'session-' + sessionId + '.' + format);
    } catch (error) {
      res.status(500).json({ error: 'Export failed' });
    }
  }

  private async handleDeleteSession(req: express.Request, res: express.Response): Promise<void> {
    try {
      const { sessionId } = req.params;
      const sessionPath = path.join(this.config.outputDir, sessionId);
      
      if (fs.existsSync(sessionPath)) {
        fs.rmSync(sessionPath, { recursive: true, force: true });
      }

      res.json({ success: true });
    } catch (error) {
      res.status(500).json({ error: 'Failed to delete session' });
    }
  }

  private async handleGetOverviewStats(req: express.Request, res: express.Response): Promise<void> {
    try {
      const sessions = this.getSessionList();
      const totalRequests = sessions.reduce((sum, s) => sum + s.totalRequests, 0);
      const activeSessions = sessions.filter(s => s.status === 'running').length;

      res.json({
        totalSessions: sessions.length,
        activeTests: activeSessions,
        totalRequests
      });
    } catch (error) {
      res.status(500).json({ error: 'Failed to load overview stats' });
    }
  }


  private async handleSessionUpdate(req: express.Request, res: express.Response): Promise<void> {
    const { sessionId } = req.params;
    const updateData = req.body;
    
    // Broadcast update to subscribed clients
    this.broadcastToSubscribers(sessionId, updateData);
    
    res.json({ success: true });
  }

  // Helper methods
  private getSessionList(): SessionSummary[] {
    if (!fs.existsSync(this.config.outputDir)) {
      return [];
    }

    const sessions: SessionSummary[] = [];
    const entries = fs.readdirSync(this.config.outputDir, { withFileTypes: true });
    
    for (const entry of entries) {
      if (entry.isDirectory() && entry.name.startsWith('stress_test_')) {
        try {
          const sessionPath = path.join(this.config.outputDir, entry.name, 'session-result.json');
          
          if (fs.existsSync(sessionPath)) {
            const sessionData = JSON.parse(fs.readFileSync(sessionPath, 'utf-8'));
            
            sessions.push({
              sessionId: sessionData.sessionId,
              startTime: sessionData.startTime,
              endTime: sessionData.endTime,
              duration: sessionData.endTime - sessionData.startTime,
              totalRequests: sessionData.overallStats.totalRequests,
              successRate: sessionData.overallStats.overallSuccessRate,
              maxQPS: sessionData.maxSustainableQPS,
              avgResponseTime: sessionData.overallStats.avgResponseTime || 0,
              status: 'completed'
            });
          }
        } catch (error) {
          console.warn('Failed to load session ' + entry.name + ':', error);
        }
      }
    }

    return sessions.sort((a, b) => b.startTime - a.startTime);
  }

  private async generateCSVExport(sessionId: string, filePath: string): Promise<void> {
    // Simple CSV export implementation
    const requestsPath = path.join(this.config.outputDir, sessionId, 'requests');
    
    if (!fs.existsSync(requestsPath)) {
      throw new Error('No request data found');
    }

    const csvLines = ['timestamp,endpoint,success,responseTime,requestSize,responseSize,qps,error'];
    const files = fs.readdirSync(requestsPath).filter(file => file.endsWith('.jsonl'));
    
    for (const file of files) {
      const content = fs.readFileSync(path.join(requestsPath, file), 'utf-8');
      const lines = content.trim().split('\n').filter(line => line.trim());
      
      for (const line of lines) {
        try {
          const request = JSON.parse(line);
          csvLines.push([
            new Date(request.timestamp).toISOString(),
            request.endpoint,
            request.success,
            request.responseTime,
            request.requestSize,
            request.responseSize,
            request.qps,
            request.error ? request.error.replace(/,/g, ';') : ''
          ].join(','));
        } catch (error) {
          console.warn('Failed to parse request line for CSV export:', line);
        }
      }
    }

    fs.writeFileSync(filePath, csvLines.join('\n'));
  }

  private async generateJSONExport(sessionId: string, filePath: string): Promise<void> {
    // Load session data
    const sessionPath = path.join(this.config.outputDir, sessionId, 'session-result.json');
    
    const exportData: any = {};
    
    if (fs.existsSync(sessionPath)) {
      exportData.session = JSON.parse(fs.readFileSync(sessionPath, 'utf-8'));
    }

    fs.writeFileSync(filePath, JSON.stringify(exportData, null, 2));
  }

  // Public methods
  public async start(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.server.listen(this.config.port, this.config.host, () => {
        const url = `http://localhost:${this.config.port}`;
        console.log('🌐 Dashboard server started at ' + url);
        
        // Auto-open browser
        this.openBrowser(url);
        
        resolve();
      });

      this.server.on('error', (error) => {
        console.error('Dashboard server error:', error);
        reject(error);
      });
    });
  }

  private openBrowser(url: string): void {
    let command: string;
    
    switch (process.platform) {
      case 'darwin': // macOS
        command = `open "${url}"`;
        break;
      case 'win32': // Windows
        command = `start "" "${url}"`;
        break;
      default: // Linux and others
        command = `xdg-open "${url}"`;
        break;
    }
    
    exec(command, (error) => {
      if (error) {
        console.log('💡 Open browser manually: ' + url);
      } else {
        console.log('🚀 Dashboard opened in browser automatically');
      }
    });
  }

  public async stop(): Promise<void> {
    return new Promise((resolve) => {
      this.server.close(() => {
        console.log('Dashboard server stopped');
        resolve();
      });
    });
  }

  public notifySessionUpdate(sessionId: string, data: any): void {
    this.broadcastToSubscribers(sessionId, data);
  }

  public getUrl(): string {
    return 'http://' + this.config.host + ':' + this.config.port;
  }
}

// CLI execution
async function main() {
  try {
    const outputDir = process.argv[2] || './stress-test-results';
    console.log('🚀 Starting Dashboard Server...');
    console.log('📂 Output directory:', outputDir);
    
    const dashboard = new Dashboard(outputDir);
    await dashboard.start();
    
    // Keep the process running
    process.on('SIGINT', async () => {
      console.log('\n👋 Shutting down dashboard...');
      await dashboard.stop();
      process.exit(0);
    });
    
  } catch (error) {
    console.error('❌ Failed to start dashboard:', error);
    process.exit(1);
  }
}

// Run if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}