/**
 * MCP 反作弊 API
 *
 * 提供反作弊功能的前端调用接口
 */

import { invoke } from "@tauri-apps/api/core";

// ==================== 类型定义 ====================

/** 检测类型 */
export type DetectionTypeId =
  | "speed_hack"
  | "fly_hack"
  | "kill_aura"
  | "xray"
  | "auto_clicker"
  | "reach_hack"
  | "no_fall"
  | "no_knockback"
  | "scaffold"
  | "baritone"
  | "illegal_mod"
  | "dupe_exploit"
  | "abnormal_behavior";

/** 检测分类 */
export type DetectionCategory = "hack" | "mod" | "exploit" | "behavior";

/** 严重程度 */
export type Severity = "low" | "medium" | "high" | "critical";

/** 检测状态 */
export type DetectionStatus = "pending" | "processed" | "ignored" | "false_positive";

/** 玩家状态 */
export type PlayerStatus = "clean" | "watched" | "suspicious" | "dangerous" | "banned";

/** 处罚动作类型 */
export type PunishmentActionType =
  | "warning"
  | "kick"
  | "temporary_ban"
  | "permanent_ban"
  | "monitor";

/** 检测记录 */
export interface DetectionRecord {
  id: string;
  player_name: string;
  player_uuid: string;
  detection_type: DetectionTypeId;
  category: DetectionCategory;
  severity: Severity;
  confidence: number;
  suspicion_score: number;
  danger_score: number;
  risk_score: number;
  timestamp: number;
  details: string;
  related_logs: string[];
  server_id: string;
  status: DetectionStatus;
  action_taken: PunishmentAction | null;
  action_timestamp: number | null;
}

/** 处罚动作 */
export interface PunishmentAction {
  type: PunishmentActionType;
  duration_hours?: number;
}

/** 检测规则 */
export interface DetectionRule {
  id: string;
  name: string;
  detection_type: DetectionTypeId;
  severity: Severity;
  enabled: boolean;
  threshold: number;
  description: string;
  conditions: Record<string, unknown>;
  punishment_config: PunishmentConfig;
}

/** 处罚配置 */
export interface PunishmentConfig {
  enabled: boolean;
  warning_threshold: number;
  kick_threshold: number;
  ban_threshold: number;
  first_offense: PunishmentAction;
  second_offense: PunishmentAction;
  third_offense: PunishmentAction;
  escalation_enabled: boolean;
}

/** 玩家档案 */
export interface PlayerProfile {
  player_name: string;
  player_uuid: string;
  violation_count: number;
  total_risk_score: number;
  detection_history: string[];
  last_detection_time: number | null;
  status: PlayerStatus;
  created_at: number;
  updated_at: number;
}

/** 反作弊统计数据 */
export interface AntiCheatStatistics {
  detections_today: number;
  bans_today: number;
  warnings_today: number;
  kicks_today: number;
  total_detections: number;
  total_bans: number;
  by_type: Record<string, number>;
  by_severity: Record<string, number>;
  stats_date: string;
}

/** 玩家行为数据 */
export interface PlayerBehaviorData {
  server_id: string;
  player_name: string;
  player_uuid: string;
  move_speed: number;
  is_flying: boolean;
  has_flight_permission: boolean;
  clicks_per_second: number;
  attack_reach: number;
  blocks_broken_normal: number;
  blocks_broken_rare: number;
  targets_attacked: number;
  no_fall_count: number;
  no_knockback_count: number;
}

/** 模组信息 */
export interface ModInfo {
  mod_id: string;
  name: string;
  version: string;
}

/** 检测类型信息 */
export interface DetectionTypeInfo {
  id: string;
  name: string;
  category: string;
}

/** 检测记录过滤器 */
export interface DetectionFilter {
  detection_type?: string;
  severity?: string;
  status?: string;
  player_name?: string;
}

// ==================== 行为分析 ====================

/**
 * 分析玩家行为
 */
export async function analyzePlayerBehavior(
  data: PlayerBehaviorData,
): Promise<DetectionRecord | null> {
  return invoke<DetectionRecord | null>("analyze_player_behavior", { request: data });
}

/**
 * 分析客户端模组
 */
export async function analyzeClientMods(
  serverId: string,
  playerName: string,
  playerUuid: string,
  mods: ModInfo[],
): Promise<DetectionRecord[]> {
  return invoke<DetectionRecord[]>("analyze_client_mods", {
    request: {
      server_id: serverId,
      player_name: playerName,
      player_uuid: playerUuid,
      mods,
    },
  });
}

// ==================== 检测记录管理 ====================

/**
 * 提交检测记录
 */
export async function submitDetection(detection: DetectionRecord): Promise<void> {
  return invoke("submit_detection", { detection });
}

/**
 * 获取检测记录列表
 */
export async function getDetections(filter?: DetectionFilter): Promise<DetectionRecord[]> {
  return invoke<DetectionRecord[]>("get_detections", { filter });
}

/**
 * 确定处罚动作
 */
export async function determinePunishment(detectionId: string): Promise<PunishmentAction> {
  return invoke<PunishmentAction>("determine_punishment", { detectionId });
}

/**
 * 执行处罚
 */
export async function executePunishment(
  detectionId: string,
  action: PunishmentAction,
): Promise<boolean> {
  return invoke<boolean>("execute_punishment", { detectionId, action });
}

// ==================== 玩家档案 ====================

/**
 * 获取玩家档案
 */
export async function getPlayerProfile(playerUuid: string): Promise<PlayerProfile | null> {
  return invoke<PlayerProfile | null>("get_player_profile", { playerUuid });
}

// ==================== 规则管理 ====================

/**
 * 获取检测规则列表
 */
export async function getDetectionRules(): Promise<DetectionRule[]> {
  return invoke<DetectionRule[]>("get_detection_rules");
}

/**
 * 保存检测规则
 */
export async function saveDetectionRule(rule: DetectionRule): Promise<void> {
  return invoke("save_detection_rule", { rule });
}

/**
 * 删除检测规则
 */
export async function deleteDetectionRule(ruleId: string): Promise<boolean> {
  return invoke<boolean>("delete_detection_rule", { ruleId });
}

// ==================== 处罚配置 ====================

/**
 * 获取处罚配置
 */
export async function getPunishmentConfig(): Promise<PunishmentConfig> {
  return invoke<PunishmentConfig>("get_punishment_config");
}

/**
 * 更新处罚配置
 */
export async function updatePunishmentConfig(config: PunishmentConfig): Promise<void> {
  return invoke("update_punishment_config", { config });
}

// ==================== 统计数据 ====================

/**
 * 获取统计数据
 */
export async function getAntiCheatStatistics(): Promise<AntiCheatStatistics> {
  return invoke<AntiCheatStatistics>("get_anticheat_statistics");
}

/**
 * 重置每日统计
 */
export async function resetDailyStatistics(): Promise<void> {
  return invoke("reset_daily_statistics");
}

// ==================== 检测类型 ====================

/**
 * 获取所有检测类型
 */
export async function getDetectionTypes(): Promise<DetectionTypeInfo[]> {
  return invoke<DetectionTypeInfo[]>("get_detection_types");
}

// ==================== 工具函数 ====================

/**
 * 获取严重程度显示信息
 */
export function getSeverityInfo(severity: Severity): {
  text: string;
  color: string;
  level: number;
} {
  const map: Record<Severity, { text: string; color: string; level: number }> = {
    low: { text: "低", color: "#52c41a", level: 1 },
    medium: { text: "中", color: "#faad14", level: 2 },
    high: { text: "高", color: "#ff7a45", level: 3 },
    critical: { text: "严重", color: "#ff4d4f", level: 4 },
  };
  return map[severity];
}

/**
 * 获取检测状态显示信息
 */
export function getStatusInfo(status: DetectionStatus): { text: string; color: string } {
  const map: Record<DetectionStatus, { text: string; color: string }> = {
    pending: { text: "待处理", color: "#faad14" },
    processed: { text: "已处理", color: "#52c41a" },
    ignored: { text: "已忽略", color: "#8c8c8c" },
    false_positive: { text: "误报", color: "#1890ff" },
  };
  return map[status];
}

/**
 * 获取玩家状态显示信息
 */
export function getPlayerStatusInfo(status: PlayerStatus): { text: string; color: string } {
  const map: Record<PlayerStatus, { text: string; color: string }> = {
    clean: { text: "清白", color: "#52c41a" },
    watched: { text: "监控中", color: "#1890ff" },
    suspicious: { text: "可疑", color: "#faad14" },
    dangerous: { text: "危险", color: "#ff7a45" },
    banned: { text: "已封禁", color: "#ff4d4f" },
  };
  return map[status];
}

/**
 * 获取处罚动作显示信息
 */
export function getPunishmentActionInfo(action: PunishmentAction): string {
  switch (action.type) {
    case "warning":
      return "警告";
    case "kick":
      return "踢出服务器";
    case "temporary_ban":
      return `临时封禁 ${action.duration_hours} 小时`;
    case "permanent_ban":
      return "永久封禁";
    case "monitor":
      return "仅监控";
    default:
      return "未知";
  }
}

/**
 * 获取检测类型显示名称
 */
export function getDetectionTypeName(type: DetectionTypeId): string {
  const map: Record<DetectionTypeId, string> = {
    speed_hack: "加速外挂",
    fly_hack: "飞行外挂",
    kill_aura: "杀戮光环",
    xray: "透视外挂",
    auto_clicker: "自动点击",
    reach_hack: "攻击距离扩展",
    no_fall: "无摔落伤害",
    no_knockback: "无击退",
    scaffold: "自动搭路",
    baritone: "自动行走",
    illegal_mod: "违规模组",
    dupe_exploit: "复制漏洞利用",
    abnormal_behavior: "异常行为",
  };
  return map[type] || type;
}

/**
 * 获取分类显示名称
 */
export function getCategoryName(category: DetectionCategory): string {
  const map: Record<DetectionCategory, string> = {
    hack: "外挂",
    mod: "违规模组",
    exploit: "漏洞利用",
    behavior: "异常行为",
  };
  return map[category] || category;
}

/**
 * 格式化时间戳
 */
export function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

/**
 * 计算风险等级
 */
export function getRiskLevel(riskScore: number): { level: string; color: string } {
  if (riskScore >= 80) return { level: "极高风险", color: "#ff4d4f" };
  if (riskScore >= 60) return { level: "高风险", color: "#ff7a45" };
  if (riskScore >= 40) return { level: "中风险", color: "#faad14" };
  if (riskScore >= 20) return { level: "低风险", color: "#52c41a" };
  return { level: "极低风险", color: "#8c8c8c" };
}

// 默认导出
export default {
  // 行为分析
  analyzePlayerBehavior,
  analyzeClientMods,

  // 检测记录管理
  submitDetection,
  getDetections,
  determinePunishment,
  executePunishment,

  // 玩家档案
  getPlayerProfile,

  // 规则管理
  getDetectionRules,
  saveDetectionRule,
  deleteDetectionRule,

  // 处罚配置
  getPunishmentConfig,
  updatePunishmentConfig,

  // 统计数据
  getAntiCheatStatistics,
  resetDailyStatistics,

  // 检测类型
  getDetectionTypes,

  // 工具函数
  getSeverityInfo,
  getStatusInfo,
  getPlayerStatusInfo,
  getPunishmentActionInfo,
  getDetectionTypeName,
  getCategoryName,
  formatTimestamp,
  getRiskLevel,
};
