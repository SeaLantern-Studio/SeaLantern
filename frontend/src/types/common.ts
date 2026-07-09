/**
 * 通用类型定义
 */

// ID类型
export type ServerId = string;
export type PlayerId = string;

// 服务器状态
export type ServerStatus = "Stopped" | "Starting" | "Running" | "Stopping" | "Error";

// 通用结果类型
export interface Result<T, E = string> {
  ok: boolean;
  data?: T;
  error?: E;
}

// 日志级别
export type LogLevel = "info" | "warn" | "error" | "debug";

// 日志条目
export interface LogEntry {
  timestamp: number;
  level: LogLevel;
  message: string;
  source?: string;
}

// 分页参数
export interface Pagination {
  offset: number;
  limit: number;
}

// 分页结果
export interface PaginatedResult<T> {
  items: T[];
  total: number;
  offset: number;
  limit: number;
}
