/**
 * SeaLantern 控制台日志统一类型定义。
 *
 * 本文件集中定义控制台日志在前端（解析后）使用的数据结构，
 * 消除项目中原先碎片化的日志类型（ConsoleLineObj / ConsoleLine /
 * LogEntry / ServerLogLine 互相不一致的问题）。
 */

/** 日志行类型（渲染层用于着色与筛选） */
export type LogLineType = "input" | "output" | "error" | "warning" | "info" | "success" | "system";

/** 一条解析后的日志行对象（ConsoleOutput 内部维护的数据结构） */
export interface ConsoleLineObj {
  text: string;
  type?: LogLineType;
  timestamp?: string;
  /** 原始行在 lines 全量列表中的索引（搜索导航时映射回全量列表用） */
  sourceIndex?: number;
}

/** 日志筛选级别（对应 ConsoleLogList 的级别筛选按钮组） */
export type LogFilterLevel = "all" | "info" | "warn" | "error" | "debug";

/** 段着色类型（由解析日志行文本得到，用于分级着色） */
export type LogSegmentBracket = "level" | "time" | "meta" | null;
