class StressDashboard {
    constructor() {
        this.ws = null;
        this.charts = {};
        this.data = {
            responseTime: [],
            rps: [],
            cpu: [],
            memory: [],
            errors: new Map()
        };
        this.isConnected = false;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        
        this.init();
    }

    init() {
        this.connectWebSocket();
        this.initializeCharts();
        this.setupEventListeners();
        this.startDataCleanup();
    }

    connectWebSocket() {
        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${wsProtocol}//${window.location.host}`;
        
        try {
            this.ws = new WebSocket(wsUrl);
            
            this.ws.onopen = () => {
                console.log('Connected to dashboard server');
                this.isConnected = true;
                this.reconnectAttempts = 0;
                this.addLog('Connected to dashboard server', 'success');
            };
            
            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleMessage(message);
                } catch (error) {
                    console.error('Error parsing WebSocket message:', error);
                }
            };
            
            this.ws.onclose = () => {
                this.isConnected = false;
                this.addLog('Disconnected from dashboard server', 'error');
                this.attemptReconnect();
            };
            
            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                this.addLog('WebSocket connection error', 'error');
            };
            
        } catch (error) {
            console.error('Failed to create WebSocket connection:', error);
            this.addLog('Failed to connect to dashboard server', 'error');
        }
    }

    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            setTimeout(() => {
                this.addLog(`Reconnection attempt ${this.reconnectAttempts}...`, 'info');
                this.connectWebSocket();
            }, 2000 * this.reconnectAttempts);
        }
    }

    handleMessage(message) {
        switch (message.type) {
            case 'status':
                this.updateStatus(message.data);
                break;
            case 'testStarted':
                this.handleTestStarted(message.data);
                break;
            case 'testCompleted':
                this.handleTestCompleted(message.data);
                break;
            case 'testFailed':
                this.handleTestFailed(message.data);
                break;
            case 'requestCompleted':
                this.handleRequestCompleted(message.data);
                break;
            case 'requestFailed':
                this.handleRequestFailed(message.data);
                break;
            case 'systemMetrics':
                this.handleSystemMetrics(message.data);
                break;
        }
    }

    updateStatus(data) {
        const statusElement = document.getElementById('status');
        const statusText = document.getElementById('statusText');
        const statusDetails = document.getElementById('statusDetails');
        
        statusElement.className = `status ${data.status}`;
        statusText.textContent = data.status.charAt(0).toUpperCase() + data.status.slice(1);
        
        if (data.session) {
            statusDetails.textContent = `Session: ${data.session.sessionId} | Test: ${data.session.config.testName}`;
        } else {
            statusDetails.textContent = 'Ready to start stress test';
        }
        
        // Update button states
        const startBtn = document.getElementById('startTest');
        const stopBtn = document.getElementById('stopTest');
        
        if (data.status === 'running') {
            startBtn.disabled = true;
            stopBtn.disabled = false;
        } else {
            startBtn.disabled = false;
            stopBtn.disabled = true;
        }
    }

    handleTestStarted(data) {
        this.addLog(`Test started: ${data.sessionId}`, 'success');
        this.clearCharts();
    }

    handleTestCompleted(data) {
        this.addLog(`Test completed: ${data.sessionId}`, 'success');
        if (data.summary) {
            this.addLog(`Total requests: ${data.summary.totalRequests}, Success rate: ${((data.summary.successfulRequests / data.summary.totalRequests) * 100).toFixed(2)}%`, 'info');
        }
    }

    handleTestFailed(data) {
        this.addLog(`Test failed: ${data.error}`, 'error');
    }

    handleRequestCompleted(data) {
        const now = Date.now();
        this.data.responseTime.push({
            x: now,
            y: data.responseTime
        });
        
        this.updateRealTimeMetrics();
        this.updateCharts();
    }

    handleRequestFailed(data) {
        const errorType = this.categorizeError(data.error || 'Unknown');
        const count = this.data.errors.get(errorType) || 0;
        this.data.errors.set(errorType, count + 1);
        
        this.updateRealTimeMetrics();
        this.updateCharts();
        
        this.addLog(`Request failed: ${data.error}`, 'error');
    }

    handleSystemMetrics(data) {
        const now = Date.now();
        
        this.data.cpu.push({
            x: now,
            y: data.cpu.usage
        });
        
        this.data.memory.push({
            x: now,
            y: data.memory.usage
        });
        
        // Update current metrics
        document.getElementById('cpuUsage').textContent = `${data.cpu.usage.toFixed(1)}%`;
        document.getElementById('memoryUsage').textContent = `${data.memory.usage.toFixed(1)}%`;
        
        this.updateCharts();
    }

    categorizeError(error) {
        const errorLower = error.toLowerCase();
        
        if (errorLower.includes('timeout')) return 'Timeout';
        if (errorLower.includes('connection')) return 'Connection';
        if (errorLower.includes('dns')) return 'DNS';
        if (errorLower.includes('ssl') || errorLower.includes('tls')) return 'SSL/TLS';
        if (errorLower.includes('auth')) return 'Authentication';
        
        return 'Other';
    }

    updateRealTimeMetrics() {
        const now = Date.now();
        const lastMinute = now - 60000;
        
        // Filter data for last minute
        const recentRequests = this.data.responseTime.filter(point => point.x > lastMinute);
        const recentResponseTimes = recentRequests.map(point => point.y);
        
        // Calculate RPS (requests in last minute / 60)
        const rps = recentRequests.length / 60;
        document.getElementById('currentRps').textContent = rps.toFixed(1);
        
        // Calculate average response time
        const avgResponseTime = recentResponseTimes.length > 0 
            ? recentResponseTimes.reduce((a, b) => a + b, 0) / recentResponseTimes.length 
            : 0;
        document.getElementById('avgResponseTime').textContent = `${avgResponseTime.toFixed(0)}ms`;
        
        // Update total requests
        document.getElementById('totalRequests').textContent = this.data.responseTime.length;
        
        // Calculate error rate
        const totalErrors = Array.from(this.data.errors.values()).reduce((a, b) => a + b, 0);
        const totalRequests = this.data.responseTime.length + totalErrors;
        const errorRate = totalRequests > 0 ? (totalErrors / totalRequests) * 100 : 0;
        
        const errorRateElement = document.getElementById('errorRate');
        errorRateElement.textContent = `${errorRate.toFixed(1)}%`;
        errorRateElement.className = `metric-value ${this.getErrorRateClass(errorRate)}`;
        
        // Update RPS data for chart
        this.data.rps.push({
            x: now,
            y: rps
        });
    }

    getErrorRateClass(errorRate) {
        if (errorRate === 0) return 'success';
        if (errorRate < 5) return 'warning';
        return 'danger';
    }

    initializeCharts() {
        // Response Time Chart
        this.charts.responseTime = new Chart(document.getElementById('responseTimeChart'), {
            type: 'line',
            data: {
                datasets: [{
                    label: 'Response Time (ms)',
                    data: this.data.responseTime,
                    borderColor: 'rgb(75, 192, 192)',
                    backgroundColor: 'rgba(75, 192, 192, 0.2)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    x: {
                        type: 'time',
                        time: {
                            unit: 'second'
                        }
                    },
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Response Time (ms)'
                        }
                    }
                },
                plugins: {
                    legend: {
                        display: false
                    }
                }
            }
        });

        // RPS Chart
        this.charts.rps = new Chart(document.getElementById('rpsChart'), {
            type: 'line',
            data: {
                datasets: [{
                    label: 'Requests per Second',
                    data: this.data.rps,
                    borderColor: 'rgb(255, 99, 132)',
                    backgroundColor: 'rgba(255, 99, 132, 0.2)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    x: {
                        type: 'time',
                        time: {
                            unit: 'second'
                        }
                    },
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Requests per Second'
                        }
                    }
                },
                plugins: {
                    legend: {
                        display: false
                    }
                }
            }
        });

        // System Resources Chart
        this.charts.system = new Chart(document.getElementById('systemChart'), {
            type: 'line',
            data: {
                datasets: [{
                    label: 'CPU Usage (%)',
                    data: this.data.cpu,
                    borderColor: 'rgb(54, 162, 235)',
                    backgroundColor: 'rgba(54, 162, 235, 0.2)',
                    tension: 0.4
                }, {
                    label: 'Memory Usage (%)',
                    data: this.data.memory,
                    borderColor: 'rgb(255, 206, 86)',
                    backgroundColor: 'rgba(255, 206, 86, 0.2)',
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    x: {
                        type: 'time',
                        time: {
                            unit: 'second'
                        }
                    },
                    y: {
                        beginAtZero: true,
                        max: 100,
                        title: {
                            display: true,
                            text: 'Usage (%)'
                        }
                    }
                }
            }
        });

        // Error Distribution Chart
        this.charts.errors = new Chart(document.getElementById('errorChart'), {
            type: 'doughnut',
            data: {
                labels: [],
                datasets: [{
                    data: [],
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.8)',
                        'rgba(54, 162, 235, 0.8)',
                        'rgba(255, 205, 86, 0.8)',
                        'rgba(75, 192, 192, 0.8)',
                        'rgba(153, 102, 255, 0.8)'
                    ]
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false
            }
        });
    }

    updateCharts() {
        // Update all charts
        Object.values(this.charts).forEach(chart => {
            if (chart && typeof chart.update === 'function') {
                chart.update('none');
            }
        });

        // Update error chart data
        if (this.charts.errors) {
            const errorLabels = Array.from(this.data.errors.keys());
            const errorValues = Array.from(this.data.errors.values());
            
            this.charts.errors.data.labels = errorLabels;
            this.charts.errors.data.datasets[0].data = errorValues;
            this.charts.errors.update('none');
        }
    }

    clearCharts() {
        this.data.responseTime = [];
        this.data.rps = [];
        this.data.cpu = [];
        this.data.memory = [];
        this.data.errors.clear();
        
        this.updateCharts();
    }

    setupEventListeners() {
        document.getElementById('startTest').addEventListener('click', () => {
            this.startTest();
        });

        document.getElementById('stopTest').addEventListener('click', () => {
            this.stopTest();
        });

        document.getElementById('clearResults').addEventListener('click', () => {
            this.clearResults();
        });
    }

    async startTest() {
        const config = {
            testName: document.getElementById('testName').value,
            duration: parseInt(document.getElementById('duration').value),
            concurrency: parseInt(document.getElementById('concurrency').value),
            rampUpTime: parseInt(document.getElementById('rampUpTime').value),
            rampDownTime: 10,
            targetUrl: document.getElementById('targetUrl').value,
            requestTimeout: parseInt(document.getElementById('requestTimeout').value),
            retries: 3,
            outputFormat: ['json', 'console'],
            outputPath: './results',
            enableDashboard: true,
            dashboardPort: 3000
        };

        try {
            const response = await fetch('/api/start-test', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(config)
            });

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const result = await response.json();
            this.addLog(`Test started with session ID: ${result.sessionId}`, 'success');
            
        } catch (error) {
            this.addLog(`Failed to start test: ${error.message}`, 'error');
        }
    }

    async stopTest() {
        try {
            const response = await fetch('/api/stop-test', {
                method: 'POST'
            });

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            this.addLog('Test stopped', 'info');
            
        } catch (error) {
            this.addLog(`Failed to stop test: ${error.message}`, 'error');
        }
    }

    clearResults() {
        this.clearCharts();
        document.getElementById('logContainer').innerHTML = '';
        
        // Reset metrics
        document.getElementById('currentRps').textContent = '0';
        document.getElementById('errorRate').textContent = '0%';
        document.getElementById('avgResponseTime').textContent = '0ms';
        document.getElementById('totalRequests').textContent = '0';
        document.getElementById('cpuUsage').textContent = '0%';
        document.getElementById('memoryUsage').textContent = '0%';
        
        this.addLog('Results cleared', 'info');
    }

    addLog(message, type = 'info') {
        const container = document.getElementById('logContainer');
        const logEntry = document.createElement('div');
        logEntry.className = `log-entry ${type}`;
        
        const timestamp = new Date().toLocaleTimeString();
        logEntry.textContent = `[${timestamp}] ${message}`;
        
        container.appendChild(logEntry);
        container.scrollTop = container.scrollHeight;
        
        // Keep only last 100 log entries
        while (container.children.length > 100) {
            container.removeChild(container.firstChild);
        }
    }

    startDataCleanup() {
        // Clean old data every 30 seconds to prevent memory issues
        setInterval(() => {
            const now = Date.now();
            const maxAge = 10 * 60 * 1000; // 10 minutes
            
            this.data.responseTime = this.data.responseTime.filter(point => now - point.x < maxAge);
            this.data.rps = this.data.rps.filter(point => now - point.x < maxAge);
            this.data.cpu = this.data.cpu.filter(point => now - point.x < maxAge);
            this.data.memory = this.data.memory.filter(point => now - point.x < maxAge);
        }, 30000);
    }
}

// Initialize dashboard when page loads
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new StressDashboard();
});