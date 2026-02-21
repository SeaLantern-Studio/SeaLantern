/**
 * 格式化工具函数
 * 提供各种数据格式化功能
 */

/**
 * 格式化字节数为人类可读格式
 * @param bytes 字节数
 * @param decimals 小数位数
 * @returns 格式化后的字符串，如 "1.5 GB"
 */
export function formatBytes(bytes: number, decimals: number = 1): string {
  if (bytes === 0) return "0 B";

  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(decimals)) + " " + sizes[i];
}

/**
 * 格式化服务器路径，只显示 servers 目录及其后面的内容
 * @param path 完整路径
 * @returns 简化后的路径
 */
export function formatServerPath(path: string): string {
  const serversIndex = path.indexOf("servers/");
  if (serversIndex !== -1) {
    return path.substring(serversIndex);
  }

  const serversIndexBackslash = path.indexOf("servers\\");
  if (serversIndexBackslash !== -1) {
    return path.substring(serversIndexBackslash);
  }

  return path;
}

/**
 * 格式化运行时间（秒）为人类可读格式
 * @param seconds 秒数
 * @returns 格式化后的字符串，如 "2天 3小时 15分钟"
 */
export function formatUptime(seconds: number): string {
  if (seconds == null || seconds < 0) return "0秒";

  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  const parts: string[] = [];

  if (days > 0) parts.push(`${days}天`);
  if (hours > 0) parts.push(`${hours}小时`);
  if (minutes > 0) parts.push(`${minutes}分钟`);
  if (secs > 0 && parts.length === 0) parts.push(`${secs}秒`);

  return parts.length > 0 ? parts.join(" ") : "0秒";
}

/**
 * 格式化日期时间
 * @param date 日期对象或时间戳
 * @param format 格式类型
 * @returns 格式化后的字符串
 */
export function formatDateTime(
  date: Date | number | string,
  format: "full" | "date" | "time" | "datetime" = "datetime",
): string {
  const d = typeof date === "string" || typeof date === "number" ? new Date(date) : date;

  if (isNaN(d.getTime())) return "";

  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const hours = String(d.getHours()).padStart(2, "0");
  const minutes = String(d.getMinutes()).padStart(2, "0");
  const seconds = String(d.getSeconds()).padStart(2, "0");

  switch (format) {
    case "date":
      return `${year}-${month}-${day}`;
    case "time":
      return `${hours}:${minutes}:${seconds}`;
    case "full":
      return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
    case "datetime":
    default:
      return `${month}-${day} ${hours}:${minutes}`;
  }
}

/**
 * 格式化数字，添加千位分隔符
 * @param num 数字
 * @param decimals 小数位数
 * @returns 格式化后的字符串
 */
export function formatNumber(num: number, decimals: number = 0): string {
  return num.toLocaleString("zh-CN", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

/**
 * 格式化百分比
 * @param value 数值（0-100）
 * @param decimals 小数位数
 * @returns 格式化后的字符串
 */
export function formatPercent(value: number, decimals: number = 1): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * 格式化内存大小（MB）
 * @param mb MB 数
 * @returns 格式化后的字符串
 */
export function formatMemory(mb: number): string {
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)} GB`;
  }
  return `${mb} MB`;
}

/**
 * 截断字符串并添加省略号
 * @param str 原字符串
 * @param maxLength 最大长度
 * @param suffix 后缀
 * @returns 截断后的字符串
 */
export function truncate(str: string, maxLength: number, suffix: string = "..."): string {
  if (str.length <= maxLength) return str;
  return str.substring(0, maxLength - suffix.length) + suffix;
}

/**
 * 格式化玩家名称（确保符合 Minecraft 玩家名规范）
 * @param name 玩家名
 * @returns 格式化后的玩家名
 */
export function formatPlayerName(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_]/g, "")
    .substring(0, 16);
}

/**
 * 格式化端口号（确保在有效范围内）
 * @param port 端口号
 * @returns 格式化后的端口号字符串
 */
export function formatPort(port: number | string): string {
  const num = typeof port === "string" ? parseInt(port, 10) : port;
  if (isNaN(num) || num < 1 || num > 65535) {
    return "25565";
  }
  return String(num);
}

/**
 * 格式化 Java 版本显示名称
 * @param version 主版本号
 * @param vendor 供应商
 * @param is64bit 是否64位
 * @returns 格式化后的字符串
 */
export function formatJavaVersion(version: number, vendor: string, is64bit: boolean): string {
  let simplifiedVendor = vendor;

  if (vendor.includes("Oracle") || vendor.includes("Sun")) {
    simplifiedVendor = "Oracle";
  } else if (vendor.includes("Temurin") || vendor.includes("Adopt")) {
    simplifiedVendor = "Eclipse Temurin";
  } else if (vendor.includes("Amazon")) {
    simplifiedVendor = "Amazon Corretto";
  } else if (vendor.includes("Microsoft")) {
    simplifiedVendor = "Microsoft";
  } else if (vendor.includes("Zulu") || vendor.includes("Azul")) {
    simplifiedVendor = "Azul Zulu";
  } else if (vendor.includes("Liberica") || vendor.includes("BellSoft")) {
    simplifiedVendor = "Liberica";
  }

  const arch = is64bit ? "64-bit" : "32-bit";
  return `Java ${version} ${simplifiedVendor} ${arch}`;
}
