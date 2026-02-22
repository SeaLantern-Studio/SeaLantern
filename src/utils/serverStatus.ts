/**
 * 服务器状态工具函数
 */

import { SERVER_STATUS } from "@utils/constants";
import { i18n } from "@language";

export type StatusVariant = "success" | "warning" | "error" | "neutral";

export type StatusClass = "running" | "starting" | "stopping" | "stopped" | "error";

/**
 * 根据服务器状态获取对应的徽章变体
 */
export function getStatusVariant(status: string | undefined): StatusVariant {
  switch (status) {
    case SERVER_STATUS.RUNNING:
      return "success";
    case SERVER_STATUS.STARTING:
    case SERVER_STATUS.STOPPING:
      return "warning";
    case SERVER_STATUS.ERROR:
      return "error";
    default:
      return "neutral";
  }
}

/**
 * 根据服务器状态获取对应的本地化文本
 */
export function getStatusText(status: string | undefined): string {
  switch (status) {
    case SERVER_STATUS.RUNNING:
      return i18n.t("console.running");
    case SERVER_STATUS.STARTING:
      return i18n.t("console.starting");
    case SERVER_STATUS.STOPPING:
      return i18n.t("console.stopping");
    case SERVER_STATUS.ERROR:
      return i18n.t("console.error");
    default:
      return i18n.t("console.stopped");
  }
}

/**
 * 根据服务器状态获取对应的 CSS 类名
 */
export function getStatusClass(status: string | undefined): StatusClass {
  switch (status) {
    case SERVER_STATUS.RUNNING:
      return "running";
    case SERVER_STATUS.STARTING:
      return "starting";
    case SERVER_STATUS.STOPPING:
      return "stopping";
    case SERVER_STATUS.ERROR:
      return "error";
    default:
      return "stopped";
  }
}

/**
 * 检查服务器是否正在运行
 */
export function isRunning(status: string | undefined): boolean {
  return status === SERVER_STATUS.RUNNING;
}

/**
 * 检查服务器是否已停止
 */
export function isStopped(status: string | undefined): boolean {
  return !status || status === SERVER_STATUS.STOPPED;
}

/**
 * 检查服务器是否处于过渡状态（启动中或停止中）
 */
export function isTransitioning(status: string | undefined): boolean {
  return status === SERVER_STATUS.STARTING || status === SERVER_STATUS.STOPPING;
}

/**
 * 检查服务器是否可以启动
 */
export function canStart(status: string | undefined): boolean {
  return isStopped(status) || status === SERVER_STATUS.ERROR;
}

/**
 * 检查服务器是否可以停止
 */
export function canStop(status: string | undefined): boolean {
  return status === SERVER_STATUS.RUNNING || status === SERVER_STATUS.STARTING;
}

/**
 * 获取服务器状态颜色（十六进制）
 */
export function getStatusColor(status: string | undefined): string {
  switch (status) {
    case SERVER_STATUS.RUNNING:
      return "#22c55e";
    case SERVER_STATUS.STARTING:
    case SERVER_STATUS.STOPPING:
      return "#f59e0b";
    case SERVER_STATUS.ERROR:
      return "#ef4444";
    default:
      return "#6b7280";
  }
}

/**
 * 服务器状态信息接口
 */
export interface ServerStatusInfo {
  status: string;
  variant: StatusVariant;
  text: string;
  className: StatusClass;
  color: string;
  canStart: boolean;
  canStop: boolean;
}

/**
 * 获取完整的服务器状态信息
 */
export function getServerStatusInfo(status: string | undefined): ServerStatusInfo {
  return {
    status: status || SERVER_STATUS.STOPPED,
    variant: getStatusVariant(status),
    text: getStatusText(status),
    className: getStatusClass(status),
    color: getStatusColor(status),
    canStart: canStart(status),
    canStop: canStop(status),
  };
}
