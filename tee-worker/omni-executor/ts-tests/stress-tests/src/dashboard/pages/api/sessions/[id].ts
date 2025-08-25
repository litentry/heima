import { NextApiRequest, NextApiResponse } from 'next';
import fs from 'fs';
import path from 'path';
import { QPSStepResult, DetailedSessionResult } from '../../../types';


const RESULT_PATH = process.env.RESULT_PATH || './stress-test-results';

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