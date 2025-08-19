import si from 'systeminformation';
import pidusage from 'pidusage';
import { SystemMetrics } from '../types/index';

export class SystemMonitor {
  private isRunning = false;
  private monitoringInterval: NodeJS.Timeout | null = null;
  private processId: number;
  private networkBaseline: { bytesIn: number; bytesOut: number; timestamp: number } | null = null;

  constructor() {
    this.processId = process.pid;
  }

  async start(): Promise<void> {
    this.isRunning = true;
    
    // Get initial network baseline
    try {
      const networkStats = await si.networkStats();
      if (networkStats.length > 0) {
        this.networkBaseline = {
          bytesIn: networkStats[0].rx_bytes || 0,
          bytesOut: networkStats[0].tx_bytes || 0,
          timestamp: Date.now()
        };
      }
    } catch (error) {
      console.warn('Failed to get network baseline:', error);
    }
  }

  async stop(): Promise<void> {
    this.isRunning = false;
    
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = null;
    }
  }

  async collectMetrics(): Promise<SystemMetrics> {
    const timestamp = Date.now();
    
    try {
      const [
        cpuData,
        memoryData,
        diskData,
        networkData,
        loadData,
        processData
      ] = await Promise.all([
        this.getCpuMetrics(),
        this.getMemoryMetrics(),
        this.getDiskMetrics(),
        this.getNetworkMetrics(),
        this.getLoadMetrics(),
        this.getProcessMetrics()
      ]);

      return {
        timestamp,
        cpu: {
          usage: cpuData.usage,
          cores: cpuData.cores,
          loadAverage: loadData,
          contextSwitches: processData?.cswitch || 0,
          interrupts: processData?.interrupts || 0
        },
        memory: {
          total: memoryData.total,
          used: memoryData.used,
          available: memoryData.available,
          usage: memoryData.usage,
          cached: memoryData.cached,
          buffers: memoryData.buffers
        },
        disk: {
          readRate: diskData.readRate,
          writeRate: diskData.writeRate,
          ioWait: diskData.ioWait,
          queueLength: diskData.queueLength,
          usage: diskData.usage
        },
        network: {
          bytesIn: networkData.bytesIn,
          bytesOut: networkData.bytesOut,
          packetsIn: networkData.packetsIn,
          packetsOut: networkData.packetsOut,
          errors: networkData.errors,
          dropped: networkData.dropped,
          latency: networkData.latency
        }
      };
    } catch (error) {
      console.error('Error collecting system metrics:', error);
      return this.getDefaultMetrics(timestamp);
    }
  }

  private async getCpuMetrics(): Promise<{ usage: number; cores: number[] }> {
    try {
      const currentLoad = await si.currentLoad();
      const cpuData = await si.cpu();
      
      // Get per-core usage
      const coreUsages = currentLoad.cpus?.map(cpu => cpu.load) || [];
      
      return {
        usage: currentLoad.currentLoad || 0,
        cores: coreUsages
      };
    } catch (error) {
      console.warn('Failed to get CPU metrics:', error);
      return { usage: 0, cores: [] };
    }
  }

  private async getMemoryMetrics(): Promise<{
    total: number;
    used: number;
    available: number;
    usage: number;
    cached: number;
    buffers: number;
  }> {
    try {
      const memory = await si.mem();
      
      return {
        total: memory.total || 0,
        used: memory.used || 0,
        available: memory.available || 0,
        usage: ((memory.used || 0) / (memory.total || 1)) * 100,
        cached: memory.cached || 0,
        buffers: memory.buffers || 0
      };
    } catch (error) {
      console.warn('Failed to get memory metrics:', error);
      return { total: 0, used: 0, available: 0, usage: 0, cached: 0, buffers: 0 };
    }
  }

  private async getDiskMetrics(): Promise<{
    readRate: number;
    writeRate: number;
    ioWait: number;
    queueLength: number;
    usage: number;
  }> {
    try {
      const [diskIO, diskLayout, currentLoad] = await Promise.all([
        si.disksIO(),
        si.diskLayout(),
        si.currentLoad()
      ]);
      
      return {
        readRate: diskIO.rIO_sec || 0,
        writeRate: diskIO.wIO_sec || 0,
        ioWait: (currentLoad as any).currentLoadIowait || 0,
        queueLength: diskIO.tIO || 0,
        usage: diskLayout.length > 0 ? (diskLayout[0].size || 0) / (1024 * 1024 * 1024) : 0 // GB
      };
    } catch (error) {
      console.warn('Failed to get disk metrics:', error);
      return { readRate: 0, writeRate: 0, ioWait: 0, queueLength: 0, usage: 0 };
    }
  }

  private async getNetworkMetrics(): Promise<{
    bytesIn: number;
    bytesOut: number;
    packetsIn: number;
    packetsOut: number;
    errors: number;
    dropped: number;
    latency: number;
  }> {
    try {
      const networkStats = await si.networkStats();
      const currentTime = Date.now();
      
      if (networkStats.length === 0) {
        return { bytesIn: 0, bytesOut: 0, packetsIn: 0, packetsOut: 0, errors: 0, dropped: 0, latency: 0 };
      }
      
      const primaryInterface = networkStats[0];
      let bytesInRate = 0;
      let bytesOutRate = 0;
      
      // Calculate rates if we have baseline
      if (this.networkBaseline) {
        const timeDiff = (currentTime - this.networkBaseline.timestamp) / 1000;
        if (timeDiff > 0) {
          bytesInRate = ((primaryInterface.rx_bytes || 0) - this.networkBaseline.bytesIn) / timeDiff;
          bytesOutRate = ((primaryInterface.tx_bytes || 0) - this.networkBaseline.bytesOut) / timeDiff;
        }
        
        // Update baseline
        this.networkBaseline = {
          bytesIn: primaryInterface.rx_bytes || 0,
          bytesOut: primaryInterface.tx_bytes || 0,
          timestamp: currentTime
        };
      }
      
      // Estimate latency (simple ping to localhost)
      let latency = 0;
      try {
        const pingStart = Date.now();
        await this.simplePing();
        latency = Date.now() - pingStart;
      } catch {
        latency = 0;
      }
      
      return {
        bytesIn: Math.max(0, bytesInRate),
        bytesOut: Math.max(0, bytesOutRate),
        packetsIn: primaryInterface.rx_sec || 0,
        packetsOut: primaryInterface.tx_sec || 0,
        errors: (primaryInterface.rx_errors || 0) + (primaryInterface.tx_errors || 0),
        dropped: (primaryInterface.rx_dropped || 0) + (primaryInterface.tx_dropped || 0),
        latency
      };
    } catch (error) {
      console.warn('Failed to get network metrics:', error);
      return { bytesIn: 0, bytesOut: 0, packetsIn: 0, packetsOut: 0, errors: 0, dropped: 0, latency: 0 };
    }
  }

  private async getLoadMetrics(): Promise<number[]> {
    try {
      const currentLoad = await si.currentLoad();
      const avgLoad = (currentLoad as any).avgLoad;
      return avgLoad ? [
        avgLoad[0] || 0,
        avgLoad[1] || 0,
        avgLoad[2] || 0
      ] : [0, 0, 0];
    } catch (error) {
      console.warn('Failed to get load metrics:', error);
      return [0, 0, 0];
    }
  }

  private async getProcessMetrics(): Promise<{
    cswitch: number;
    interrupts: number;
  } | null> {
    try {
      const processStats = await pidusage(this.processId);
      return {
        cswitch: 0, // Not available in pidusage
        interrupts: 0 // Not available in pidusage
      };
    } catch (error) {
      console.warn('Failed to get process metrics:', error);
      return null;
    }
  }

  private simplePing(): Promise<void> {
    return new Promise(async (resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Ping timeout'));
      }, 1000);
      
      // Simple network check  
      const net = await import('net');
      const socket = new net.Socket();
      socket.setTimeout(1000);
      
      socket.connect(80, '127.0.0.1', () => {
        clearTimeout(timeout);
        socket.destroy();
        resolve();
      });
      
      socket.on('error', () => {
        clearTimeout(timeout);
        socket.destroy();
        reject(new Error('Connection failed'));
      });
      
      socket.on('timeout', () => {
        clearTimeout(timeout);
        socket.destroy();
        reject(new Error('Connection timeout'));
      });
    });
  }

  private getDefaultMetrics(timestamp: number): SystemMetrics {
    return {
      timestamp,
      cpu: {
        usage: 0,
        cores: [],
        loadAverage: [0, 0, 0],
        contextSwitches: 0,
        interrupts: 0
      },
      memory: {
        total: 0,
        used: 0,
        available: 0,
        usage: 0,
        cached: 0,
        buffers: 0
      },
      disk: {
        readRate: 0,
        writeRate: 0,
        ioWait: 0,
        queueLength: 0,
        usage: 0
      },
      network: {
        bytesIn: 0,
        bytesOut: 0,
        packetsIn: 0,
        packetsOut: 0,
        errors: 0,
        dropped: 0,
        latency: 0
      }
    };
  }

  // Real-time monitoring for dashboard
  startRealTimeMonitoring(callback: (metrics: SystemMetrics) => void, intervalMs = 1000): void {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
    }
    
    this.monitoringInterval = setInterval(async () => {
      if (this.isRunning) {
        try {
          const metrics = await this.collectMetrics();
          callback(metrics);
        } catch (error) {
          console.error('Error in real-time monitoring:', error);
        }
      }
    }, intervalMs);
  }
}