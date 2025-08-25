import React, { useState, useEffect } from 'react';
import { QPSStepResult } from '../types';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js';
import { Line, Bar, Doughnut } from 'react-chartjs-2';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend
);

interface SessionData {
  sessionId: string;
  startTime: number;
  endTime: number;
  duration: number;
  status: string;
  config: any;
  maxSustainableQPS: number;
  recommendedMaxQPS: number;
  overallStats: {
    totalRequests: number;
    totalSuccessful: number;
    totalFailed: number;
    overallSuccessRate: number;
    avgResponseTime: number;
    maxResponseTime: number;
    minResponseTime: number;
  };
}


interface DetailedSessionData {
  sessionId: string;
  startTime: number;
  endTime: number;
  config: any;
  steps: QPSStepResult[];
  maxSustainableQPS: number;
  recommendedMaxQPS: number;
  overallStats: {
    totalRequests: number;
    totalSuccessful: number;
    totalFailed: number;
    overallSuccessRate: number;
    avgResponseTime: number;
    maxResponseTime: number;
    minResponseTime: number;
  };
}

interface OverviewStats {
  totalSessions: number;
  totalRequests: number;
  avgQPS: number;
  avgSuccessRate: number;
  avgResponseTime: number;
}

export default function Dashboard() {
  const [sessions, setSessions] = useState<SessionData[]>([]);
  const [selectedSession, setSelectedSession] = useState<SessionData | null>(null);
  const [selectedSessionDetails, setSelectedSessionDetails] = useState<DetailedSessionData | null>(null);
  const [loadingDetails, setLoadingDetails] = useState(false);
  const [overviewStats, setOverviewStats] = useState<OverviewStats>({
    totalSessions: 0,
    totalRequests: 0,
    avgQPS: 0,
    avgSuccessRate: 0,
    avgResponseTime: 0
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadInitialData();
    const interval = setInterval(loadInitialData, 30000); // Refresh every 30 seconds
    return () => clearInterval(interval);
  }, []);

  const loadInitialData = async () => {
    try {
      const response = await fetch('/api/sessions');
      if (!response.ok) {
        throw new Error(`Failed to fetch sessions: ${response.statusText}`);
      }
      const data = await response.json();
      setSessions(data);
      
      if (data.length > 0) {
        if (!selectedSession) {
          const firstSession = data[0];
          setSelectedSession(firstSession);
          loadSessionDetails(firstSession.sessionId);
        }
        calculateOverviewStats(data);
      }
      
      setLoading(false);
    } catch (err) {
      console.error('Error loading data:', err);
      setError(err instanceof Error ? err.message : 'Unknown error');
      setLoading(false);
    }
  };

  const calculateOverviewStats = (sessionsData: SessionData[]) => {
    if (sessionsData.length === 0) return;

    const totalRequests = sessionsData.reduce((sum, s) => sum + s.overallStats.totalRequests, 0);
    const avgSuccessRate = sessionsData.reduce((sum, s) => sum + s.overallStats.overallSuccessRate, 0) / sessionsData.length;
    const avgQPS = sessionsData.reduce((sum, s) => sum + s.maxSustainableQPS, 0) / sessionsData.length;
    const avgResponseTime = sessionsData.reduce((sum, s) => sum + (s.overallStats.avgResponseTime || 0), 0) / sessionsData.length;

    setOverviewStats({
      totalSessions: sessionsData.length,
      totalRequests,
      avgQPS,
      avgSuccessRate,
      avgResponseTime
    });
  };

  const loadSessionDetails = async (sessionId: string) => {
    setLoadingDetails(true);
    try {
      const response = await fetch(`/api/sessions/${sessionId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch session details: ${response.statusText}`);
      }
      const detailsData = await response.json();
      setSelectedSessionDetails(detailsData);
    } catch (err) {
      console.error('Error loading session details:', err);
      setSelectedSessionDetails(null);
    } finally {
      setLoadingDetails(false);
    }
  };

  const handleSessionSelect = (session: SessionData) => {
    setSelectedSession(session);
    loadSessionDetails(session.sessionId);
  };

  // Chart configurations
  const qpsChartData = {
    labels: sessions.map((_, index) => `Session ${index + 1}`),
    datasets: [
      {
        label: 'Max Sustainable QPS',
        data: sessions.map(s => s.maxSustainableQPS),
        backgroundColor: 'rgba(54, 162, 235, 0.6)',
        borderColor: 'rgba(54, 162, 235, 1)',
        borderWidth: 2,
      },
      {
        label: 'Recommended QPS',
        data: sessions.map(s => s.recommendedMaxQPS),
        backgroundColor: 'rgba(255, 99, 132, 0.6)',
        borderColor: 'rgba(255, 99, 132, 1)',
        borderWidth: 2,
      }
    ],
  };

  const successRateChartData = {
    labels: sessions.map(s => new Date(s.startTime).toLocaleDateString()),
    datasets: [
      {
        label: 'Success Rate (%)',
        data: sessions.map(s => s.overallStats.overallSuccessRate),
        backgroundColor: sessions.map(s => 
          s.overallStats.overallSuccessRate >= 95 ? 'rgba(75, 192, 192, 0.6)' :
          s.overallStats.overallSuccessRate >= 90 ? 'rgba(255, 205, 86, 0.6)' :
          'rgba(255, 99, 132, 0.6)'
        ),
        borderColor: 'rgba(75, 192, 192, 1)',
        borderWidth: 2,
      }
    ],
  };

  const statusDistribution = sessions.reduce((acc, session) => {
    acc[session.status] = (acc[session.status] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const statusChartData = {
    labels: Object.keys(statusDistribution),
    datasets: [
      {
        data: Object.values(statusDistribution),
        backgroundColor: [
          'rgba(75, 192, 192, 0.6)',
          'rgba(255, 99, 132, 0.6)',
          'rgba(255, 205, 86, 0.6)',
          'rgba(153, 102, 255, 0.6)',
        ],
      }
    ],
  };

  // Session Details Charts
  const sessionQPSPerformanceData = selectedSessionDetails ? {
    labels: selectedSessionDetails.steps.map((_, index) => `Step ${index + 1}`),
    datasets: [
      {
        label: 'Target QPS',
        data: selectedSessionDetails.steps.map(step => step.qps),
        borderColor: 'rgba(54, 162, 235, 1)',
        backgroundColor: 'rgba(54, 162, 235, 0.1)',
        borderWidth: 2,
        fill: false,
        tension: 0.1,
      },
      {
        label: 'Actual QPS',
        data: selectedSessionDetails.steps.map(step => step.actualQPS),
        borderColor: 'rgba(255, 99, 132, 1)',
        backgroundColor: 'rgba(255, 99, 132, 0.1)',
        borderWidth: 2,
        fill: false,
        tension: 0.1,
      }
    ],
  } : null;

  const sessionResponseTimeData = selectedSessionDetails ? {
    labels: selectedSessionDetails.steps.map((_, index) => `Step ${index + 1}`),
    datasets: [
      {
        label: 'Average Response Time (ms)',
        data: selectedSessionDetails.steps.map(step => step.responseMetrics.avgLatency),
        borderColor: 'rgba(75, 192, 192, 1)',
        backgroundColor: 'rgba(75, 192, 192, 0.1)',
        borderWidth: 2,
        fill: false,
        tension: 0.1,
        yAxisID: 'y',
      },
      {
        label: 'P95 Response Time (ms)',
        data: selectedSessionDetails.steps.map(step => step.responseMetrics.p95Latency),
        borderColor: 'rgba(255, 205, 86, 1)',
        backgroundColor: 'rgba(255, 205, 86, 0.1)',
        borderWidth: 2,
        fill: false,
        tension: 0.1,
        yAxisID: 'y',
      },
      {
        label: 'P99 Response Time (ms)',
        data: selectedSessionDetails.steps.map(step => step.responseMetrics.p99Latency),
        borderColor: 'rgba(153, 102, 255, 1)',
        backgroundColor: 'rgba(153, 102, 255, 0.1)',
        borderWidth: 2,
        fill: false,
        tension: 0.1,
        yAxisID: 'y',
      }
    ],
  } : null;

  const sessionSuccessRateData = selectedSessionDetails ? {
    labels: selectedSessionDetails.steps.map((_, index) => `Step ${index + 1}`),
    datasets: [
      {
        label: 'Success Rate (%)',
        data: selectedSessionDetails.steps.map(step => 
          (step.successfulRequests / step.totalRequests) * 100
        ),
        borderColor: 'rgba(34, 197, 94, 1)',
        backgroundColor: 'rgba(34, 197, 94, 0.1)',
        borderWidth: 2,
        fill: true,
        tension: 0.1,
      }
    ],
  } : null;

  const chartOptions = {
    responsive: true,
    plugins: {
      legend: {
        position: 'top' as const,
      },
    },
    scales: {
      y: {
        beginAtZero: true,
      },
    },
  };

  const responseTimeChartOptions = {
    responsive: true,
    plugins: {
      legend: {
        position: 'top' as const,
      },
    },
    scales: {
      y: {
        type: 'linear' as const,
        display: true,
        position: 'left' as const,
        beginAtZero: true,
        title: {
          display: true,
          text: 'Response Time (ms)'
        }
      },
    },
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const formatDuration = (duration: number) => {
    const minutes = Math.floor(duration / 60000);
    const seconds = Math.floor((duration % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  };

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
        <div>Loading dashboard data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh', color: 'red' }}>
        <div>
          <h2>Error loading dashboard</h2>
          <p>{error}</p>
          <button onClick={() => window.location.reload()}>Retry</button>
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: '20px', fontFamily: 'Arial, sans-serif', backgroundColor: '#f5f5f5', minHeight: '100vh' }}>
      <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', marginBottom: '20px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
        <h1 style={{ margin: 0, color: '#333' }}>Stress Test Dashboard</h1>
        <p style={{ margin: '10px 0 0 0', color: '#666' }}>Real-time monitoring and analysis</p>
      </div>

      {/* Overview Stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '20px', marginBottom: '20px' }}>
        <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3 style={{ margin: 0, color: '#333' }}>Total Sessions</h3>
          <p style={{ fontSize: '2em', margin: '10px 0', color: '#2196F3' }}>{overviewStats.totalSessions}</p>
        </div>
        <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3 style={{ margin: 0, color: '#333' }}>Total Requests</h3>
          <p style={{ fontSize: '2em', margin: '10px 0', color: '#4CAF50' }}>{overviewStats.totalRequests.toLocaleString()}</p>
        </div>
        <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3 style={{ margin: 0, color: '#333' }}>Avg QPS</h3>
          <p style={{ fontSize: '2em', margin: '10px 0', color: '#FF9800' }}>{Math.round(overviewStats.avgQPS)}</p>
        </div>
        <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3 style={{ margin: 0, color: '#333' }}>Avg Success Rate</h3>
          <p style={{ fontSize: '2em', margin: '10px 0', color: overviewStats.avgSuccessRate >= 95 ? '#4CAF50' : '#FF5722' }}>
            {Math.round(overviewStats.avgSuccessRate)}%
          </p>
        </div>
      </div>

      {/* Charts */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))', gap: '20px', marginBottom: '20px' }}>
        <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3>QPS Performance</h3>
          <Bar data={qpsChartData} />
        </div>
        <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3>Success Rate Trends</h3>
          <Line data={successRateChartData} />
        </div>
        <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3>Session Status Distribution</h3>
          <Doughnut data={statusChartData} />
        </div>
      </div>

      {/* Session Details Charts */}
      {selectedSessionDetails && (
        <div style={{ marginBottom: '20px' }}>
          <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)', marginBottom: '20px' }}>
            <h2 style={{ margin: '0 0 10px 0', color: '#333' }}>
              Session Details: {selectedSession?.sessionId.substring(selectedSession.sessionId.length - 8)}
              {loadingDetails && <span style={{ marginLeft: '10px', fontSize: '0.8em', color: '#666' }}>Loading...</span>}
            </h2>
            <div style={{ fontSize: '0.9em', color: '#666', marginBottom: '20px' }}>
              <strong>Target URL:</strong> {selectedSessionDetails.config.targetUrl} | 
              <strong> Duration:</strong> {formatDuration(selectedSessionDetails.endTime - selectedSessionDetails.startTime)} | 
              <strong> Steps:</strong> {selectedSessionDetails.steps.length}
            </div>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(500px, 1fr))', gap: '20px', marginBottom: '20px' }}>
            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
              <h3>QPS Performance Over Time</h3>
              {sessionQPSPerformanceData && <Line data={sessionQPSPerformanceData} options={chartOptions} />}
            </div>
            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
              <h3>Response Time Distribution</h3>
              {sessionResponseTimeData && <Line data={sessionResponseTimeData} options={responseTimeChartOptions} />}
            </div>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(500px, 1fr))', gap: '20px', marginBottom: '20px' }}>
            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
              <h3>Success Rate Over Time</h3>
              {sessionSuccessRateData && <Line data={sessionSuccessRateData} options={chartOptions} />}
            </div>
            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
              <h3>Step Details</h3>
              <div style={{ overflowX: 'auto', maxHeight: '400px', overflowY: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.9em' }}>
                  <thead style={{ position: 'sticky', top: 0, backgroundColor: '#f5f5f5' }}>
                    <tr>
                      <th style={{ padding: '8px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Step</th>
                      <th style={{ padding: '8px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Target QPS</th>
                      <th style={{ padding: '8px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Actual QPS</th>
                      <th style={{ padding: '8px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Requests</th>
                      <th style={{ padding: '8px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Success%</th>
                      <th style={{ padding: '8px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Avg RT</th>
                      <th style={{ padding: '8px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>P95 RT</th>
                    </tr>
                  </thead>
                  <tbody>
                    {selectedSessionDetails.steps.map((step, index) => (
                      <tr key={index} style={{ borderBottom: '1px solid #eee' }}>
                        <td style={{ padding: '8px' }}>{index + 1}</td>
                        <td style={{ padding: '8px' }}>{step.qps}</td>
                        <td style={{ padding: '8px', color: Math.abs(step.actualQPS - step.qps) / step.qps > 0.1 ? '#FF5722' : '#333' }}>
                          {step.actualQPS.toFixed(1)}
                        </td>
                        <td style={{ padding: '8px' }}>{step.totalRequests}</td>
                        <td style={{ padding: '8px', color: (step.successfulRequests / step.totalRequests) >= 0.95 ? '#4CAF50' : '#FF5722' }}>
                          {((step.successfulRequests / step.totalRequests) * 100).toFixed(1)}%
                        </td>
                        <td style={{ padding: '8px' }}>{step.responseMetrics.avgLatency.toFixed(1)}ms</td>
                        <td style={{ padding: '8px' }}>{step.responseMetrics.p95Latency.toFixed(1)}ms</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Session List */}
      <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
        <h3>Recent Sessions</h3>
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ backgroundColor: '#f5f5f5' }}>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Select</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Session ID</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Start Time</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Duration</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Status</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Max QPS</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Recommended QPS</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Requests</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Success Rate</th>
                <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #ddd' }}>Avg Response Time</th>
              </tr>
            </thead>
            <tbody>
              {sessions.map((session) => (
                <tr 
                  key={session.sessionId} 
                  style={{ 
                    borderBottom: '1px solid #eee',
                    backgroundColor: selectedSession?.sessionId === session.sessionId ? '#f0f8ff' : 'transparent',
                    cursor: 'pointer'
                  }}
                  onClick={() => handleSessionSelect(session)}
                >
                  <td style={{ padding: '12px' }}>
                    <input 
                      type="radio" 
                      checked={selectedSession?.sessionId === session.sessionId}
                      onChange={() => handleSessionSelect(session)}
                    />
                  </td>
                  <td style={{ padding: '12px', fontFamily: 'monospace', fontSize: '0.9em' }}>
                    {session.sessionId.substring(session.sessionId.length - 8)}
                  </td>
                  <td style={{ padding: '12px' }}>{formatDate(session.startTime)}</td>
                  <td style={{ padding: '12px' }}>{formatDuration(session.duration)}</td>
                  <td style={{ padding: '12px' }}>
                    <span style={{
                      padding: '4px 8px',
                      borderRadius: '4px',
                      fontSize: '0.8em',
                      color: 'white',
                      backgroundColor: session.status === 'completed' ? '#4CAF50' : '#FF9800'
                    }}>
                      {session.status}
                    </span>
                  </td>
                  <td style={{ padding: '12px', fontWeight: 'bold' }}>{session.maxSustainableQPS}</td>
                  <td style={{ padding: '12px', color: '#2196F3' }}>{session.recommendedMaxQPS}</td>
                  <td style={{ padding: '12px' }}>{session.overallStats.totalRequests.toLocaleString()}</td>
                  <td style={{ padding: '12px', color: session.overallStats.overallSuccessRate >= 95 ? '#4CAF50' : '#FF5722' }}>
                    {session.overallStats.overallSuccessRate}%
                  </td>
                  <td style={{ padding: '12px' }}>
                    {session.overallStats.avgResponseTime > 0 ? `${session.overallStats.avgResponseTime.toFixed(1)}ms` : 'N/A'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}