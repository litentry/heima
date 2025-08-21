// Dashboard JavaScript
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
        console.log('Loading initial data...');
        const [sessions, stats] = await Promise.all([
            fetch('/api/sessions').then(r => {
                console.log('Sessions response:', r.status);
                return r.json();
            }),
            fetch('/api/stats/overview').then(r => {
                console.log('Stats response:', r.status);
                return r.json();
            })
        ]);
        
        console.log('Loaded sessions:', sessions);
        console.log('Loaded stats:', stats);
        
        updateSessionsTable(sessions);
        updateOverviewStats(stats);
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
        row.innerHTML = `
            <td><a href="#" onclick="viewSession('${session.sessionId}')">${session.sessionId}</a></td>
            <td>${new Date(session.startTime).toLocaleString()}</td>
            <td>${formatDuration(session.duration)}</td>
            <td>${session.totalRequests.toLocaleString()}</td>
            <td>${session.maxQPS}</td>
            <td>${session.successRate.toFixed(1)}%</td>
            <td><span class="status ${session.status}">${session.status}</span></td>
            <td>
                <button onclick="viewSession('${session.sessionId}')">View</button>
                <button onclick="exportSession('${session.sessionId}')">Export</button>
                <button onclick="deleteSession('${session.sessionId}')" class="danger">Delete</button>
            </td>
        `;
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
        const [session, analysis] = await Promise.all([
            fetch('/api/sessions/' + sessionId).then(r => r.json()),
            fetch('/api/sessions/' + sessionId + '/analysis').then(r => r.json())
        ]);
        
        showSessionModal(session, analysis);
    } catch (error) {
        console.error('Failed to load session details:', error);
        alert('Failed to load session details');
    }
}

function showSessionModal(session, analysis) {
    const content = `
        <h2>Session Details: ${session.sessionId}</h2>
        <div class="session-details">
            <div class="detail-grid">
                <div><strong>Duration:</strong> ${formatDuration(session.endTime - session.startTime)}</div>
                <div><strong>Total Requests:</strong> ${session.overallStats.totalRequests.toLocaleString()}</div>
                <div><strong>Success Rate:</strong> ${session.overallStats.overallSuccessRate.toFixed(2)}%</div>
                <div><strong>Max Sustainable QPS:</strong> ${session.maxSustainableQPS}</div>
                <div><strong>Recommended QPS:</strong> ${session.recommendedMaxQPS}</div>
            </div>
            
            ${analysis ? `
                <h3>Performance Analysis</h3>
                <div class="analysis-summary">
                    <p><strong>Peak QPS:</strong> ${analysis.summary.peakQPS}</p>
                    <p><strong>Average Response Time:</strong> ${analysis.summary.averageResponseTime.toFixed(2)}ms</p>
                    <p><strong>Top Errors:</strong></p>
                    <ul>
                        ' + analysis.summary.topErrors.map(error => 
                            '<li>' + error.type + ': ' + error.count + ' (' + error.percentage.toFixed(1) + '%)</li>'
                        ).join('') + '
                    </ul>
                </div>
            ` : '<p>Analysis not available</p>'}
        </div>
    `;
    
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
}