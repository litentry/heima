import fs from 'fs';
import path from 'path';
import chalk from 'chalk';

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
  CRITICAL = 4
}

interface LogEntry {
  timestamp: number;
  level: LogLevel;
  message: string;
  metadata?: Record<string, any>;
  sessionId: string;
  source?: string;
}

interface LoggerConfig {
  level: LogLevel;
  enableConsole: boolean;
  enableFile: boolean;
  enableJson: boolean;
  maxFileSize: number; // MB
  maxFiles: number;
  outputDir: string;
  sessionId: string;
}

export class Logger {
  private config: LoggerConfig;
  private logBuffer: LogEntry[] = [];
  private bufferFlushInterval: NodeJS.Timeout;
  private currentLogFile: string;
  private currentLogSize: number = 0;
  private logFileIndex: number = 0;

  constructor(outputDir: string, sessionId: string, config: Partial<LoggerConfig> = {}) {
    this.config = {
      level: LogLevel.INFO,
      enableConsole: true,
      enableFile: true,
      enableJson: true,
      maxFileSize: 50, // 50MB
      maxFiles: 10,
      outputDir,
      sessionId,
      ...config
    };

    // Create output directory if it doesn't exist
    const sessionDir = path.join(this.config.outputDir, this.config.sessionId);
    if (!fs.existsSync(sessionDir)) {
      fs.mkdirSync(sessionDir, { recursive: true });
    }

    // Initialize log files
    this.currentLogFile = path.join(sessionDir, `stress-test-${this.logFileIndex}.log`);
    
    // Flush buffer every 5 seconds
    this.bufferFlushInterval = setInterval(() => {
      this.flushBuffer();
    }, 5000);

    // Log initial session info
    this.info('Logger initialized', {
      sessionId: this.config.sessionId,
      outputDir: this.config.outputDir,
      config: {
        level: LogLevel[this.config.level],
        enableConsole: this.config.enableConsole,
        enableFile: this.config.enableFile,
        enableJson: this.config.enableJson,
        maxFileSize: this.config.maxFileSize,
        maxFiles: this.config.maxFiles
      }
    });
  }

  private shouldLog(level: LogLevel): boolean {
    return level >= this.config.level;
  }

  private formatTimestamp(timestamp: number): string {
    return new Date(timestamp).toISOString();
  }

  private formatConsoleMessage(entry: LogEntry): string {
    const timestamp = this.formatTimestamp(entry.timestamp);
    const level = LogLevel[entry.level].padEnd(8);
    
    let coloredLevel: string;
    switch (entry.level) {
      case LogLevel.DEBUG:
        coloredLevel = chalk.gray(level);
        break;
      case LogLevel.INFO:
        coloredLevel = chalk.blue(level);
        break;
      case LogLevel.WARN:
        coloredLevel = chalk.yellow(level);
        break;
      case LogLevel.ERROR:
        coloredLevel = chalk.red(level);
        break;
      case LogLevel.CRITICAL:
        coloredLevel = chalk.bgRed.white(level);
        break;
      default:
        coloredLevel = level;
    }

    let message = `${chalk.gray(timestamp)} ${coloredLevel} ${entry.message}`;
    
    if (entry.metadata && Object.keys(entry.metadata).length > 0) {
      message += '\n' + chalk.gray('  Metadata: ') + JSON.stringify(entry.metadata, null, 2)
        .split('\n')
        .map((line, index) => index === 0 ? line : '  ' + line)
        .join('\n');
    }

    return message;
  }

  private formatFileMessage(entry: LogEntry): string {
    const timestamp = this.formatTimestamp(entry.timestamp);
    const level = LogLevel[entry.level].padEnd(8);
    
    let message = `[${timestamp}] ${level} ${entry.message}`;
    
    if (entry.metadata && Object.keys(entry.metadata).length > 0) {
      message += ' | Metadata: ' + JSON.stringify(entry.metadata);
    }

    return message;
  }

  private log(level: LogLevel, message: string, metadata?: Record<string, any>, source?: string): void {
    if (!this.shouldLog(level)) {
      return;
    }

    const entry: LogEntry = {
      timestamp: Date.now(),
      level,
      message,
      metadata,
      sessionId: this.config.sessionId,
      source
    };

    // Console output
    if (this.config.enableConsole) {
      console.log(this.formatConsoleMessage(entry));
    }

    // Add to buffer for file output
    if (this.config.enableFile) {
      this.logBuffer.push(entry);
    }

    // Immediate flush for critical errors
    if (level >= LogLevel.ERROR) {
      this.flushBuffer();
    }
  }

  private flushBuffer(): void {
    if (this.logBuffer.length === 0 || !this.config.enableFile) {
      return;
    }

    const entries = [...this.logBuffer];
    this.logBuffer = [];

    try {
      // Check if we need to rotate log file
      this.checkLogRotation();

      // Write to text log file
      const textMessages = entries.map(entry => this.formatFileMessage(entry));
      const textContent = textMessages.join('\n') + '\n';
      
      fs.appendFileSync(this.currentLogFile, textContent);
      this.currentLogSize += Buffer.byteLength(textContent, 'utf8');

      // Write to JSON log file if enabled
      if (this.config.enableJson) {
        const jsonLogFile = this.currentLogFile.replace('.log', '.json');
        const jsonEntries = entries.map(entry => JSON.stringify(entry)).join('\n') + '\n';
        fs.appendFileSync(jsonLogFile, jsonEntries);
      }

    } catch (error) {
      // Fallback to console if file writing fails
      console.error('Failed to write to log file:', error);
      if (this.config.enableConsole) {
        entries.forEach(entry => console.log(this.formatConsoleMessage(entry)));
      }
    }
  }

  private checkLogRotation(): void {
    const maxSizeBytes = this.config.maxFileSize * 1024 * 1024; // Convert MB to bytes
    
    if (this.currentLogSize >= maxSizeBytes) {
      this.rotateLogFile();
    }
  }

  private rotateLogFile(): void {
    this.logFileIndex++;
    
    // Clean up old log files if we exceed maxFiles
    if (this.logFileIndex >= this.config.maxFiles) {
      const oldestLogFile = path.join(
        path.dirname(this.currentLogFile),
        `stress-test-${this.logFileIndex - this.config.maxFiles}.log`
      );
      const oldestJsonFile = oldestLogFile.replace('.log', '.json');
      
      try {
        if (fs.existsSync(oldestLogFile)) fs.unlinkSync(oldestLogFile);
        if (fs.existsSync(oldestJsonFile)) fs.unlinkSync(oldestJsonFile);
      } catch (error) {
        console.warn('Failed to clean up old log files:', error);
      }
    }

    // Create new log file
    this.currentLogFile = path.join(
      path.dirname(this.currentLogFile),
      `stress-test-${this.logFileIndex}.log`
    );
    this.currentLogSize = 0;

    this.info('Log file rotated', {
      newFile: this.currentLogFile,
      fileIndex: this.logFileIndex
    });
  }

  // Public logging methods
  public debug(message: string, metadata?: Record<string, any>, source?: string): void {
    this.log(LogLevel.DEBUG, message, metadata, source);
  }

  public info(message: string, metadata?: Record<string, any>, source?: string): void {
    this.log(LogLevel.INFO, message, metadata, source);
  }

  public warn(message: string, metadata?: Record<string, any>, source?: string): void {
    this.log(LogLevel.WARN, message, metadata, source);
  }

  public error(message: string, metadata?: Record<string, any>, source?: string): void {
    this.log(LogLevel.ERROR, message, metadata, source);
  }

  public critical(message: string, metadata?: Record<string, any>, source?: string): void {
    this.log(LogLevel.CRITICAL, message, metadata, source);
  }

  // Request-specific logging
  public logRequest(method: string, params: any, requestId?: string): void {
    this.debug('Request started', {
      method,
      params,
      requestId,
      timestamp: Date.now()
    }, 'REQUEST');
  }

  public logResponse(method: string, success: boolean, responseTime: number, requestId?: string, error?: string, response?: any): void {
    const level = success ? LogLevel.DEBUG : LogLevel.WARN;
    const message = success ? 'Request completed successfully' : 'Request failed';
    
    this.log(level, message, {
      method,
      success,
      responseTime,
      requestId,
      error,
      response: success ? response : undefined
    }, 'RESPONSE');
  }

  // Performance logging
  public logPerformance(metric: string, value: number, unit: string, context?: Record<string, any>): void {
    this.info('Performance metric', {
      metric,
      value,
      unit,
      context
    }, 'PERFORMANCE');
  }

  // QPS step logging
  public logQPSStep(qps: number, stepResult: any): void {
    this.info('QPS step completed', {
      targetQPS: qps,
      actualQPS: stepResult.actualQPS,
      totalRequests: stepResult.totalRequests,
      successfulRequests: stepResult.successfulRequests,
      failedRequests: stepResult.failedRequests,
      errorRate: (stepResult.failedRequests / stepResult.totalRequests * 100).toFixed(2),
      duration: stepResult.duration,
      endpointStats: stepResult.endpointStats
    }, 'QPS_STEP');
  }

  // Error analysis logging
  public logErrorAnalysis(errorSummary: Record<string, number>, totalErrors: number): void {
    this.warn('Error analysis', {
      errorSummary,
      totalErrors,
      topErrors: Object.entries(errorSummary)
        .sort(([,a], [,b]) => b - a)
        .slice(0, 5)
        .map(([type, count]) => ({ type, count, percentage: (count / totalErrors * 100).toFixed(2) }))
    }, 'ERROR_ANALYSIS');
  }

  // System metrics logging
  public logSystemMetrics(metrics: Record<string, number>): void {
    this.debug('System metrics', metrics, 'SYSTEM');
  }

  // Test session events
  public logSessionStart(config: any): void {
    this.info('Test session started', {
      config,
      timestamp: Date.now()
    }, 'SESSION');
  }

  public logSessionEnd(sessionResult: any): void {
    this.info('Test session completed', {
      sessionId: sessionResult.sessionId,
      duration: sessionResult.endTime - sessionResult.startTime,
      totalSteps: sessionResult.steps.length,
      totalRequests: sessionResult.overallStats.totalRequests,
      successRate: sessionResult.overallStats.overallSuccessRate,
      maxSustainableQPS: sessionResult.maxSustainableQPS,
      recommendedMaxQPS: sessionResult.recommendedMaxQPS
    }, 'SESSION');
  }

  // Endpoint-specific logging
  public logEndpointTest(endpoint: string, result: any): void {
    const level = result.success ? LogLevel.DEBUG : LogLevel.WARN;
    const message = result.success ? 'Endpoint test successful' : 'Endpoint test failed';
    
    this.log(level, message, {
      endpoint,
      success: result.success,
      responseTime: result.responseTime,
      requestSize: result.requestSize,
      responseSize: result.responseSize,
      error: result.error
    }, 'ENDPOINT');
  }

  // Connectivity testing
  public logConnectivityTest(endpoint: string, success: boolean, error?: string): void {
    const level = success ? LogLevel.INFO : LogLevel.ERROR;
    const message = success ? 'Connectivity test passed' : 'Connectivity test failed';
    
    this.log(level, message, {
      endpoint,
      success,
      error
    }, 'CONNECTIVITY');
  }

  // Configuration changes
  public logConfigChange(property: string, oldValue: any, newValue: any): void {
    this.info('Configuration changed', {
      property,
      oldValue,
      newValue
    }, 'CONFIG');
  }

  // Cleanup method
  public async close(): Promise<void> {
    // Clear the flush interval
    if (this.bufferFlushInterval) {
      clearInterval(this.bufferFlushInterval);
    }

    // Final flush
    this.flushBuffer();

    this.info('Logger closed', {
      finalBufferSize: this.logBuffer.length,
      currentLogFile: this.currentLogFile,
      currentLogSize: this.currentLogSize
    });
  }

  // Utility methods
  public setLogLevel(level: LogLevel): void {
    const oldLevel = this.config.level;
    this.config.level = level;
    this.logConfigChange('logLevel', LogLevel[oldLevel], LogLevel[level]);
  }

  public enableConsoleLogging(enabled: boolean): void {
    const oldValue = this.config.enableConsole;
    this.config.enableConsole = enabled;
    this.logConfigChange('enableConsole', oldValue, enabled);
  }

  public enableFileLogging(enabled: boolean): void {
    const oldValue = this.config.enableFile;
    this.config.enableFile = enabled;
    this.logConfigChange('enableFile', oldValue, enabled);
  }

  public getLogStats(): { 
    currentFile: string; 
    currentSize: number; 
    bufferSize: number; 
    fileIndex: number;
  } {
    return {
      currentFile: this.currentLogFile,
      currentSize: this.currentLogSize,
      bufferSize: this.logBuffer.length,
      fileIndex: this.logFileIndex
    };
  }

  // Search logs (simple text search)
  public searchLogs(query: string, maxResults: number = 100): LogEntry[] {
    const results: LogEntry[] = [];
    
    // Search in current buffer first
    for (const entry of this.logBuffer) {
      if (this.entryMatchesQuery(entry, query)) {
        results.push(entry);
        if (results.length >= maxResults) break;
      }
    }

    // TODO: Could be extended to search in log files
    // This would require reading and parsing log files
    
    return results;
  }

  private entryMatchesQuery(entry: LogEntry, query: string): boolean {
    const lowerQuery = query.toLowerCase();
    
    // Search in message
    if (entry.message.toLowerCase().includes(lowerQuery)) return true;
    
    // Search in metadata
    if (entry.metadata) {
      const metadataStr = JSON.stringify(entry.metadata).toLowerCase();
      if (metadataStr.includes(lowerQuery)) return true;
    }
    
    // Search in source
    if (entry.source && entry.source.toLowerCase().includes(lowerQuery)) return true;
    
    return false;
  }
}

// Export utility functions
export function createLogger(outputDir: string, sessionId: string, config?: Partial<LoggerConfig>): Logger {
  return new Logger(outputDir, sessionId, config);
}

export function formatDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  
  if (hours > 0) {
    return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  } else {
    return `${seconds}s`;
  }
}

export function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  
  return `${size.toFixed(2)} ${units[unitIndex]}`;
}