/**
 * 服务器状态工具函数
 */

export type StatusVariant = "success" | "warning" | "error" | "neutral";

/**
 * 根据服务器状态获取对应的徽章变体
 */
export function getStatusVariant(status: string | undefined): StatusVariant {
  switch (status) {
    case "Running":
      return "success";
    case "Starting":
    case "Stopping":
      return "warning";
    case "Error":
      return "error";
    default:
      return "neutral";
  }
}

/**
 * 根据服务器状态获取对应的中文文本
 */
export function getStatusText(status: string | undefined): string {
  switch (status) {
    case "Running":
      return "运行中";
    case "Starting":
      return "启动中";
    case "Stopping":
      return "停止中";
    case "Error":
      return "异常";
    default:
      return "已停止";
  }
}
