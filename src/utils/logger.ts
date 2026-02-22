/**
 * 日志工具
 */

import type { LogLevel, LogEntry } from "@type/common";

/**
 * 日志管理器
 */
class Logger {
  private isDev = import.meta.env.DEV;

  /**
   * 创建日志条目
   */
  private createEntry(level: LogLevel, message: string, source?: string): LogEntry {
    return {
      timestamp: Date.now(),
      level,
      message,
      source,
    };
  }

  /**
   * 格式化日志消息
   */
  private format(entry: LogEntry): string {
    const time = new Date(entry.timestamp).toLocaleTimeString();
    const source = entry.source ? `[${entry.source}]` : "";
    return `${time} ${source} ${entry.message}`;
  }

  /**
   * 输出日志
   */
  private output(entry: LogEntry) {
    if (!this.isDev && entry.level === "debug") {
      return; // 生产环境不输出debug日志
    }

    const formatted = this.format(entry);

    switch (entry.level) {
      case "error":
        console.error(formatted);
        break;
      case "warn":
        console.warn(formatted);
        break;
      case "debug":
        console.debug(formatted);
        break;
      default:
        console.log(formatted);
    }
  }

  /**
   * 记录信息日志
   */
  info(message: string, source?: string) {
    this.output(this.createEntry("info", message, source));
  }

  /**
   * 记录警告日志
   */
  warn(message: string, source?: string) {
    this.output(this.createEntry("warn", message, source));
  }

  /**
   * 记录错误日志
   */
  error(message: string, source?: string) {
    this.output(this.createEntry("error", message, source));
  }

  /**
   * 记录调试日志
   */
  debug(message: string, source?: string) {
    this.output(this.createEntry("debug", message, source));
  }

  /**
   * 记录分组开始
   */
  group(label: string) {
    if (this.isDev) {
      console.group(label);
    }
  }

  /**
   * 记录分组结束
   */
  groupEnd() {
    if (this.isDev) {
      console.groupEnd();
    }
  }
}

// 导出单例
export const logger = new Logger();
