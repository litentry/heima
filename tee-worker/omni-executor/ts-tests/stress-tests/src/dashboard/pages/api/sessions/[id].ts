import { NextApiRequest, NextApiResponse } from 'next';
import fs from 'fs';
import path from 'path';

interface QPSStepResult {
  qps: number;
  actualQPS: number;
  duration: number;
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  endpointStats: Record<string, {
    total: number;
    successful: number;
    failed: number;
    avgResponseTime: number;
    minResponseTime: number;
    maxResponseTime: number;
    errorRate: number;
    throughput: number;
  }>;
  responseMetrics: {
    avgLatency: number;
    minLatency: number;
    maxLatency: number;
    p50Latency: number;
    p95Latency: number;
    p99Latency: number;
  };
  errorDetails: Record<string, number>;
  shouldStop: boolean;
  stopReason?: string;
}

interface DetailedSessionResult {
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

const RESULT_PATH = '/Users/verin/Repo/Heima/heima/tee-worker/omni-executor/ts-tests/stress-tests/stress-test-results';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { id } = req.query;
  if (!id || typeof id !== 'string') {
    return res.status(400).json({ error: 'Session ID required' });
  }

  try {
    const sessionPath = path.join(RESULT_PATH, id);
    if (!fs.existsSync(sessionPath)) {
      return res.status(404).json({ error: 'Session not found' });
    }

    const resultPath = path.join(sessionPath, 'session-result.json');
    if (!fs.existsSync(resultPath)) {
      return res.status(404).json({ error: 'Session result not found' });
    }

    const resultContent = fs.readFileSync(resultPath, 'utf8');
    const sessionResult: DetailedSessionResult = JSON.parse(resultContent);

    res.json(sessionResult);
  } catch (error) {
    console.error('Error fetching session details:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
}