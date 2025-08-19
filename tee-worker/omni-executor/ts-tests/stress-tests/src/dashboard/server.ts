import express from 'express';
import { WebSocketServer } from 'ws';
import { createServer } from 'http';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { StressTestEngine } from '../core/stress-engine';
import { DataStorage } from '../storage/data-storage';
import { TestSession, SystemMetrics, RequestResult } from '../types/index';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export class DashboardServer {
  private app: express.Application;
  private server: ReturnType<typeof createServer>;
  private wss: WebSocketServer;
  private port: number;
  private stressEngine?: StressTestEngine;
  private dataStorage: DataStorage;
  private isRunning = false;

  constructor(port = 3000) {
    this.port = port;
    this.app = express();
    this.server = createServer(this.app);
    this.wss = new WebSocketServer({ server: this.server });
    this.dataStorage = new DataStorage();
    
    this.setupMiddleware();
    this.setupRoutes();
    this.setupWebSocket();
  }

  private setupMiddleware(): void {
    this.app.use(express.json());
    this.app.use(express.static(join(__dirname, 'public')));
  }

  private setupRoutes(): void {
    // Serve the main dashboard page
    this.app.get('/', (req, res) => {
      res.sendFile(join(__dirname, 'public', 'index.html'));
    });

    // API endpoints
    this.app.post('/api/start-test', async (req, res) => {
      try {
        if (this.stressEngine && this.isRunning) {
          return res.status(400).json({ error: 'Test already running' });
        }

        const config = req.body;
        this.stressEngine = new StressTestEngine();
        
        // Setup real-time event listeners
        this.setupEngineListeners();
        
        const sessionId = await this.stressEngine.startTest(config);
        this.isRunning = true;
        
        res.json({ sessionId, status: 'started' });
      } catch (error) {
        res.status(500).json({ 
          error: error instanceof Error ? error.message : 'Unknown error' 
        });
      }
    });

    this.app.post('/api/stop-test', async (req, res) => {
      try {
        if (!this.stressEngine || !this.isRunning) {
          return res.status(400).json({ error: 'No test running' });
        }

        await this.stressEngine.stopTest();
        this.isRunning = false;
        
        res.json({ status: 'stopped' });
      } catch (error) {
        res.status(500).json({ 
          error: error instanceof Error ? error.message : 'Unknown error' 
        });
      }
    });

    this.app.get('/api/test-status', (req, res) => {
      if (!this.stressEngine) {
        return res.json({ status: 'idle', session: null });
      }

      const session = this.stressEngine.getSession();
      res.json({ 
        status: this.isRunning ? 'running' : 'stopped', 
        session 
      });
    });

    this.app.get('/api/sessions', async (req, res) => {
      try {
        const sessionIds = await this.dataStorage.listTestSessions();
        const sessions: TestSession[] = [];
        
        for (const id of sessionIds.slice(0, 20)) { // Limit to latest 20
          const session = await this.dataStorage.loadTestSession(id);
          if (session) {
            sessions.push(session);
          }
        }
        
        res.json(sessions);
      } catch (error) {
        res.status(500).json({ 
          error: error instanceof Error ? error.message : 'Unknown error' 
        });
      }
    });

    this.app.get('/api/sessions/:id', async (req, res) => {
      try {
        const session = await this.dataStorage.loadTestSession(req.params.id);
        if (!session) {
          return res.status(404).json({ error: 'Session not found' });
        }
        res.json(session);
      } catch (error) {
        res.status(500).json({ 
          error: error instanceof Error ? error.message : 'Unknown error' 
        });
      }
    });

    this.app.delete('/api/sessions/:id', async (req, res) => {
      try {
        const success = await this.dataStorage.deleteTestSession(req.params.id);
        if (!success) {
          return res.status(404).json({ error: 'Session not found' });
        }
        res.json({ status: 'deleted' });
      } catch (error) {
        res.status(500).json({ 
          error: error instanceof Error ? error.message : 'Unknown error' 
        });
      }
    });

    this.app.post('/api/export', async (req, res) => {
      try {
        const { sessionIds } = req.body;
        const exportPath = await this.dataStorage.exportSessionsToJson(sessionIds);
        res.json({ exportPath });
      } catch (error) {
        res.status(500).json({ 
          error: error instanceof Error ? error.message : 'Unknown error' 
        });
      }
    });
  }

  private setupWebSocket(): void {
    this.wss.on('connection', (ws) => {
      console.log('Dashboard client connected');
      
      // Send initial state
      if (this.stressEngine) {
        const session = this.stressEngine.getSession();
        ws.send(JSON.stringify({
          type: 'status',
          data: { 
            status: this.isRunning ? 'running' : 'stopped', 
            session 
          }
        }));
      }

      ws.on('close', () => {
        console.log('Dashboard client disconnected');
      });

      ws.on('error', (error) => {
        console.error('WebSocket error:', error);
      });
    });
  }

  private setupEngineListeners(): void {
    if (!this.stressEngine) return;

    this.stressEngine.on('testStarted', (data) => {
      this.broadcast({
        type: 'testStarted',
        data
      });
    });

    this.stressEngine.on('requestCompleted', (result: RequestResult) => {
      this.broadcast({
        type: 'requestCompleted',
        data: result
      });
    });

    this.stressEngine.on('requestFailed', (result: RequestResult) => {
      this.broadcast({
        type: 'requestFailed',
        data: result
      });
    });

    this.stressEngine.on('metricsCollected', (metrics: SystemMetrics) => {
      this.broadcast({
        type: 'systemMetrics',
        data: metrics
      });
    });

    this.stressEngine.on('testCompleted', (data) => {
      this.broadcast({
        type: 'testCompleted',
        data
      });
      this.isRunning = false;
    });

    this.stressEngine.on('testFailed', (data) => {
      this.broadcast({
        type: 'testFailed',
        data
      });
      this.isRunning = false;
    });
  }

  private broadcast(message: any): void {
    const data = JSON.stringify(message);
    this.wss.clients.forEach((client) => {
      if (client.readyState === client.OPEN) {
        client.send(data);
      }
    });
  }

  start(): Promise<void> {
    return new Promise((resolve) => {
      this.server.listen(this.port, () => {
        console.log(`Dashboard server running on http://localhost:${this.port}`);
        resolve();
      });
    });
  }

  stop(): Promise<void> {
    return new Promise((resolve) => {
      this.wss.close(() => {
        this.server.close(() => {
          console.log('Dashboard server stopped');
          resolve();
        });
      });
    });
  }
}