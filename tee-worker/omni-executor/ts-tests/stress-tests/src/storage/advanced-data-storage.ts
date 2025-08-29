import fs from 'fs';
import path from 'path';
import { performance } from 'perf_hooks';
import { RequestResult, QPSStepResult, TestSessionResult } from '../types';


interface AnalysisResult {
  sessionId: string;
  generatedAt: number;
  performanceAnalysis: {
    throughputAnalysis: any;
    latencyAnalysis: any;
    errorAnalysis: any;
    endpointAnalysis: any;
    recommendationsAnalysis: any;
  };
  summary: {
    testDuration: number;
    totalRequests: number;
    peakQPS: number;
    averageResponseTime: number;
    overallSuccessRate: number;
    topErrors: Array<{ type: string; count: number; percentage: number }>;
  };
}

// Storage configuration
interface StorageConfig {
  outputDir: string;
  sessionId: string;
  enableCompression: boolean;
  enableBackup: boolean;
  maxSessionHistory: number;
  enableEncryption: boolean;
}

// Storage statistics
interface StorageStats {
  totalSessions: number;
  totalRequests: number;
  totalSteps: number;
  totalSize: number;
  lastUpdated: number;
  averageRequestsPerSession: number;
  averageStepsPerSession: number;
}

export class DataStorage {
  private config: StorageConfig;
  private sessionDir: string;
  private writeQueue: Array<{ operation: () => Promise<void>; priority: number }> = [];
  private isProcessingQueue = false;
  private stats: StorageStats;

  constructor(outputDir: string, sessionId: string, config: Partial<StorageConfig> = {}) {
    this.config = {
      outputDir,
      sessionId,
      enableCompression: false,
      enableBackup: true,
      maxSessionHistory: 50,
      enableEncryption: false,
      ...config
    };

    this.sessionDir = path.join(this.config.outputDir, this.config.sessionId);
    
    // Initialize storage directory structure
    this.initializeStorage();
    
    // Load or initialize statistics
    this.stats = this.loadStats();
  }

  private initializeStorage(): void {
    // Create main output directory
    if (!fs.existsSync(this.config.outputDir)) {
      fs.mkdirSync(this.config.outputDir, { recursive: true });
    }

    // Create session directory
    if (!fs.existsSync(this.sessionDir)) {
      fs.mkdirSync(this.sessionDir, { recursive: true });
    }

    // Create subdirectories
    const subdirs = ['requests', 'steps', 'analysis', 'raw', 'exports', 'backups'];
    for (const subdir of subdirs) {
      const dirPath = path.join(this.sessionDir, subdir);
      if (!fs.existsSync(dirPath)) {
        fs.mkdirSync(dirPath, { recursive: true });
      }
    }

    // Create session metadata file
    const metadata = {
      sessionId: this.config.sessionId,
      createdAt: Date.now(),
      version: '1.0.0',
      config: this.config
    };

    fs.writeFileSync(
      path.join(this.sessionDir, 'metadata.json'),
      JSON.stringify(metadata, null, 2)
    );
  }

  private loadStats(): StorageStats {
    const statsFile = path.join(this.config.outputDir, 'storage-stats.json');
    
    if (fs.existsSync(statsFile)) {
      try {
        return JSON.parse(fs.readFileSync(statsFile, 'utf-8'));
      } catch (error) {
        console.warn('Failed to load storage stats, initializing new stats');
      }
    }

    return {
      totalSessions: 0,
      totalRequests: 0,
      totalSteps: 0,
      totalSize: 0,
      lastUpdated: Date.now(),
      averageRequestsPerSession: 0,
      averageStepsPerSession: 0
    };
  }

  private updateStats(requests?: number, steps?: number): void {
    if (requests) {
      this.stats.totalRequests += requests;
    }
    if (steps) {
      this.stats.totalSteps += steps;
    }
    
    this.stats.lastUpdated = Date.now();
    this.stats.averageRequestsPerSession = this.stats.totalRequests / Math.max(this.stats.totalSessions, 1);
    this.stats.averageStepsPerSession = this.stats.totalSteps / Math.max(this.stats.totalSessions, 1);

    // Calculate total storage size
    this.stats.totalSize = this.calculateStorageSize();

    // Save stats
    this.saveStats();
  }

  private saveStats(): void {
    const statsFile = path.join(this.config.outputDir, 'storage-stats.json');
    try {
      fs.writeFileSync(statsFile, JSON.stringify(this.stats, null, 2));
    } catch (error) {
      console.error('Failed to save storage stats:', error);
    }
  }

  private calculateStorageSize(): number {
    let totalSize = 0;
    
    try {
      const calculateDirSize = (dirPath: string): number => {
        let size = 0;
        if (fs.existsSync(dirPath)) {
          const files = fs.readdirSync(dirPath);
          for (const file of files) {
            const filePath = path.join(dirPath, file);
            const stat = fs.statSync(filePath);
            if (stat.isDirectory()) {
              size += calculateDirSize(filePath);
            } else {
              size += stat.size;
            }
          }
        }
        return size;
      };

      totalSize = calculateDirSize(this.config.outputDir);
    } catch (error) {
      console.warn('Failed to calculate storage size:', error);
    }

    return totalSize;
  }

  private async addToQueue(operation: () => Promise<void>, priority: number = 1): Promise<void> {
    return new Promise((resolve, reject) => {
      this.writeQueue.push({
        operation: async () => {
          try {
            await operation();
            resolve();
          } catch (error) {
            reject(error);
          }
        },
        priority
      });

      this.processQueue();
    });
  }

  private async processQueue(): Promise<void> {
    if (this.isProcessingQueue || this.writeQueue.length === 0) {
      return;
    }

    this.isProcessingQueue = true;

    // Sort queue by priority (higher priority first)
    this.writeQueue.sort((a, b) => b.priority - a.priority);

    while (this.writeQueue.length > 0) {
      const { operation } = this.writeQueue.shift()!;
      try {
        await operation();
      } catch (error) {
        console.error('Queue operation failed:', error);
      }
    }

    this.isProcessingQueue = false;
  }

  // Store individual request result
  public async storeRequestResult(request: RequestResult): Promise<void> {
    return this.addToQueue(async () => {
      const timestamp = new Date(request.timestamp);
      const dateStr = timestamp.toISOString().split('T')[0]; // YYYY-MM-DD
      const hourStr = timestamp.getUTCHours().toString().padStart(2, '0');
      
      // Organize by date and hour for efficient querying
      const filename = `requests-${dateStr}-${hourStr}.jsonl`;
      const filepath = path.join(this.sessionDir, 'requests', filename);
      
      // Append to JSONL file (one JSON object per line)
      const line = JSON.stringify(request) + '\n';
      fs.appendFileSync(filepath, line);
      
      this.updateStats(1, 0);
    }, 2); // Medium priority
  }

  // Store QPS step result
  public async storeStepResult(step: QPSStepResult): Promise<void> {
    return this.addToQueue(async () => {
      const filename = `step-${step.qps}-${Date.now()}.json`;
      const filepath = path.join(this.sessionDir, 'steps', filename);
      
      fs.writeFileSync(filepath, JSON.stringify(step, null, 2));
      
      this.updateStats(0, 1);
    }, 3); // High priority
  }

  // Store session result
  public async storeSessionResult(session: TestSessionResult): Promise<void> {
    return this.addToQueue(async () => {
      const filepath = path.join(this.sessionDir, 'session-result.json');
      fs.writeFileSync(filepath, JSON.stringify(session, null, 2));
      
      // Also store in main directory for easy access
      const mainFilepath = path.join(this.config.outputDir, `session-${session.sessionId}.json`);
      fs.writeFileSync(mainFilepath, JSON.stringify(session, null, 2));
      
      // Update total sessions count
      this.stats.totalSessions++;
      this.updateStats(0, 0);
    }, 4); // Highest priority
  }

  // Store analysis result
  public async storeAnalysis(analysis: AnalysisResult): Promise<void> {
    return this.addToQueue(async () => {
      const filepath = path.join(this.sessionDir, 'analysis', 'full-analysis.json');
      fs.writeFileSync(filepath, JSON.stringify(analysis, null, 2));
      
      // Store summary separately for quick access
      const summaryPath = path.join(this.sessionDir, 'analysis', 'summary.json');
      fs.writeFileSync(summaryPath, JSON.stringify(analysis.summary, null, 2));
    }, 3); // High priority
  }

  // Store raw data for detailed analysis
  public async storeRawData(data: any, filename: string): Promise<void> {
    return this.addToQueue(async () => {
      const filepath = path.join(this.sessionDir, 'raw', filename);
      fs.writeFileSync(filepath, JSON.stringify(data, null, 2));
    }, 1); // Low priority
  }

  // Export data in various formats
  public async exportToCSV(filename: string = 'requests.csv'): Promise<string> {
    const csvPath = path.join(this.sessionDir, 'exports', filename);
    
    // Read all request files
    const requests = await this.loadAllRequests();
    
    if (requests.length === 0) {
      throw new Error('No request data available for export');
    }

    // Generate CSV headers
    const headers = [
      'timestamp', 'endpoint', 'success', 'responseTime', 'requestSize',
      'responseSize', 'qps', 'error', 'statusCode'
    ];

    // Generate CSV content
    const csvContent = [
      headers.join(','),
      ...requests.map(req => [
        new Date(req.timestamp).toISOString(),
        req.endpoint,
        req.success,
        req.responseTime,
        req.requestSize,
        req.responseSize,
        req.qps,
        req.error ? `"${req.error.replace(/"/g, '""')}"` : '',
        req.statusCode || ''
      ].join(','))
    ].join('\n');

    fs.writeFileSync(csvPath, csvContent);
    return csvPath;
  }

  // Export steps data to CSV
  public async exportStepsToCSV(filename: string = 'steps.csv'): Promise<string> {
    const csvPath = path.join(this.sessionDir, 'exports', filename);
    
    const steps = await this.loadAllSteps();
    
    if (steps.length === 0) {
      throw new Error('No step data available for export');
    }

    const headers = [
      'qps', 'actualQPS', 'duration', 'totalRequests', 'successfulRequests',
      'failedRequests', 'errorRate', 'avgResponseTime', 'shouldStop', 'stopReason'
    ];

    const csvContent = [
      headers.join(','),
      ...steps.map(step => {
        const errorRate = step.totalRequests > 0 ? (step.failedRequests / step.totalRequests * 100) : 0;
        const endpointStats = step.endpointStats || {};
        const statsValues = Object.values(endpointStats);
        const avgResponseTime = statsValues.length > 0 
          ? statsValues.reduce((sum, stat) => sum + stat.avgResponseTime, 0) / statsValues.length
          : 0;
        
        return [
          step.qps,
          step.actualQPS,
          step.duration,
          step.totalRequests,
          step.successfulRequests,
          step.failedRequests,
          errorRate.toFixed(2),
          avgResponseTime.toFixed(2),
          step.shouldStop,
          step.stopReason ? `"${step.stopReason.replace(/"/g, '""')}"` : ''
        ].join(',');
      })
    ].join('\n');

    fs.writeFileSync(csvPath, csvContent);
    return csvPath;
  }

  // Load all requests from storage
  public async loadAllRequests(): Promise<RequestResult[]> {
    const requests: RequestResult[] = [];
    const requestsDir = path.join(this.sessionDir, 'requests');
    
    if (!fs.existsSync(requestsDir)) {
      return requests;
    }

    const files = fs.readdirSync(requestsDir).filter(file => file.endsWith('.jsonl'));
    
    for (const file of files) {
      const filepath = path.join(requestsDir, file);
      const content = fs.readFileSync(filepath, 'utf-8');
      const lines = content.trim().split('\n').filter(line => line.trim());
      
      for (const line of lines) {
        try {
          requests.push(JSON.parse(line));
        } catch (error) {
          console.warn(`Failed to parse line in ${file}:`, line);
        }
      }
    }

    // Sort by timestamp
    return requests.sort((a, b) => a.timestamp - b.timestamp);
  }

  // Load all QPS steps from storage
  public async loadAllSteps(): Promise<QPSStepResult[]> {
    const steps: QPSStepResult[] = [];
    const stepsDir = path.join(this.sessionDir, 'steps');
    
    if (!fs.existsSync(stepsDir)) {
      return steps;
    }

    const files = fs.readdirSync(stepsDir).filter(file => file.endsWith('.json'));
    
    for (const file of files) {
      const filepath = path.join(stepsDir, file);
      try {
        const content = fs.readFileSync(filepath, 'utf-8');
        steps.push(JSON.parse(content));
      } catch (error) {
        console.warn(`Failed to load step file ${file}:`, error);
      }
    }

    // Sort by QPS
    return steps.sort((a, b) => a.qps - b.qps);
  }

  // Load session result
  public async loadSessionResult(): Promise<TestSessionResult | null> {
    const filepath = path.join(this.sessionDir, 'session-result.json');
    
    if (!fs.existsSync(filepath)) {
      return null;
    }

    try {
      const content = fs.readFileSync(filepath, 'utf-8');
      return JSON.parse(content);
    } catch (error) {
      console.error('Failed to load session result:', error);
      return null;
    }
  }

  // Load analysis result
  public async loadAnalysis(): Promise<AnalysisResult | null> {
    const filepath = path.join(this.sessionDir, 'analysis', 'full-analysis.json');
    
    if (!fs.existsSync(filepath)) {
      return null;
    }

    try {
      const content = fs.readFileSync(filepath, 'utf-8');
      return JSON.parse(content);
    } catch (error) {
      console.error('Failed to load analysis:', error);
      return null;
    }
  }

  // Query requests by time range
  public async queryRequestsByTimeRange(startTime: number, endTime: number): Promise<RequestResult[]> {
    const allRequests = await this.loadAllRequests();
    return allRequests.filter(req => req.timestamp >= startTime && req.timestamp <= endTime);
  }

  // Query requests by endpoint
  public async queryRequestsByEndpoint(endpoint: string): Promise<RequestResult[]> {
    const allRequests = await this.loadAllRequests();
    return allRequests.filter(req => req.endpoint === endpoint);
  }

  // Query failed requests
  public async queryFailedRequests(): Promise<RequestResult[]> {
    const allRequests = await this.loadAllRequests();
    return allRequests.filter(req => !req.success);
  }

  // Get storage statistics
  public getStorageStats(): StorageStats {
    return { ...this.stats };
  }

  // Get session summary
  public async getSessionSummary(): Promise<any> {
    const session = await this.loadSessionResult();
    const requests = await this.loadAllRequests();
    const steps = await this.loadAllSteps();
    
    return {
      sessionId: this.config.sessionId,
      sessionResult: session,
      totalRequests: requests.length,
      totalSteps: steps.length,
      storageSize: this.calculateStorageSize(),
      createdAt: session?.startTime || Date.now(),
      duration: session ? session.endTime - session.startTime : 0
    };
  }

  // Cleanup old sessions
  public async cleanupOldSessions(): Promise<void> {
    const mainDir = this.config.outputDir;
    
    if (!fs.existsSync(mainDir)) {
      return;
    }

    // Get all session directories
    const entries = fs.readdirSync(mainDir, { withFileTypes: true });
    const sessionDirs = entries
      .filter(entry => entry.isDirectory() && entry.name.startsWith('stress_test_'))
      .map(entry => ({
        name: entry.name,
        path: path.join(mainDir, entry.name),
        timestamp: parseInt(entry.name.split('_')[2]) || 0
      }))
      .sort((a, b) => b.timestamp - a.timestamp); // Sort by newest first

    // Remove old sessions if we exceed the limit
    if (sessionDirs.length > this.config.maxSessionHistory) {
      const sessionsToRemove = sessionDirs.slice(this.config.maxSessionHistory);
      
      for (const session of sessionsToRemove) {
        try {
          // Create backup if enabled
          if (this.config.enableBackup) {
            await this.createBackup(session.path);
          }
          
          // Remove directory
          fs.rmSync(session.path, { recursive: true, force: true });
          console.log(`Cleaned up old session: ${session.name}`);
        } catch (error) {
          console.error(`Failed to cleanup session ${session.name}:`, error);
        }
      }
    }
  }

  // Create backup of session data
  public async createBackup(sessionPath?: string): Promise<string> {
    const targetPath = sessionPath || this.sessionDir;
    const backupDir = path.join(this.config.outputDir, 'backups');
    
    if (!fs.existsSync(backupDir)) {
      fs.mkdirSync(backupDir, { recursive: true });
    }

    const backupName = `backup-${path.basename(targetPath)}-${Date.now()}.tar.gz`;
    const backupPath = path.join(backupDir, backupName);

    // Simple copy-based backup (could be enhanced with compression)
    const backupSessionDir = path.join(backupDir, path.basename(targetPath));
    this.copyDirectory(targetPath, backupSessionDir);

    return backupPath;
  }

  private copyDirectory(src: string, dest: string): void {
    if (!fs.existsSync(dest)) {
      fs.mkdirSync(dest, { recursive: true });
    }

    const entries = fs.readdirSync(src, { withFileTypes: true });
    
    for (const entry of entries) {
      const srcPath = path.join(src, entry.name);
      const destPath = path.join(dest, entry.name);
      
      if (entry.isDirectory()) {
        this.copyDirectory(srcPath, destPath);
      } else {
        fs.copyFileSync(srcPath, destPath);
      }
    }
  }

  // Get all available sessions
  public getAvailableSessions(): Array<{ sessionId: string; timestamp: number; path: string }> {
    const mainDir = this.config.outputDir;
    
    if (!fs.existsSync(mainDir)) {
      return [];
    }

    const entries = fs.readdirSync(mainDir, { withFileTypes: true });
    return entries
      .filter(entry => entry.isDirectory() && entry.name.startsWith('stress_test_'))
      .map(entry => ({
        sessionId: entry.name,
        timestamp: parseInt(entry.name.split('_')[2]) || 0,
        path: path.join(mainDir, entry.name)
      }))
      .sort((a, b) => b.timestamp - a.timestamp);
  }

  // Ensure all pending writes are completed
  public async flush(): Promise<void> {
    while (this.writeQueue.length > 0 || this.isProcessingQueue) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }
  }

  // Cleanup and close storage
  public async close(): Promise<void> {
    await this.flush();
    this.saveStats();
    
    // Cleanup old sessions if needed
    await this.cleanupOldSessions();
  }
}