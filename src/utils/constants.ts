/**
 * 应用常量定义
 */

import { i18n } from "@language";

// ==================== 时间相关 ====================
export const TIME = {
  /** 轮询间隔（毫秒） */
  POLLING_INTERVAL: 800,
  /** 状态刷新间隔（毫秒） */
  STATUS_REFRESH_INTERVAL: 3000,
  /** 日志刷新延迟（毫秒） */
  LOG_REFRESH_DELAY: 1500,
  /** 成功提示显示时长（毫秒） */
  SUCCESS_MESSAGE_DURATION: 2000,
  /** 错误提示显示时长（毫秒） */
  ERROR_MESSAGE_DURATION: 3000,
} as const;

// ==================== 限制相关 ====================
export const LIMITS = {
  /** 最大日志行数 */
  MAX_LOG_LINES: 5000,
  /** 默认分页大小 */
  DEFAULT_PAGE_SIZE: 100,
  /** 玩家名最大长度 */
  MAX_PLAYER_NAME_LENGTH: 16,
  /** 服务器名最大长度 */
  MAX_SERVER_NAME_LENGTH: 50,
} as const;

// ==================== 默认值 ====================
export const DEFAULTS = {
  /** 默认控制台字体大小 */
  CONSOLE_FONT_SIZE: 13,
  /** 默认最大内存（MB） */
  DEFAULT_MAX_MEMORY: 2048,
  /** 默认最小内存（MB） */
  DEFAULT_MIN_MEMORY: 1024,
  /** 默认服务器端口 */
  DEFAULT_SERVER_PORT: 25565,
} as const;

// ==================== 服务器状态 ====================
export const SERVER_STATUS = {
  STOPPED: "Stopped",
  STARTING: "Starting",
  RUNNING: "Running",
  STOPPING: "Stopping",
  ERROR: "Error",
} as const;

export function getServerStatusText(status: string): string {
  const statusMap: Record<string, string> = {
    [SERVER_STATUS.STOPPED]: i18n.t("common.server_status_stopped"),
    [SERVER_STATUS.STARTING]: i18n.t("common.server_status_starting"),
    [SERVER_STATUS.RUNNING]: i18n.t("common.server_status_running"),
    [SERVER_STATUS.STOPPING]: i18n.t("common.server_status_stopping"),
    [SERVER_STATUS.ERROR]: i18n.t("common.server_status_error"),
  };
  return statusMap[status] || status;
}

// ==================== 日志前缀 ====================
export const LOG_PREFIX = {
  SEA_LANTERN: "[Sea Lantern]",
  ERROR: "[ERROR]",
  WARN: "[WARN]",
  STDERR: "[STDERR]",
} as const;

// ==================== 消息模板 ====================
export const MESSAGES = {
  // 成功消息
  SUCCESS: {
    SERVER_STARTED: "common.message_server_started",
    SERVER_STOPPED: "common.message_server_stopped",
    COMMAND_SENT: "common.message_command_sent",
    WHITELIST_ADDED: "common.message_whitelist_added",
    WHITELIST_REMOVED: "common.message_whitelist_removed",
    PLAYER_BANNED: "common.message_player_banned",
    PLAYER_UNBANNED: "common.message_player_unbanned",
    OP_ADDED: "common.message_op_added",
    OP_REMOVED: "common.message_op_removed",
    PLAYER_KICKED: "common.message_player_kicked",
  },
  // 错误消息
  ERROR: {
    SERVER_NOT_RUNNING: "common.message_server_not_running",
    SERVER_ALREADY_RUNNING: "common.message_server_already_running",
    SERVER_NOT_FOUND: "common.message_server_not_found",
    INVALID_PLAYER_NAME: "common.message_invalid_player_name",
    NETWORK_ERROR: "common.message_network_error",
    UNKNOWN_ERROR: "common.message_unknown_error",
  },
  // 提示消息
  HINT: {
    SELECT_SERVER: "common.message_select_server",
    SERVER_REQUIRED: "common.message_server_required",
    ENTER_PLAYER_NAME: "common.message_enter_player_name",
  },
} as const;

export function getMessage(key: string): string {
  return i18n.t(key);
}

// ==================== 正则表达式 ====================
export const REGEX = {
  /** Minecraft玩家名（3-16个字母数字下划线） */
  PLAYER_NAME: /^[a-zA-Z0-9_]{3,16}$/,
  /** UUID格式 */
  UUID: /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
  /** IP地址 */
  IP_ADDRESS: /^(\d{1,3}\.){3}\d{1,3}$/,
  /** 端口号 */
  PORT: /^([1-9]\d{0,3}|[1-5]\d{4}|6[0-4]\d{3}|65[0-4]\d{2}|655[0-2]\d|6553[0-5])$/,
} as const;

// ==================== 键盘快捷键 ====================
export const KEYBOARD = {
  ENTER: "Enter",
  ESCAPE: "Escape",
  TAB: "Tab",
  ARROW_UP: "ArrowUp",
  ARROW_DOWN: "ArrowDown",
  CTRL_C: "Control+C",
  CTRL_V: "Control+V",
} as const;
