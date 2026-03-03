//! MCP 反作弊服务
//! 
//! 提供智能检测违规客户端模组和外挂的功能
//! 支持自动封禁、可疑程度评估、危害等级判断

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::{AIConfig, AIMessage, AIProvider, ai_provider::get_provider};

// ==================== 数据模型 ====================

/// 检测类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DetectionType {
    /// 加速外挂
    SpeedHack,
    /// 飞行外挂
    FlyHack,
    /// 杀戮光环
    KillAura,
    /// 透视外挂
    XRay,
    /// 自动点击
    AutoClicker,
    /// 攻击距离扩展
    ReachHack,
    /// 无摔落伤害
    NoFall,
    /// 无击退
    NoKnockback,
    /// 自动搭路
    Scaffold,
    /// 自动行走 (Baritone)
    Baritone,
    /// 违规模组
    IllegalMod,
    /// 复制漏洞利用
    DupeExploit,
    /// 异常行为
    AbnormalBehavior,
    /// 自定义类型
    Custom(String),
}

impl DetectionType {
    pub fn display_name(&self) -> &str {
        match self {
            DetectionType::SpeedHack => "加速外挂",
            DetectionType::FlyHack => "飞行外挂",
            DetectionType::KillAura => "杀戮光环",
            DetectionType::XRay => "透视外挂",
            DetectionType::AutoClicker => "自动点击",
            DetectionType::ReachHack => "攻击距离扩展",
            DetectionType::NoFall => "无摔落伤害",
            DetectionType::NoKnockback => "无击退",
            DetectionType::Scaffold => "自动搭路",
            DetectionType::Baritone => "自动行走",
            DetectionType::IllegalMod => "违规模组",
            DetectionType::DupeExploit => "复制漏洞利用",
            DetectionType::AbnormalBehavior => "异常行为",
            DetectionType::Custom(name) => name,
        }
    }

    pub fn category(&self) -> DetectionCategory {
        match self {
            DetectionType::SpeedHack | DetectionType::FlyHack | DetectionType::KillAura 
            | DetectionType::XRay | DetectionType::AutoClicker | DetectionType::ReachHack
            | DetectionType::NoFall | DetectionType::NoKnockback | DetectionType::Scaffold
            | DetectionType::Baritone => DetectionCategory::Hack,
            DetectionType::IllegalMod => DetectionCategory::Mod,
            DetectionType::DupeExploit => DetectionCategory::Exploit,
            DetectionType::AbnormalBehavior => DetectionCategory::Behavior,
            DetectionType::Custom(_) => DetectionCategory::Behavior,
        }
    }

    pub fn default_severity(&self) -> Severity {
        match self {
            DetectionType::KillAura | DetectionType::XRay | DetectionType::DupeExploit => Severity::Critical,
            DetectionType::SpeedHack | DetectionType::FlyHack | DetectionType::ReachHack 
            | DetectionType::Scaffold | DetectionType::Baritone => Severity::High,
            DetectionType::AutoClicker | DetectionType::NoFall | DetectionType::NoKnockback => Severity::Medium,
            DetectionType::IllegalMod => Severity::Medium,
            DetectionType::AbnormalBehavior => Severity::Low,
            DetectionType::Custom(_) => Severity::Medium,
        }
    }
}

/// 检测分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DetectionCategory {
    /// 外挂
    Hack,
    /// 违规模组
    Mod,
    /// 漏洞利用
    Exploit,
    /// 异常行为
    Behavior,
}

impl DetectionCategory {
    pub fn display_name(&self) -> &str {
        match self {
            DetectionCategory::Hack => "外挂",
            DetectionCategory::Mod => "违规模组",
            DetectionCategory::Exploit => "漏洞利用",
            DetectionCategory::Behavior => "异常行为",
        }
    }
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl Severity {
    pub fn display_name(&self) -> &str {
        match self {
            Severity::Low => "低",
            Severity::Medium => "中",
            Severity::High => "高",
            Severity::Critical => "严重",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Severity::Low => "#52c41a",
            Severity::Medium => "#faad14",
            Severity::High => "#ff7a45",
            Severity::Critical => "#ff4d4f",
        }
    }
}

/// 检测记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRecord {
    /// 记录 ID
    pub id: String,
    /// 玩家名称
    pub player_name: String,
    /// 玩家 UUID
    pub player_uuid: String,
    /// 检测类型
    pub detection_type: DetectionType,
    /// 检测分类
    pub category: DetectionCategory,
    /// 严重程度
    pub severity: Severity,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 可疑程度评分 (0 - 100)
    pub suspicion_score: u32,
    /// 危害程度评分 (0 - 100)
    pub danger_score: u32,
    /// 综合风险评分 (0 - 100)
    pub risk_score: u32,
    /// 检测时间戳
    pub timestamp: u64,
    /// 检测详情
    pub details: String,
    /// 相关日志
    pub related_logs: Vec<String>,
    /// 服务器 ID
    pub server_id: String,
    /// 处理状态
    pub status: DetectionStatus,
    /// 处理动作
    pub action_taken: Option<PunishmentAction>,
    /// 处理时间
    pub action_timestamp: Option<u64>,
}

impl DetectionRecord {
    pub fn new(
        player_name: String,
        player_uuid: String,
        detection_type: DetectionType,
        confidence: f32,
        details: String,
        server_id: String,
    ) -> Self {
        let category = detection_type.category();
        let severity = detection_type.default_severity();
        
        // 计算可疑程度
        let suspicion_score = Self::calculate_suspicion_score(confidence, &detection_type);
        
        // 计算危害程度
        let danger_score = Self::calculate_danger_score(&severity, &category);
        
        // 综合风险评分
        let risk_score = (suspicion_score as f32 * 0.6 + danger_score as f32 * 0.4) as u32;

        Self {
            id: generate_id(),
            player_name,
            player_uuid,
            detection_type,
            category,
            severity,
            confidence,
            suspicion_score,
            danger_score,
            risk_score,
            timestamp: current_timestamp(),
            details,
            related_logs: Vec::new(),
            server_id,
            status: DetectionStatus::Pending,
            action_taken: None,
            action_timestamp: None,
        }
    }

    /// 计算可疑程度评分
    fn calculate_suspicion_score(confidence: f32, detection_type: &DetectionType) -> u32 {
        let base_score = (confidence * 70.0) as u32;
        
        // 根据检测类型调整
        let type_modifier = match detection_type {
            DetectionType::KillAura | DetectionType::XRay => 20,
            DetectionType::SpeedHack | DetectionType::FlyHack | DetectionType::ReachHack => 15,
            DetectionType::Scaffold | DetectionType::Baritone => 10,
            DetectionType::AutoClicker | DetectionType::NoFall => 5,
            _ => 0,
        };

        (base_score + type_modifier).min(100)
    }

    /// 计算危害程度评分
    fn calculate_danger_score(severity: &Severity, category: &DetectionCategory) -> u32 {
        let severity_score = match severity {
            Severity::Critical => 90,
            Severity::High => 70,
            Severity::Medium => 50,
            Severity::Low => 25,
        };

        let category_modifier = match category {
            DetectionCategory::Hack => 10,
            DetectionCategory::Exploit => 15,
            DetectionCategory::Mod => 5,
            DetectionCategory::Behavior => 0,
        };

        (severity_score + category_modifier).min(100)
    }
}

/// 检测状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DetectionStatus {
    /// 待处理
    Pending,
    /// 已处理
    Processed,
    /// 已忽略
    Ignored,
    /// 误报
    FalsePositive,
}

/// 处罚动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PunishmentAction {
    /// 警告
    Warning,
    /// 踢出
    Kick,
    /// 临时封禁
    TemporaryBan { duration_hours: u32 },
    /// 永久封禁
    PermanentBan,
    /// 监控
    Monitor,
}

/// 检测规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    /// 规则 ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 检测类型
    pub detection_type: DetectionType,
    /// 严重程度
    pub severity: Severity,
    /// 是否启用
    pub enabled: bool,
    /// 触发阈值
    pub threshold: f32,
    /// 描述
    pub description: String,
    /// 触发条件 (JSON 格式)
    pub conditions: HashMap<String, serde_json::Value>,
    /// 处罚配置
    pub punishment_config: PunishmentConfig,
}

impl Default for DetectionRule {
    fn default() -> Self {
        Self {
            id: generate_id(),
            name: String::new(),
            detection_type: DetectionType::AbnormalBehavior,
            severity: Severity::Medium,
            enabled: true,
            threshold: 0.7,
            description: String::new(),
            conditions: HashMap::new(),
            punishment_config: PunishmentConfig::default(),
        }
    }
}

/// 处罚配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunishmentConfig {
    /// 启用自动处罚
    pub enabled: bool,
    /// 警告阈值
    pub warning_threshold: u32,
    /// 踢出阈值
    pub kick_threshold: u32,
    /// 封禁阈值
    pub ban_threshold: u32,
    /// 首次违规处罚
    pub first_offense: PunishmentAction,
    /// 二次违规处罚
    pub second_offense: PunishmentAction,
    /// 三次违规处罚
    pub third_offense: PunishmentAction,
    /// 启用递增处罚
    pub escalation_enabled: bool,
}

impl Default for PunishmentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            warning_threshold: 30,
            kick_threshold: 50,
            ban_threshold: 70,
            first_offense: PunishmentAction::Warning,
            second_offense: PunishmentAction::Kick,
            third_offense: PunishmentAction::TemporaryBan { duration_hours: 24 },
            escalation_enabled: true,
        }
    }
}

/// 玩家档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    /// 玩家名称
    pub player_name: String,
    /// 玩家 UUID
    pub player_uuid: String,
    /// 违规次数
    pub violation_count: u32,
    /// 累计风险评分
    pub total_risk_score: u32,
    /// 检测历史
    pub detection_history: Vec<String>, // Detection IDs
    /// 最后检测时间
    pub last_detection_time: Option<u64>,
    /// 当前状态
    pub status: PlayerStatus,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

impl PlayerProfile {
    pub fn new(player_name: String, player_uuid: String) -> Self {
        let now = current_timestamp();
        Self {
            player_name,
            player_uuid,
            violation_count: 0,
            total_risk_score: 0,
            detection_history: Vec::new(),
            last_detection_time: None,
            status: PlayerStatus::Clean,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_detection(&mut self, detection: &DetectionRecord) {
        self.violation_count += 1;
        self.total_risk_score += detection.risk_score;
        self.detection_history.push(detection.id.clone());
        self.last_detection_time = Some(detection.timestamp);
        self.updated_at = current_timestamp();

        // 更新玩家状态
        self.status = if self.violation_count >= 5 || self.total_risk_score >= 300 {
            PlayerStatus::Dangerous
        } else if self.violation_count >= 2 || self.total_risk_score >= 100 {
            PlayerStatus::Suspicious
        } else {
            PlayerStatus::Watched
        };
    }
}

/// 玩家状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
    /// 清白
    Clean,
    /// 被监控
    Watched,
    /// 可疑
    Suspicious,
    /// 危险
    Dangerous,
    /// 已封禁
    Banned,
}

/// MCP 反作弊服务
pub struct MCAntiCheatService {
    /// 检测记录
    detection_records: HashMap<String, DetectionRecord>,
    /// 玩家档案
    player_profiles: HashMap<String, PlayerProfile>,
    /// 检测规则
    detection_rules: HashMap<String, DetectionRule>,
    /// 全局处罚配置
    global_punishment_config: PunishmentConfig,
    /// 统计数据
    statistics: AntiCheatStatistics,
}

/// 反作弊统计数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AntiCheatStatistics {
    /// 今日检测数
    pub detections_today: u32,
    /// 今日封禁数
    pub bans_today: u32,
    /// 今日警告数
    pub warnings_today: u32,
    /// 今日踢出数
    pub kicks_today: u32,
    /// 总检测数
    pub total_detections: u64,
    /// 总封禁数
    pub total_bans: u64,
    /// 按类型统计
    pub by_type: HashMap<String, u32>,
    /// 按严重程度统计
    pub by_severity: HashMap<String, u32>,
    /// 统计日期
    pub stats_date: String,
}

impl MCAntiCheatService {
    /// 创建新的反作弊服务
    pub fn new() -> Self {
        Self {
            detection_records: HashMap::new(),
            player_profiles: HashMap::new(),
            detection_rules: Self::get_default_rules(),
            global_punishment_config: PunishmentConfig::default(),
            statistics: AntiCheatStatistics::default(),
        }
    }

    /// 获取默认检测规则
    fn get_default_rules() -> HashMap<String, DetectionRule> {
        let mut rules = HashMap::new();

        // 加速检测规则
        let speed_rule = DetectionRule {
            id: "rule_speed_hack".to_string(),
            name: "加速外挂检测".to_string(),
            detection_type: DetectionType::SpeedHack,
            severity: Severity::High,
            enabled: true,
            threshold: 0.75,
            description: "检测玩家移动速度是否超出正常范围".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("max_speed".to_string(), serde_json::json!(10.0));
                map.insert("check_interval_ms".to_string(), serde_json::json!(100));
                map
            },
            punishment_config: PunishmentConfig::default(),
        };
        rules.insert(speed_rule.id.clone(), speed_rule);

        // 飞行检测规则
        let fly_rule = DetectionRule {
            id: "rule_fly_hack".to_string(),
            name: "飞行外挂检测".to_string(),
            detection_type: DetectionType::FlyHack,
            severity: Severity::High,
            enabled: true,
            threshold: 0.8,
            description: "检测玩家是否存在非法飞行行为".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("max_air_time_ms".to_string(), serde_json::json!(3000));
                map.insert("check_fall_speed".to_string(), serde_json::json!(true));
                map
            },
            punishment_config: PunishmentConfig::default(),
        };
        rules.insert(fly_rule.id.clone(), fly_rule);

        // 杀戮光环检测规则
        let kill_aura_rule = DetectionRule {
            id: "rule_kill_aura".to_string(),
            name: "杀戮光环检测".to_string(),
            detection_type: DetectionType::KillAura,
            severity: Severity::Critical,
            enabled: true,
            threshold: 0.85,
            description: "检测玩家是否存在自动攻击行为".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("max_cps".to_string(), serde_json::json!(20));
                map.insert("check_rotation".to_string(), serde_json::json!(true));
                map.insert("max_targets_per_second".to_string(), serde_json::json!(5));
                map
            },
            punishment_config: PunishmentConfig {
                enabled: true,
                warning_threshold: 20,
                kick_threshold: 40,
                ban_threshold: 60,
                first_offense: PunishmentAction::Kick,
                second_offense: PunishmentAction::TemporaryBan { duration_hours: 24 },
                third_offense: PunishmentAction::PermanentBan,
                escalation_enabled: true,
            },
        };
        rules.insert(kill_aura_rule.id.clone(), kill_aura_rule);

        // 透视检测规则
        let xray_rule = DetectionRule {
            id: "rule_xray".to_string(),
            name: "透视外挂检测".to_string(),
            detection_type: DetectionType::XRay,
            severity: Severity::Critical,
            enabled: true,
            threshold: 0.7,
            description: "分析玩家挖矿模式检测透视行为".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("rare_ore_ratio_threshold".to_string(), serde_json::json!(3.0));
                map.insert("min_blocks_broken".to_string(), serde_json::json!(100));
                map
            },
            punishment_config: PunishmentConfig {
                enabled: true,
                warning_threshold: 25,
                kick_threshold: 45,
                ban_threshold: 65,
                first_offense: PunishmentAction::TemporaryBan { duration_hours: 72 },
                second_offense: PunishmentAction::PermanentBan,
                third_offense: PunishmentAction::PermanentBan,
                escalation_enabled: true,
            },
        };
        rules.insert(xray_rule.id.clone(), xray_rule);

        // 自动点击检测规则
        let auto_clicker_rule = DetectionRule {
            id: "rule_auto_clicker".to_string(),
            name: "自动点击检测".to_string(),
            detection_type: DetectionType::AutoClicker,
            severity: Severity::Medium,
            enabled: true,
            threshold: 0.7,
            description: "检测玩家点击模式是否为自动化".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("max_cps".to_string(), serde_json::json!(15));
                map.insert("consistency_threshold".to_string(), serde_json::json!(0.95));
                map
            },
            punishment_config: PunishmentConfig::default(),
        };
        rules.insert(auto_clicker_rule.id.clone(), auto_clicker_rule);

        // 攻击距离检测规则
        let reach_rule = DetectionRule {
            id: "rule_reach_hack".to_string(),
            name: "攻击距离扩展检测".to_string(),
            detection_type: DetectionType::ReachHack,
            severity: Severity::High,
            enabled: true,
            threshold: 0.8,
            description: "检测玩家攻击距离是否超出正常范围".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("max_reach_blocks".to_string(), serde_json::json!(3.5));
                map.insert("samples_required".to_string(), serde_json::json!(10));
                map
            },
            punishment_config: PunishmentConfig::default(),
        };
        rules.insert(reach_rule.id.clone(), reach_rule);

        rules
    }

    /// 处理玩家行为数据
    pub async fn analyze_player_behavior(
        &self,
        player_name: &str,
        player_uuid: &str,
        behavior_data: &PlayerBehaviorData,
        config: &AIConfig,
    ) -> Option<DetectionRecord> {
        // 本地规则检测
        if let Some(detection) = self.local_behavior_check(player_name, player_uuid, behavior_data) {
            return Some(detection);
        }

        // AI 增强检测
        if config.enabled {
            return self.ai_behavior_analysis(player_name, player_uuid, behavior_data, config).await;
        }

        None
    }

    /// 本地行为检测
    fn local_behavior_check(
        &self,
        player_name: &str,
        player_uuid: &str,
        behavior_data: &PlayerBehaviorData,
    ) -> Option<DetectionRecord> {
        // 检测速度
        if behavior_data.move_speed > 10.0 {
            let confidence = ((behavior_data.move_speed - 10.0) / 20.0).min(1.0) as f32;
            if confidence >= 0.5 {
                return Some(DetectionRecord::new(
                    player_name.to_string(),
                    player_uuid.to_string(),
                    DetectionType::SpeedHack,
                    confidence,
                    format!("移动速度异常: {:.2} blocks/s", behavior_data.move_speed),
                    behavior_data.server_id.clone(),
                ));
            }
        }

        // 检测飞行
        if behavior_data.is_flying && !behavior_data.has_flight_permission {
            return Some(DetectionRecord::new(
                player_name.to_string(),
                player_uuid.to_string(),
                DetectionType::FlyHack,
                0.9,
                "未授权飞行行为".to_string(),
                behavior_data.server_id.clone(),
            ));
        }

        // 检测 CPS
        if behavior_data.clicks_per_second > 20 {
            let confidence = ((behavior_data.clicks_per_second - 20) as f32 / 30.0).min(1.0);
            if confidence >= 0.5 {
                return Some(DetectionRecord::new(
                    player_name.to_string(),
                    player_uuid.to_string(),
                    DetectionType::AutoClicker,
                    confidence,
                    format!("点击速度异常: {} CPS", behavior_data.clicks_per_second),
                    behavior_data.server_id.clone(),
                ));
            }
        }

        // 检测攻击距离
        if behavior_data.attack_reach > 3.5 {
            let confidence = ((behavior_data.attack_reach - 3.5) / 3.0).min(1.0) as f32;
            if confidence >= 0.6 {
                return Some(DetectionRecord::new(
                    player_name.to_string(),
                    player_uuid.to_string(),
                    DetectionType::ReachHack,
                    confidence,
                    format!("攻击距离异常: {:.2} blocks", behavior_data.attack_reach),
                    behavior_data.server_id.clone(),
                ));
            }
        }

        None
    }

    /// AI 行为分析
    async fn ai_behavior_analysis(
        &self,
        player_name: &str,
        player_uuid: &str,
        behavior_data: &PlayerBehaviorData,
        config: &AIConfig,
    ) -> Option<DetectionRecord> {
        let provider = get_provider(&config.provider);

        let system_prompt = r#"你是一个 Minecraft 服务器反作弊专家。分析玩家行为数据，判断是否存在作弊行为。

返回 JSON 格式：
{
    "detected": true/false,
    "type": "检测类型 (speed_hack, fly_hack, kill_aura, xray, auto_clicker, reach_hack, no_fall, scaffold, abnormal_behavior)",
    "confidence": 0.0-1.0,
    "severity": "low/medium/high/critical",
    "details": "详细说明"
}

如果未检测到异常，返回 {"detected": false}"#;

        let user_message = format!(
            "玩家: {} ({})\n服务器: {}\n\n行为数据:\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            player_name,
            player_uuid,
            behavior_data.server_id,
            format!("移动速度: {:.2} blocks/s", behavior_data.move_speed),
            format!("是否飞行: {}", behavior_data.is_flying),
            format!("点击速度: {} CPS", behavior_data.clicks_per_second),
            format!("攻击距离: {:.2} blocks", behavior_data.attack_reach),
            format!("挖矿统计: {} 普通 / {} 稀有", behavior_data.blocks_broken_normal, behavior_data.blocks_broken_rare),
            format!("攻击目标数: {}", behavior_data.targets_attacked),
            format!("摔落伤害豁免次数: {}", behavior_data.no_fall_count),
            format!("击退豁免次数: {}", behavior_data.no_knockback_count),
        );

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: user_message,
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;

        if response.success {
            if let Some(content) = response.content {
                return self.parse_ai_detection(&content, player_name, player_uuid, &behavior_data.server_id);
            }
        }

        None
    }

    /// 解析 AI 检测结果
    fn parse_ai_detection(
        &self,
        content: &str,
        player_name: &str,
        player_uuid: &str,
        server_id: &str,
    ) -> Option<DetectionRecord> {
        let json_start = content.find('{');
        let json_end = content.rfind('}');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &content[start..=end];

            #[derive(Deserialize)]
            struct AIDetectionResult {
                detected: bool,
                #[serde(rename = "type")]
                detection_type: Option<String>,
                confidence: Option<f32>,
                severity: Option<String>,
                details: Option<String>,
            }

            if let Ok(result) = serde_json::from_str::<AIDetectionResult>(json_str) {
                if !result.detected {
                    return None;
                }

                let detection_type = result.detection_type
                    .and_then(|t| match t.as_str() {
                        "speed_hack" => Some(DetectionType::SpeedHack),
                        "fly_hack" => Some(DetectionType::FlyHack),
                        "kill_aura" => Some(DetectionType::KillAura),
                        "xray" => Some(DetectionType::XRay),
                        "auto_clicker" => Some(DetectionType::AutoClicker),
                        "reach_hack" => Some(DetectionType::ReachHack),
                        "no_fall" => Some(DetectionType::NoFall),
                        "scaffold" => Some(DetectionType::Scaffold),
                        "abnormal_behavior" => Some(DetectionType::AbnormalBehavior),
                        _ => None,
                    })
                    .unwrap_or(DetectionType::AbnormalBehavior);

                return Some(DetectionRecord::new(
                    player_name.to_string(),
                    player_uuid.to_string(),
                    detection_type,
                    result.confidence.unwrap_or(0.7),
                    result.details.unwrap_or_else(|| "AI 检测到异常行为".to_string()),
                    server_id.to_string(),
                ));
            }
        }

        None
    }

    /// 分析客户端模组列表
    pub async fn analyze_client_mods(
        &self,
        player_name: &str,
        player_uuid: &str,
        mods: &[ModInfo],
        config: &AIConfig,
    ) -> Vec<DetectionRecord> {
        let mut detections = Vec::new();

        // 本地黑名单检测
        let blacklist = get_mod_blacklist();
        for mod_info in mods {
            if let Some(blacklisted) = blacklist.get(&mod_info.mod_id.to_lowercase()) {
                detections.push(DetectionRecord::new(
                    player_name.to_string(),
                    player_uuid.to_string(),
                    DetectionType::IllegalMod,
                    blacklisted.confidence,
                    format!("检测到违规模组: {} - {}", mod_info.name, blacklisted.reason),
                    mod_info.server_id.clone(),
                ));
            }
        }

        // AI 增强模组分析
        if config.enabled && !mods.is_empty() {
            if let Some(detection) = self.ai_mod_analysis(player_name, player_uuid, mods, config).await {
                detections.push(detection);
            }
        }

        detections
    }

    /// AI 模组分析
    async fn ai_mod_analysis(
        &self,
        player_name: &str,
        player_uuid: &str,
        mods: &[ModInfo],
        config: &AIConfig,
    ) -> Option<DetectionRecord> {
        let provider = get_provider(&config.provider);

        let mod_list: Vec<String> = mods.iter().map(|m| format!("{} ({})", m.name, m.mod_id)).collect();

        let system_prompt = r#"你是一个 Minecraft 模组安全专家。分析模组列表，识别可能的作弊模组或违规模组。

返回 JSON 格式：
{
    "detected": true/false,
    "mod_name": "违规模组名",
    "confidence": 0.0-1.0,
    "severity": "low/medium/high/critical",
    "reason": "违规原因",
    "category": "hack/utility/unfair_advantage/information"
}

常见作弊模组类型：
- Kill Aura / Aura / KillAura
- X-Ray / XRay / Ore ESP
- Flight / Fly / FlyHack
- Speed / SpeedHack / Blink
- Scaffold / AutoBuild
- AutoClicker / Auto Clicker
- Reach / ReachHack
- ESP / Tracers / Wallhack
- NoFall / NoFallDamage
- Baritone (某些配置)"#;

        let messages = vec![
            AIMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            AIMessage {
                role: "user".to_string(),
                content: format!(
                    "玩家: {}\nUUID: {}\n\n模组列表:\n{}",
                    player_name,
                    player_uuid,
                    mod_list.join("\n")
                ),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        let response = provider.chat(&messages, config).await;

        if response.success {
            if let Some(content) = response.content {
                return self.parse_ai_mod_detection(&content, player_name, player_uuid, &mods.first().unwrap().server_id);
            }
        }

        None
    }

    /// 解析 AI 模组检测结果
    fn parse_ai_mod_detection(
        &self,
        content: &str,
        player_name: &str,
        player_uuid: &str,
        server_id: &str,
    ) -> Option<DetectionRecord> {
        let json_start = content.find('{');
        let json_end = content.rfind('}');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &content[start..=end];

            #[derive(Deserialize)]
            struct AIModDetectionResult {
                detected: bool,
                mod_name: Option<String>,
                confidence: Option<f32>,
                severity: Option<String>,
                reason: Option<String>,
            }

            if let Ok(result) = serde_json::from_str::<AIModDetectionResult>(json_str) {
                if !result.detected {
                    return None;
                }

                return Some(DetectionRecord::new(
                    player_name.to_string(),
                    player_uuid.to_string(),
                    DetectionType::IllegalMod,
                    result.confidence.unwrap_or(0.7),
                    result.reason.unwrap_or_else(|| format!("检测到违规模组: {}", result.mod_name.unwrap_or_default())),
                    server_id.to_string(),
                ));
            }
        }

        None
    }

    /// 根据风险评分决定处罚动作
    pub fn determine_punishment(&self, detection: &DetectionRecord) -> PunishmentAction {
        let config = &self.global_punishment_config;

        if !config.enabled {
            return PunishmentAction::Monitor;
        }

        // 根据风险评分决定
        if detection.risk_score >= config.ban_threshold {
            // 检查玩家历史违规次数
            if let Some(profile) = self.player_profiles.get(&detection.player_uuid) {
                if config.escalation_enabled {
                    return match profile.violation_count {
                        0 => config.first_offense.clone(),
                        1 => config.second_offense.clone(),
                        _ => config.third_offense.clone(),
                    };
                }
            }
            return config.third_offense.clone();
        } else if detection.risk_score >= config.kick_threshold {
            return PunishmentAction::Kick;
        } else if detection.risk_score >= config.warning_threshold {
            return PunishmentAction::Warning;
        }

        PunishmentAction::Monitor
    }

    /// 添加检测记录
    pub fn add_detection(&mut self, detection: DetectionRecord) {
        // 更新玩家档案
        let profile = self.player_profiles
            .entry(detection.player_uuid.clone())
            .or_insert_with(|| PlayerProfile::new(detection.player_name.clone(), detection.player_uuid.clone()));
        profile.add_detection(&detection);

        // 更新统计
        self.statistics.detections_today += 1;
        self.statistics.total_detections += 1;
        
        let type_key = format!("{:?}", detection.detection_type);
        *self.statistics.by_type.entry(type_key).or_insert(0) += 1;
        
        let severity_key = format!("{:?}", detection.severity);
        *self.statistics.by_severity.entry(severity_key).or_insert(0) += 1;

        // 保存记录
        self.detection_records.insert(detection.id.clone(), detection);
    }

    /// 执行处罚
    pub fn execute_punishment(&mut self, detection_id: &str, action: PunishmentAction) -> bool {
        if let Some(detection) = self.detection_records.get_mut(detection_id) {
            detection.action_taken = Some(action.clone());
            detection.action_timestamp = Some(current_timestamp());
            detection.status = DetectionStatus::Processed;

            // 更新统计
            match action {
                PunishmentAction::Warning => self.statistics.warnings_today += 1,
                PunishmentAction::Kick => self.statistics.kicks_today += 1,
                PunishmentAction::TemporaryBan { .. } | PunishmentAction::PermanentBan => {
                    self.statistics.bans_today += 1;
                    self.statistics.total_bans += 1;
                    if let Some(profile) = self.player_profiles.get_mut(&detection.player_uuid) {
                        profile.status = PlayerStatus::Banned;
                    }
                }
                _ => {}
            }

            return true;
        }
        false
    }

    /// 获取检测记录
    pub fn get_detections(&self) -> Vec<&DetectionRecord> {
        self.detection_records.values().collect()
    }

    /// 获取玩家档案
    pub fn get_player_profile(&self, uuid: &str) -> Option<&PlayerProfile> {
        self.player_profiles.get(uuid)
    }

    /// 获取检测规则
    pub fn get_rules(&self) -> Vec<&DetectionRule> {
        self.detection_rules.values().collect()
    }

    /// 添加/更新检测规则
    pub fn upsert_rule(&mut self, rule: DetectionRule) {
        self.detection_rules.insert(rule.id.clone(), rule);
    }

    /// 删除检测规则
    pub fn delete_rule(&mut self, rule_id: &str) -> bool {
        self.detection_rules.remove(rule_id).is_some()
    }

    /// 获取统计数据
    pub fn get_statistics(&self) -> &AntiCheatStatistics {
        &self.statistics
    }

    /// 重置每日统计
    pub fn reset_daily_stats(&mut self) {
        self.statistics.detections_today = 0;
        self.statistics.bans_today = 0;
        self.statistics.warnings_today = 0;
        self.statistics.kicks_today = 0;
        self.statistics.stats_date = today_date();
    }

    /// 更新全局处罚配置
    pub fn update_punishment_config(&mut self, config: PunishmentConfig) {
        self.global_punishment_config = config;
    }

    /// 获取全局处罚配置
    pub fn get_punishment_config(&self) -> &PunishmentConfig {
        &self.global_punishment_config
    }
}

impl Default for MCAntiCheatService {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 辅助结构 ====================

/// 玩家行为数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBehaviorData {
    /// 服务器 ID
    pub server_id: String,
    /// 移动速度
    pub move_speed: f64,
    /// 是否在飞行
    pub is_flying: bool,
    /// 是否有飞行权限
    pub has_flight_permission: bool,
    /// 点击速度 (CPS)
    pub clicks_per_second: u32,
    /// 攻击距离
    pub attack_reach: f64,
    /// 普通方块挖掘数
    pub blocks_broken_normal: u32,
    /// 稀有方块挖掘数
    pub blocks_broken_rare: u32,
    /// 攻击目标数
    pub targets_attacked: u32,
    /// 无摔落次数
    pub no_fall_count: u32,
    /// 无击退次数
    pub no_knockback_count: u32,
    /// 时间戳
    pub timestamp: u64,
}

impl Default for PlayerBehaviorData {
    fn default() -> Self {
        Self {
            server_id: String::new(),
            move_speed: 0.0,
            is_flying: false,
            has_flight_permission: false,
            clicks_per_second: 0,
            attack_reach: 0.0,
            blocks_broken_normal: 0,
            blocks_broken_rare: 0,
            targets_attacked: 0,
            no_fall_count: 0,
            no_knockback_count: 0,
            timestamp: current_timestamp(),
        }
    }
}

/// 模组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    /// 模组 ID
    pub mod_id: String,
    /// 模组名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 服务器 ID
    pub server_id: String,
}

/// 黑名单模组信息
struct BlacklistedMod {
    confidence: f32,
    reason: String,
}

/// 获取模组黑名单
fn get_mod_blacklist() -> HashMap<String, BlacklistedMod> {
    let mut blacklist = HashMap::new();

    // 作弊客户端
    let cheat_clients = vec![
        ("wurst", "Wurst 客户端"),
        ("aristois", "Aristois 客户端"),
        ("impact", "Impact 客户端"),
        ("meteor_client", "Meteor 客户端"),
        ("inertia", "Inertia 客户端"),
        ("liquidbounce", "LiquidBounce 客户端"),
        ("impact", "Impact 客户端"),
        ("future", "Future 客户端"),
        ("salhack", "SalHack 客户端"),
        ("konas", "Konas 客户端"),
        ("phobos", "Phobos 客户端"),
        ("gamesense", "GameSense 客户端"),
    ];

    for (id, name) in cheat_clients {
        blacklist.insert(id.to_string(), BlacklistedMod {
            confidence: 0.95,
            reason: format!("已知作弊客户端: {}", name),
        });
    }

    // 作弊模组
    let cheat_mods = vec![
        ("xray", "透视外挂"),
        ("x-ray", "透视外挂"),
        ("oreesp", "矿物透视"),
        ("esp", "ESP/透视"),
        ("tracers", "追踪线"),
        ("wallhack", "透视"),
        ("killaura", "杀戮光环"),
        ("kill_aura", "杀戮光环"),
        ("aura", "杀戮光环"),
        ("autoclicker", "自动点击"),
        ("auto_clicker", "自动点击"),
        ("autoclick", "自动点击"),
        ("fly", "飞行外挂"),
        ("flyhack", "飞行外挂"),
        ("flight", "飞行外挂"),
        ("speedhack", "加速外挂"),
        ("speed_hack", "加速外挂"),
        ("speed", "加速外挂"),
        ("scaffold", "自动搭路"),
        ("nofall", "无摔落"),
        ("no_fall", "无摔落"),
        ("noknockback", "无击退"),
        ("reach", "攻击距离扩展"),
        ("reachhack", "攻击距离扩展"),
        ("baritone", "自动行走 (可能作弊)"),
        ("autoeat", "自动进食"),
        ("auto_eat", "自动进食"),
        ("autoarmor", "自动装备"),
        ("auto_armor", "自动装备"),
        ("autosprint", "自动疾跑"),
        ("auto_sprint", "自动疾跑"),
        ("antiafk", "防AFK"),
        ("anti_afk", "防AFK"),
    ];

    for (id, name) in cheat_mods {
        blacklist.insert(id.to_string(), BlacklistedMod {
            confidence: 0.85,
            reason: format!("违规作弊模组: {}", name),
        });
    }

    blacklist
}

// ==================== 工具函数 ====================

/// 生成唯一 ID
fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 获取今日日期字符串
fn today_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_type_properties() {
        assert_eq!(DetectionType::KillAura.display_name(), "杀戮光环");
        assert_eq!(DetectionType::KillAura.category(), DetectionCategory::Hack);
        assert_eq!(DetectionType::KillAura.default_severity(), Severity::Critical);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_detection_record_creation() {
        let record = DetectionRecord::new(
            "TestPlayer".to_string(),
            "uuid-1234".to_string(),
            DetectionType::SpeedHack,
            0.85,
            "速度异常".to_string(),
            "server-1".to_string(),
        );

        assert!(record.confidence > 0.0);
        assert!(record.suspicion_score > 0);
        assert!(record.danger_score > 0);
        assert!(record.risk_score > 0);
    }

    #[test]
    fn test_player_profile() {
        let mut profile = PlayerProfile::new("TestPlayer".to_string(), "uuid-1234".to_string());
        
        let detection = DetectionRecord::new(
            "TestPlayer".to_string(),
            "uuid-1234".to_string(),
            DetectionType::SpeedHack,
            0.85,
            "速度异常".to_string(),
            "server-1".to_string(),
        );

        profile.add_detection(&detection);
        
        assert_eq!(profile.violation_count, 1);
        assert!(profile.total_risk_score > 0);
        assert_eq!(profile.status, PlayerStatus::Watched);
    }

    #[test]
    fn test_punishment_determination() {
        let service = MCAntiCheatService::new();
        
        let low_risk = DetectionRecord::new(
            "Player".to_string(),
            "uuid".to_string(),
            DetectionType::AbnormalBehavior,
            0.3,
            "低风险".to_string(),
            "server".to_string(),
        );
        
        let action = service.determine_punishment(&low_risk);
        assert_eq!(action, PunishmentAction::Monitor);
    }

    #[test]
    fn test_mod_blacklist() {
        let blacklist = get_mod_blacklist();
        assert!(blacklist.contains_key("wurst"));
        assert!(blacklist.contains_key("xray"));
        assert!(blacklist.contains_key("killaura"));
    }

    #[test]
    fn test_default_rules() {
        let service = MCAntiCheatService::new();
        let rules = service.get_rules();
        assert!(!rules.is_empty());
    }
}
