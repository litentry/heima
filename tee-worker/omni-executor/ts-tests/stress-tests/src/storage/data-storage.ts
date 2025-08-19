import { promises as fs } from 'fs';
import { join, dirname } from 'path';
import ExcelJS from 'exceljs';
import { TestSession, TestSummary, RequestResult, SystemMetrics } from '../types/index';

export class DataStorage {
  private baseDir: string;

  constructor(baseDir = './stress-test-results') {
    this.baseDir = baseDir;
  }

  async initialize(): Promise<void> {
    await this.ensureDirectoryExists(this.baseDir);
  }

  async saveTestSession(session: TestSession): Promise<void> {
    await this.initialize();
    
    const sessionDir = join(this.baseDir, session.sessionId);
    await this.ensureDirectoryExists(sessionDir);
    
    // Save session data as JSON
    await this.saveSessionJson(session, sessionDir);
    
    // Save detailed Excel report
    await this.saveSessionExcel(session, sessionDir);
    
    // Save summary report
    if (session.summary) {
      await this.saveSummaryJson(session.summary, sessionDir);
    }
  }

  async loadTestSession(sessionId: string): Promise<TestSession | null> {
    try {
      const sessionPath = join(this.baseDir, sessionId, 'session.json');
      const data = await fs.readFile(sessionPath, 'utf-8');
      return JSON.parse(data);
    } catch (error) {
      console.error(`Failed to load session ${sessionId}:`, error);
      return null;
    }
  }

  async listTestSessions(): Promise<string[]> {
    try {
      await this.initialize();
      const entries = await fs.readdir(this.baseDir, { withFileTypes: true });
      return entries
        .filter(entry => entry.isDirectory())
        .map(entry => entry.name)
        .filter(name => name.startsWith('stress-test-'));
    } catch (error) {
      console.error('Failed to list sessions:', error);
      return [];
    }
  }

  async deleteTestSession(sessionId: string): Promise<boolean> {
    try {
      const sessionDir = join(this.baseDir, sessionId);
      await fs.rm(sessionDir, { recursive: true, force: true });
      return true;
    } catch (error) {
      console.error(`Failed to delete session ${sessionId}:`, error);
      return false;
    }
  }

  async exportSessionsToJson(sessionIds?: string[]): Promise<string> {
    const sessions: TestSession[] = [];
    const ids = sessionIds || await this.listTestSessions();
    
    for (const id of ids) {
      const session = await this.loadTestSession(id);
      if (session) {
        sessions.push(session);
      }
    }
    
    const exportData = {
      exportDate: new Date().toISOString(),
      sessionsCount: sessions.length,
      sessions
    };
    
    const exportPath = join(this.baseDir, `export-${Date.now()}.json`);
    await fs.writeFile(exportPath, JSON.stringify(exportData, null, 2));
    
    return exportPath;
  }

  private async saveSessionJson(session: TestSession, sessionDir: string): Promise<void> {
    const sessionPath = join(sessionDir, 'session.json');
    await fs.writeFile(sessionPath, JSON.stringify(session, null, 2));
  }

  private async saveSummaryJson(summary: TestSummary, sessionDir: string): Promise<void> {
    const summaryPath = join(sessionDir, 'summary.json');
    await fs.writeFile(summaryPath, JSON.stringify(summary, null, 2));
  }

  private async saveSessionExcel(session: TestSession, sessionDir: string): Promise<void> {
    const workbook = new ExcelJS.Workbook();
    
    // Session Overview Sheet
    const overviewSheet = workbook.addWorksheet('Overview');
    this.createOverviewSheet(overviewSheet, session);
    
    // Summary Sheet
    if (session.summary) {
      const summarySheet = workbook.addWorksheet('Summary');
      this.createSummarySheet(summarySheet, session.summary);
    }
    
    // Request Results Sheet
    const requestsSheet = workbook.addWorksheet('Request Results');
    this.createRequestsSheet(requestsSheet, session.results);
    
    // System Metrics Sheet
    const metricsSheet = workbook.addWorksheet('System Metrics');
    this.createSystemMetricsSheet(metricsSheet, session.systemMetrics);
    
    // Save workbook
    const excelPath = join(sessionDir, 'report.xlsx');
    await workbook.xlsx.writeFile(excelPath);
  }

  private createOverviewSheet(sheet: ExcelJS.Worksheet, session: TestSession): void {
    sheet.mergeCells('A1:B1');
    sheet.getCell('A1').value = 'Stress Test Session Overview';
    sheet.getCell('A1').font = { bold: true, size: 16 };
    
    const data = [
      ['Session ID', session.sessionId],
      ['Test Name', session.config.testName],
      ['Status', session.status],
      ['Start Time', new Date(session.startTime).toISOString()],
      ['End Time', session.endTime ? new Date(session.endTime).toISOString() : 'N/A'],
      ['Duration (seconds)', session.config.duration],
      ['Concurrency', session.config.concurrency],
      ['Target URL', session.config.targetUrl],
      ['Total Requests', session.results.length],
      ['Successful Requests', session.results.filter(r => r.success).length],
      ['Failed Requests', session.results.filter(r => !r.success).length]
    ];
    
    data.forEach((row, index) => {
      sheet.getCell(`A${index + 3}`).value = row[0];
      sheet.getCell(`B${index + 3}`).value = row[1];
      sheet.getCell(`A${index + 3}`).font = { bold: true };
    });
    
    sheet.getColumn('A').width = 25;
    sheet.getColumn('B').width = 35;
  }

  private createSummarySheet(sheet: ExcelJS.Worksheet, summary: TestSummary): void {
    sheet.mergeCells('A1:B1');
    sheet.getCell('A1').value = 'Performance Summary';
    sheet.getCell('A1').font = { bold: true, size: 16 };
    
    let rowIndex = 3;
    
    // Response Time Metrics
    sheet.getCell(`A${rowIndex}`).value = 'Response Time Metrics';
    sheet.getCell(`A${rowIndex}`).font = { bold: true };
    rowIndex++;
    
    const responseTimeData = [
      ['Average (ms)', summary.responseTime.average.toFixed(2)],
      ['Min (ms)', summary.responseTime.min.toFixed(2)],
      ['Max (ms)', summary.responseTime.max.toFixed(2)],
      ['50th Percentile (ms)', summary.responseTime.p50.toFixed(2)],
      ['90th Percentile (ms)', summary.responseTime.p90.toFixed(2)],
      ['95th Percentile (ms)', summary.responseTime.p95.toFixed(2)],
      ['99th Percentile (ms)', summary.responseTime.p99.toFixed(2)]
    ];
    
    responseTimeData.forEach(row => {
      sheet.getCell(`A${rowIndex}`).value = row[0];
      sheet.getCell(`B${rowIndex}`).value = row[1];
      rowIndex++;
    });
    
    rowIndex++; // Empty row
    
    // Throughput Metrics
    sheet.getCell(`A${rowIndex}`).value = 'Throughput Metrics';
    sheet.getCell(`A${rowIndex}`).font = { bold: true };
    rowIndex++;
    
    const throughputData = [
      ['Requests per Second', summary.throughput.rps.toFixed(2)],
      ['Transactions per Second', summary.throughput.tps.toFixed(2)],
      ['Data Transfer Rate (KB/s)', (summary.throughput.dataTransferRate / 1024).toFixed(2)],
      ['Peak Concurrency', summary.throughput.peakConcurrency]
    ];
    
    throughputData.forEach(row => {
      sheet.getCell(`A${rowIndex}`).value = row[0];
      sheet.getCell(`B${rowIndex}`).value = row[1];
      rowIndex++;
    });
    
    rowIndex++; // Empty row
    
    // Error Rate Metrics
    sheet.getCell(`A${rowIndex}`).value = 'Error Rate Metrics';
    sheet.getCell(`A${rowIndex}`).font = { bold: true };
    rowIndex++;
    
    const errorData = [
      ['Total Error Rate (%)', summary.errorRate.total.toFixed(2)],
      ['HTTP 4xx Error Rate (%)', summary.errorRate.http4xx.toFixed(2)],
      ['HTTP 5xx Error Rate (%)', summary.errorRate.http5xx.toFixed(2)],
      ['Timeout Error Rate (%)', summary.errorRate.timeout.toFixed(2)],
      ['Connection Error Rate (%)', summary.errorRate.connection.toFixed(2)]
    ];
    
    errorData.forEach(row => {
      sheet.getCell(`A${rowIndex}`).value = row[0];
      sheet.getCell(`B${rowIndex}`).value = row[1];
      rowIndex++;
    });
    
    sheet.getColumn('A').width = 30;
    sheet.getColumn('B').width = 20;
  }

  private createRequestsSheet(sheet: ExcelJS.Worksheet, results: RequestResult[]): void {
    const headers = [
      'Timestamp',
      'Request ID',
      'Method',
      'URL',
      'Status Code',
      'Response Time (ms)',
      'Success',
      'Error',
      'Request Size (bytes)',
      'Response Size (bytes)'
    ];
    
    // Add headers
    headers.forEach((header, index) => {
      const cell = sheet.getCell(1, index + 1);
      cell.value = header;
      cell.font = { bold: true };
      cell.fill = {
        type: 'pattern',
        pattern: 'solid',
        fgColor: { argb: 'FFCCCCCC' }
      };
    });
    
    // Add data
    results.forEach((result, index) => {
      const row = index + 2;
      sheet.getCell(row, 1).value = new Date(result.timestamp);
      sheet.getCell(row, 2).value = result.requestId;
      sheet.getCell(row, 3).value = result.method;
      sheet.getCell(row, 4).value = result.url;
      sheet.getCell(row, 5).value = result.statusCode;
      sheet.getCell(row, 6).value = result.responseTime;
      sheet.getCell(row, 7).value = result.success;
      sheet.getCell(row, 8).value = result.error || '';
      sheet.getCell(row, 9).value = result.requestSize;
      sheet.getCell(row, 10).value = result.responseSize;
      
      // Color code based on success
      if (!result.success) {
        for (let col = 1; col <= headers.length; col++) {
          sheet.getCell(row, col).fill = {
            type: 'pattern',
            pattern: 'solid',
            fgColor: { argb: 'FFFFCCCC' }
          };
        }
      }
    });
    
    // Auto-fit columns
    sheet.columns.forEach(column => {
      column.width = 15;
    });
    
    // Format timestamp column
    sheet.getColumn(1).numFmt = 'yyyy-mm-dd hh:mm:ss';
  }

  private createSystemMetricsSheet(sheet: ExcelJS.Worksheet, metrics: SystemMetrics[]): void {
    const headers = [
      'Timestamp',
      'CPU Usage (%)',
      'Memory Usage (%)',
      'Memory Used (MB)',
      'Load Average 1m',
      'Disk Read Rate (KB/s)',
      'Disk Write Rate (KB/s)',
      'Disk IO Wait (%)',
      'Network In (KB/s)',
      'Network Out (KB/s)',
      'Network Latency (ms)'
    ];
    
    // Add headers
    headers.forEach((header, index) => {
      const cell = sheet.getCell(1, index + 1);
      cell.value = header;
      cell.font = { bold: true };
      cell.fill = {
        type: 'pattern',
        pattern: 'solid',
        fgColor: { argb: 'FFCCCCCC' }
      };
    });
    
    // Add data
    metrics.forEach((metric, index) => {
      const row = index + 2;
      sheet.getCell(row, 1).value = new Date(metric.timestamp);
      sheet.getCell(row, 2).value = metric.cpu.usage.toFixed(2);
      sheet.getCell(row, 3).value = metric.memory.usage.toFixed(2);
      sheet.getCell(row, 4).value = (metric.memory.used / 1024 / 1024).toFixed(2);
      sheet.getCell(row, 5).value = metric.cpu.loadAverage[0]?.toFixed(2) || '0';
      sheet.getCell(row, 6).value = (metric.disk.readRate / 1024).toFixed(2);
      sheet.getCell(row, 7).value = (metric.disk.writeRate / 1024).toFixed(2);
      sheet.getCell(row, 8).value = metric.disk.ioWait.toFixed(2);
      sheet.getCell(row, 9).value = (metric.network.bytesIn / 1024).toFixed(2);
      sheet.getCell(row, 10).value = (metric.network.bytesOut / 1024).toFixed(2);
      sheet.getCell(row, 11).value = metric.network.latency.toFixed(2);
    });
    
    // Auto-fit columns
    sheet.columns.forEach(column => {
      column.width = 15;
    });
    
    // Format timestamp column
    sheet.getColumn(1).numFmt = 'yyyy-mm-dd hh:mm:ss';
  }

  private async ensureDirectoryExists(path: string): Promise<void> {
    try {
      await fs.access(path);
    } catch {
      await fs.mkdir(path, { recursive: true });
    }
  }
}