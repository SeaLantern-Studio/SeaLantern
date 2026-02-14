export type ServerId = string;
export type PlayerId = string;

export type ServerStatus = "Stopped" | "Starting" | "Running" | "Stopping" | "Error";

export interface Result<T, E = string> {
  ok: boolean;
  data?: T;
  error?: E;
}

export type LogLevel = "info" | "warn" | "error" | "debug";

export interface LogEntry {
  timestamp: number;
  level: LogLevel;
  message: string;
  source?: string;
}

export interface Pagination {
  offset: number;
  limit: number;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  offset: number;
  limit: number;
}

export type ColorScheme = "blue" | "cyan" | "green" | "orange" | "purple" | "pink" | "red" | "indigo" | "custom";

export type ColorMode = "light" | "dark" | "auto";

export interface CustomColorScheme {
  primary: string;
  primaryLight: string;
  primaryDark: string;
  primaryBg: string;
  accent: string;
  accentLight: string;
  success: string;
  warning: string;
  error: string;
  info: string;
}
