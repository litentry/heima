import { NextApiRequest, NextApiResponse } from 'next';
import fs from 'fs';
import path from 'path';

interface SessionSummary {
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

const RESULT_PATH = '/Users/verin/Repo/Heima/heima/tee-worker/omni-executor/ts-tests/stress-tests/stress-test-results';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    // Check if results directory exists
    if (!fs.existsSync(RESULT_PATH)) {
      return res.json([]);
    }

    const sessions: SessionSummary[] = [];
    const sessionDirs = fs.readdirSync(RESULT_PATH)
      .filter(dir => {
        const fullPath = path.join(RESULT_PATH, dir);
        return fs.statSync(fullPath).isDirectory() && dir.startsWith('stress_test_');
      });

    for (const sessionDir of sessionDirs) {
      const sessionPath = path.join(RESULT_PATH, sessionDir);
      const summaryPath = path.join(sessionPath, 'session_summary.json');

      if (fs.existsSync(summaryPath)) {
        try {
          const summaryContent = fs.readFileSync(summaryPath, 'utf8');
          const summary = JSON.parse(summaryContent) as SessionSummary;
          sessions.push(summary);
        } catch (err) {
          console.error(`Error reading session summary for ${sessionDir}:`, err);
        }
      } else {
        // Try to create session_summary from session-result.json if it exists
        const resultPath = path.join(sessionPath, 'session-result.json');
        if (fs.existsSync(resultPath)) {
          try {
            const resultContent = fs.readFileSync(resultPath, 'utf8');
            const result = JSON.parse(resultContent);
            
            const summary: SessionSummary = {
              sessionId: result.sessionId,
              startTime: result.startTime,
              endTime: result.endTime,
              duration: result.endTime - result.startTime,
              status: 'completed',
              config: result.config,
              maxSustainableQPS: result.maxSustainableQPS || 0,
              recommendedMaxQPS: result.recommendedMaxQPS || 0,
              overallStats: result.overallStats || {
                totalRequests: 0,
                totalSuccessful: 0,
                totalFailed: 0,
                overallSuccessRate: 0,
                avgResponseTime: 0,
                maxResponseTime: 0,
                minResponseTime: 0
              }
            };

            // Write the summary file for future use
            fs.writeFileSync(summaryPath, JSON.stringify(summary, null, 2));
            sessions.push(summary);
          } catch (err) {
            console.error(`Error processing session result for ${sessionDir}:`, err);
          }
        }
      }
    }

    // Sort sessions by start time (newest first)
    sessions.sort((a, b) => b.startTime - a.startTime);

    res.json(sessions);
  } catch (error) {
    console.error('Error fetching sessions:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
}