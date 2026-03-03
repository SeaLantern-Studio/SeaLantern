<template>
  <div class="anticheat-view">
    <!-- 标题栏 -->
    <div class="anticheat-header">
      <div class="anticheat-title">
        <Shield class="anticheat-icon" />
        <div>
          <h2>{{ t('ai.anticheat.title') }}</h2>
          <p class="subtitle">{{ t('ai.anticheat.subtitle') }}</p>
        </div>
      </div>
      <div class="anticheat-status" :class="{ enabled: config.enabled }">
        <span class="status-dot"></span>
        {{ config.enabled ? t('ai.anticheat.status.enabled') : t('ai.anticheat.status.disabled') }}
      </div>
    </div>

    <!-- Tab 切换 -->
    <SLTabBar :tabs="tabs" v-model="activeTab" />

    <!-- 监控面板 -->
    <div v-if="activeTab === 'dashboard'" class="tab-content">
      <div class="stats-grid">
        <div class="stat-card">
          <Users class="stat-icon" />
          <div class="stat-info">
            <span class="stat-value">{{ onlinePlayers }}</span>
            <span class="stat-label">{{ t('ai.anticheat.dashboard.online_players') }}</span>
          </div>
        </div>
        <div class="stat-card">
          <AlertTriangle class="stat-icon warning" />
          <div class="stat-info">
            <span class="stat-value">{{ stats.detections_today }}</span>
            <span class="stat-label">{{ t('ai.anticheat.dashboard.detections_today') }}</span>
          </div>
        </div>
        <div class="stat-card">
          <Ban class="stat-icon danger" />
          <div class="stat-info">
            <span class="stat-value">{{ stats.bans_today }}</span>
            <span class="stat-label">{{ t('ai.anticheat.dashboard.bans_today') }}</span>
          </div>
        </div>
        <div class="stat-card">
          <Activity class="stat-icon info" />
          <div class="stat-info">
            <span class="stat-value">{{ activeThreats }}</span>
            <span class="stat-label">{{ t('ai.anticheat.dashboard.active_threats') }}</span>
          </div>
        </div>
      </div>

      <!-- 最近检测 -->
      <div class="section-card">
        <h3>{{ t('ai.anticheat.dashboard.recent_detections') }}</h3>
        <div v-if="recentDetections.length === 0" class="empty-state">
          <CheckCircle class="empty-icon" />
          <p>暂无检测记录</p>
        </div>
        <div v-else class="detection-list">
          <div
            v-for="detection in recentDetections"
            :key="detection.id"
            class="detection-item"
            :class="detection.severity"
          >
            <div class="detection-main">
              <span class="player-name">{{ detection.player_name }}</span>
              <span class="detection-type">{{ getDetectionTypeName(detection.detection_type) }}</span>
              <span class="risk-badge" :style="{ backgroundColor: getRiskLevel(detection.risk_score).color }">
                {{ detection.risk_score }}
              </span>
            </div>
            <div class="detection-meta">
              <span class="time">{{ formatTimestamp(detection.timestamp) }}</span>
              <span class="confidence">置信度: {{ (detection.confidence * 100).toFixed(0) }}%</span>
            </div>
            <div class="detection-actions">
              <button class="action-btn" @click="viewDetectionDetails(detection)">
                <Eye />
              </button>
              <button class="action-btn danger" @click="quickBan(detection)">
                <Ban />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 检测记录 -->
    <div v-if="activeTab === 'detection'" class="tab-content">
      <div class="section-card">
        <div class="filter-bar">
          <input
            v-model="filterPlayerName"
            :placeholder="t('ai.anticheat.detection.player')"
            class="filter-input"
          />
          <select v-model="filterType" class="filter-select">
            <option value="">{{ t('ai.common.all') }}</option>
            <option v-for="type in detectionTypes" :key="type.id" :value="type.id">
              {{ type.name }}
            </option>
          </select>
          <select v-model="filterSeverity" class="filter-select">
            <option value="">{{ t('ai.common.all') }}</option>
            <option value="critical">{{ t('ai.severity.critical') }}</option>
            <option value="high">{{ t('ai.severity.error') }}</option>
            <option value="medium">{{ t('ai.severity.warning') }}</option>
            <option value="low">{{ t('ai.severity.info') }}</option>
          </select>
          <select v-model="filterStatus" class="filter-select">
            <option value="">{{ t('ai.common.all') }}</option>
            <option value="pending">待处理</option>
            <option value="processed">已处理</option>
            <option value="ignored">已忽略</option>
            <option value="false_positive">误报</option>
          </select>
          <button class="refresh-btn" @click="loadDetections">
            <RefreshCw :class="{ spinning: loading }" />
          </button>
        </div>

        <div class="detection-table">
          <div class="table-header">
            <span class="col-player">{{ t('ai.anticheat.detection.player') }}</span>
            <span class="col-type">{{ t('ai.anticheat.detection.type') }}</span>
            <span class="col-severity">{{ t('ai.anticheat.detection.severity') }}</span>
            <span class="col-confidence">{{ t('ai.anticheat.detection.confidence') }}</span>
            <span class="col-risk">风险</span>
            <span class="col-time">{{ t('ai.anticheat.detection.time') }}</span>
            <span class="col-action">{{ t('ai.anticheat.detection.action') }}</span>
          </div>
          <div
            v-for="detection in filteredDetections"
            :key="detection.id"
            class="table-row"
            :class="detection.severity"
          >
            <span class="col-player">
              <span class="player-name">{{ detection.player_name }}</span>
              <span class="player-uuid">{{ detection.player_uuid.slice(0, 8) }}...</span>
            </span>
            <span class="col-type">
              <span class="type-badge" :class="detection.category">
                {{ getDetectionTypeName(detection.detection_type) }}
              </span>
            </span>
            <span class="col-severity">
              <span class="severity-badge" :class="detection.severity">
                {{ getSeverityInfo(detection.severity).text }}
              </span>
            </span>
            <span class="col-confidence">
              <div class="confidence-bar">
                <div class="confidence-fill" :style="{ width: `${detection.confidence * 100}%` }"></div>
              </div>
              <span>{{ (detection.confidence * 100).toFixed(0) }}%</span>
            </span>
            <span class="col-risk">
              <span class="risk-score" :style="{ color: getRiskLevel(detection.risk_score).color }">
                {{ detection.risk_score }}
              </span>
            </span>
            <span class="col-time">{{ formatTimestamp(detection.timestamp) }}</span>
            <span class="col-action">
              <div class="action-buttons">
                <button class="btn-icon" title="详情" @click="viewDetectionDetails(detection)">
                  <Eye />
                </button>
                <button
                  class="btn-icon warning"
                  title="踢出"
                  @click="executeAction(detection, 'kick')"
                  v-if="detection.status === 'pending'"
                >
                  <UserX />
                </button>
                <button
                  class="btn-icon danger"
                  title="封禁"
                  @click="executeAction(detection, 'ban')"
                  v-if="detection.status === 'pending'"
                >
                  <Ban />
                </button>
                <button
                  class="btn-icon"
                  title="忽略"
                  @click="dismissDetection(detection)"
                  v-if="detection.status === 'pending'"
                >
                  <X />
                </button>
              </div>
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 检测规则 -->
    <div v-if="activeTab === 'rules'" class="tab-content">
      <div class="section-card">
        <div class="card-header">
          <h3>{{ t('ai.anticheat.rules.title') }}</h3>
          <button class="add-btn" @click="showAddRuleModal = true">
            <Plus /> {{ t('ai.anticheat.rules.add_rule') }}
          </button>
        </div>

        <div class="rules-list">
          <div
            v-for="rule in rules"
            :key="rule.id"
            class="rule-item"
            :class="{ disabled: !rule.enabled }"
          >
            <div class="rule-main">
              <div class="rule-info">
                <span class="rule-name">{{ rule.name }}</span>
                <span class="rule-type">{{ getDetectionTypeName(rule.detection_type) }}</span>
              </div>
              <div class="rule-meta">
                <span class="rule-severity" :class="rule.severity">
                  {{ getSeverityInfo(rule.severity).text }}
                </span>
                <span class="rule-threshold">阈值: {{ (rule.threshold * 100).toFixed(0) }}%</span>
              </div>
            </div>
            <p class="rule-description">{{ rule.description }}</p>
            <div class="rule-actions">
              <label class="toggle">
                <input type="checkbox" v-model="rule.enabled" @change="updateRule(rule)" />
                <span class="toggle-slider"></span>
              </label>
              <button class="btn-icon" @click="editRule(rule)">
                <Edit />
              </button>
              <button class="btn-icon danger" @click="removeRule(rule.id)">
                <Trash2 />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 自动处罚设置 -->
    <div v-if="activeTab === 'punishment'" class="tab-content">
      <div class="section-card">
        <h3>{{ t('ai.anticheat.punishment.title') }}</h3>

        <div class="punishment-settings">
          <div class="setting-row">
            <label>{{ t('ai.anticheat.punishment.enable_auto_ban') }}</label>
            <label class="toggle">
              <input type="checkbox" v-model="punishmentConfig.enabled" />
              <span class="toggle-slider"></span>
            </label>
          </div>

          <div class="setting-row">
            <label>{{ t('ai.anticheat.punishment.warning_threshold') }}</label>
            <input type="number" v-model="punishmentConfig.warning_threshold" min="0" max="100" />
          </div>

          <div class="setting-row">
            <label>{{ t('ai.anticheat.punishment.kick_threshold') }}</label>
            <input type="number" v-model="punishmentConfig.kick_threshold" min="0" max="100" />
          </div>

          <div class="setting-row">
            <label>{{ t('ai.anticheat.punishment.ban_threshold') }}</label>
            <input type="number" v-model="punishmentConfig.ban_threshold" min="0" max="100" />
          </div>

          <div class="setting-row">
            <label>{{ t('ai.anticheat.punishment.escalation') }}</label>
            <label class="toggle">
              <input type="checkbox" v-model="punishmentConfig.escalation_enabled" />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>

        <h4>{{ t('ai.anticheat.punishment.escalation') }}</h4>
        <div class="escalation-settings">
          <div class="escalation-row">
            <label>{{ t('ai.anticheat.punishment.first_offense') }}</label>
            <select v-model="punishmentConfig.first_offense.type">
              <option value="warning">警告</option>
              <option value="kick">踢出</option>
              <option value="temporary_ban">临时封禁</option>
              <option value="permanent_ban">永久封禁</option>
            </select>
            <input
              v-if="punishmentConfig.first_offense.type === 'temporary_ban'"
              type="number"
              v-model="punishmentConfig.first_offense.duration_hours"
              min="1"
              placeholder="小时"
            />
          </div>

          <div class="escalation-row">
            <label>{{ t('ai.anticheat.punishment.second_offense') }}</label>
            <select v-model="punishmentConfig.second_offense.type">
              <option value="warning">警告</option>
              <option value="kick">踢出</option>
              <option value="temporary_ban">临时封禁</option>
              <option value="permanent_ban">永久封禁</option>
            </select>
            <input
              v-if="punishmentConfig.second_offense.type === 'temporary_ban'"
              type="number"
              v-model="punishmentConfig.second_offense.duration_hours"
              min="1"
              placeholder="小时"
            />
          </div>

          <div class="escalation-row">
            <label>{{ t('ai.anticheat.punishment.third_offense') }}</label>
            <select v-model="punishmentConfig.third_offense.type">
              <option value="warning">警告</option>
              <option value="kick">踢出</option>
              <option value="temporary_ban">临时封禁</option>
              <option value="permanent_ban">永久封禁</option>
            </select>
            <input
              v-if="punishmentConfig.third_offense.type === 'temporary_ban'"
              type="number"
              v-model="punishmentConfig.third_offense.duration_hours"
              min="1"
              placeholder="小时"
            />
          </div>
        </div>

        <button class="save-btn" @click="savePunishmentConfig">
          <Save /> {{ t('ai.common.save') }}
        </button>
      </div>
    </div>

    <!-- 设置 -->
    <div v-if="activeTab === 'settings'" class="tab-content">
      <div class="section-card">
        <h3>{{ t('ai.anticheat.settings.title') }}</h3>

        <div class="settings-grid">
          <div class="setting-group">
            <h4>{{ t('ai.anticheat.settings.general') }}</h4>
            <div class="setting-row">
              <label>{{ t('ai.anticheat.settings.enable_detection') }}</label>
              <label class="toggle">
                <input type="checkbox" v-model="config.enabled" />
                <span class="toggle-slider"></span>
              </label>
            </div>
            <div class="setting-row">
              <label>{{ t('ai.anticheat.settings.log_all') }}</label>
              <label class="toggle">
                <input type="checkbox" v-model="config.logAll" />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-group">
            <h4>{{ t('ai.anticheat.settings.notification') }}</h4>
            <div class="setting-row">
              <label>{{ t('ai.anticheat.settings.notify_on_detect') }}</label>
              <label class="toggle">
                <input type="checkbox" v-model="config.notifyOnDetect" />
                <span class="toggle-slider"></span>
              </label>
            </div>
            <div class="setting-row">
              <label>{{ t('ai.anticheat.settings.notify_on_ban') }}</label>
              <label class="toggle">
                <input type="checkbox" v-model="config.notifyOnBan" />
                <span class="toggle-slider"></span>
              </label>
            </div>
            <div class="setting-row">
              <label>{{ t('ai.anticheat.settings.webhook_url') }}</label>
              <input type="text" v-model="config.webhookUrl" placeholder="https://..." />
            </div>
          </div>

          <div class="setting-group">
            <h4>{{ t('ai.anticheat.settings.detection') }}</h4>
            <div class="setting-row">
              <label>{{ t('ai.anticheat.settings.check_interval') }}</label>
              <input type="number" v-model="config.checkInterval" min="1" max="60" />
            </div>
          </div>
        </div>

        <button class="save-btn" @click="saveSettings">
          <Save /> {{ t('ai.common.save') }}
        </button>
      </div>
    </div>

    <!-- 详情弹窗 -->
    <div v-if="selectedDetection" class="modal-overlay" @click.self="selectedDetection = null">
      <div class="modal-content">
        <div class="modal-header">
          <h3>检测详情</h3>
          <button class="close-btn" @click="selectedDetection = null">
            <X />
          </button>
        </div>
        <div class="modal-body">
          <div class="detail-grid">
            <div class="detail-item">
              <label>玩家</label>
              <span>{{ selectedDetection.player_name }}</span>
            </div>
            <div class="detail-item">
              <label>UUID</label>
              <span class="uuid">{{ selectedDetection.player_uuid }}</span>
            </div>
            <div class="detail-item">
              <label>检测类型</label>
              <span>{{ getDetectionTypeName(selectedDetection.detection_type) }}</span>
            </div>
            <div class="detail-item">
              <label>分类</label>
              <span>{{ getCategoryName(selectedDetection.category) }}</span>
            </div>
            <div class="detail-item">
              <label>严重程度</label>
              <span class="severity-badge" :class="selectedDetection.severity">
                {{ getSeverityInfo(selectedDetection.severity).text }}
              </span>
            </div>
            <div class="detail-item">
              <label>置信度</label>
              <span>{{ (selectedDetection.confidence * 100).toFixed(1) }}%</span>
            </div>
            <div class="detail-item">
              <label>可疑程度</label>
              <span>{{ selectedDetection.suspicion_score }}</span>
            </div>
            <div class="detail-item">
              <label>危害程度</label>
              <span>{{ selectedDetection.danger_score }}</span>
            </div>
            <div class="detail-item">
              <label>综合风险</label>
              <span class="risk-score" :style="{ color: getRiskLevel(selectedDetection.risk_score).color }">
                {{ selectedDetection.risk_score }} ({{ getRiskLevel(selectedDetection.risk_score).level }})
              </span>
            </div>
            <div class="detail-item">
              <label>检测时间</label>
              <span>{{ formatTimestamp(selectedDetection.timestamp) }}</span>
            </div>
          </div>
          <div class="detail-section">
            <label>详情描述</label>
            <p>{{ selectedDetection.details }}</p>
          </div>
          <div v-if="selectedDetection.related_logs.length > 0" class="detail-section">
            <label>{{ t('ai.related_logs') }}</label>
            <pre v-for="(log, idx) in selectedDetection.related_logs" :key="idx">{{ log }}</pre>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn" @click="selectedDetection = null">关闭</button>
          <button
            v-if="selectedDetection.status === 'pending'"
            class="btn warning"
            @click="executeAction(selectedDetection, 'kick')"
          >
            踢出
          </button>
          <button
            v-if="selectedDetection.status === 'pending'"
            class="btn danger"
            @click="executeAction(selectedDetection, 'ban')"
          >
            封禁
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  Shield,
  Users,
  AlertTriangle,
  Ban,
  Activity,
  CheckCircle,
  Eye,
  RefreshCw,
  Plus,
  Edit,
  Trash2,
  Save,
  X,
  UserX,
} from 'lucide-vue-next';

import SLTabBar from '@components/SLTabBar.vue';

import {
  getDetections,
  getDetectionRules,
  saveDetectionRule,
  deleteDetectionRule,
  getPunishmentConfig,
  updatePunishmentConfig,
  getAntiCheatStatistics,
  getDetectionTypes,
  executePunishment,
  getSeverityInfo,
  getDetectionTypeName,
  getCategoryName,
  formatTimestamp,
  getRiskLevel,
  type DetectionRecord,
  type DetectionRule,
  type PunishmentConfig,
  type AntiCheatStatistics,
  type DetectionTypeInfo,
  type PunishmentAction,
} from '@api/anticheat';

const { t } = useI18n();

// Tab 管理
const activeTab = ref('dashboard');
const tabs = [
  { id: 'dashboard', label: computed(() => t('ai.anticheat.tabs.dashboard')) },
  { id: 'detection', label: computed(() => t('ai.anticheat.tabs.detection')) },
  { id: 'rules', label: computed(() => t('ai.anticheat.tabs.rules')) },
  { id: 'punishment', label: computed(() => t('ai.anticheat.tabs.punishment')) },
  { id: 'settings', label: computed(() => t('ai.anticheat.tabs.settings')) },
];

// 状态
const loading = ref(false);
const onlinePlayers = ref(0);
const activeThreats = ref(0);

// 数据
const detections = ref<DetectionRecord[]>([]);
const rules = ref<DetectionRule[]>([]);
const stats = ref<AntiCheatStatistics>({
  detections_today: 0,
  bans_today: 0,
  warnings_today: 0,
  kicks_today: 0,
  total_detections: 0,
  total_bans: 0,
  by_type: {},
  by_severity: {},
  stats_date: '',
});
const detectionTypes = ref<DetectionTypeInfo[]>([]);
const punishmentConfig = ref<PunishmentConfig>({
  enabled: true,
  warning_threshold: 30,
  kick_threshold: 50,
  ban_threshold: 70,
  first_offense: { type: 'warning' },
  second_offense: { type: 'kick' },
  third_offense: { type: 'temporary_ban', duration_hours: 24 },
  escalation_enabled: true,
});

// 配置
const config = ref({
  enabled: true,
  logAll: true,
  notifyOnDetect: true,
  notifyOnBan: true,
  webhookUrl: '',
  checkInterval: 5,
});

// 筛选
const filterPlayerName = ref('');
const filterType = ref('');
const filterSeverity = ref('');
const filterStatus = ref('');

// 选中的检测
const selectedDetection = ref<DetectionRecord | null>(null);
const showAddRuleModal = ref(false);

// 计算属性
const recentDetections = computed(() => {
  return detections.value.slice(0, 10);
});

const filteredDetections = computed(() => {
  return detections.value.filter(d => {
    if (filterPlayerName.value && !d.player_name.toLowerCase().includes(filterPlayerName.value.toLowerCase())) {
      return false;
    }
    if (filterType.value && d.detection_type !== filterType.value) {
      return false;
    }
    if (filterSeverity.value && d.severity !== filterSeverity.value) {
      return false;
    }
    if (filterStatus.value && d.status !== filterStatus.value) {
      return false;
    }
    return true;
  });
});

// 方法
async function loadDetections() {
  loading.value = true;
  try {
    detections.value = await getDetections();
  } finally {
    loading.value = false;
  }
}

async function loadRules() {
  rules.value = await getDetectionRules();
}

async function loadStats() {
  stats.value = await getAntiCheatStatistics();
}

async function loadPunishmentConfig() {
  punishmentConfig.value = await getPunishmentConfig();
}

async function loadDetectionTypes() {
  detectionTypes.value = await getDetectionTypes();
}

function viewDetectionDetails(detection: DetectionRecord) {
  selectedDetection.value = detection;
}

async function executeAction(detection: DetectionRecord, actionType: 'kick' | 'ban') {
  const action: PunishmentAction = actionType === 'kick'
    ? { type: 'kick' }
    : { type: 'permanent_ban' };

  if (confirm(`确定要对玩家 ${detection.player_name} 执行${actionType === 'kick' ? '踢出' : '封禁'}操作吗？`)) {
    await executePunishment(detection.id, action);
    detection.status = 'processed';
    detection.action_taken = action;
    await loadDetections();
  }
}

async function quickBan(detection: DetectionRecord) {
  await executeAction(detection, 'ban');
}

async function dismissDetection(detection: DetectionRecord) {
  detection.status = 'ignored';
}

async function updateRule(rule: DetectionRule) {
  await saveDetectionRule(rule);
}

async function removeRule(ruleId: string) {
  if (confirm('确定要删除此规则吗？')) {
    await deleteDetectionRule(ruleId);
    await loadRules();
  }
}

function editRule(rule: DetectionRule) {
  // 打开编辑模态框
  console.log('Edit rule:', rule);
}

async function savePunishmentConfig() {
  await updatePunishmentConfig(punishmentConfig.value);
  alert('设置已保存');
}

function saveSettings() {
  // 保存设置
  alert('设置已保存');
}

// 初始化
onMounted(async () => {
  await Promise.all([
    loadDetections(),
    loadRules(),
    loadStats(),
    loadPunishmentConfig(),
    loadDetectionTypes(),
  ]);
});
</script>

<style scoped>
.anticheat-view {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}

.anticheat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.anticheat-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.anticheat-icon {
  width: 32px;
  height: 32px;
  color: var(--sl-primary);
}

.anticheat-title h2 {
  margin: 0;
  font-size: 18px;
}

.subtitle {
  margin: 4px 0 0 0;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.anticheat-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--sl-error);
}

.anticheat-status.enabled .status-dot {
  background: var(--sl-success);
}

.tab-content {
  margin-top: 20px;
}

.section-card {
  background: var(--sl-card-bg);
  border-radius: 8px;
  padding: 20px;
}

.section-card h3 {
  margin: 0 0 15px 0;
  font-size: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 15px;
  margin-bottom: 20px;
}

.stat-card {
  background: var(--sl-card-bg);
  border-radius: 8px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-icon {
  width: 40px;
  height: 40px;
  color: var(--sl-primary);
}

.stat-icon.warning { color: #faad14; }
.stat-icon.danger { color: #ff4d4f; }
.stat-icon.info { color: #1890ff; }

.stat-info {
  display: flex;
  flex-direction: column;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
}

.stat-label {
  font-size: 12px;
  color: var(--sl-text-secondary);
}

/* 检测列表 */
.detection-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.detection-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: var(--sl-bg-secondary);
  border-radius: 6px;
  border-left: 3px solid var(--sl-border);
}

.detection-item.critical { border-left-color: #ff4d4f; }
.detection-item.high { border-left-color: #ff7a45; }
.detection-item.medium { border-left-color: #faad14; }
.detection-item.low { border-left-color: #52c41a; }

.detection-main {
  display: flex;
  align-items: center;
  gap: 12px;
}

.player-name {
  font-weight: 500;
}

.detection-type {
  color: var(--sl-text-secondary);
  font-size: 13px;
}

.risk-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  color: white;
}

.detection-meta {
  display: flex;
  gap: 15px;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.detection-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px;
  border: none;
  border-radius: 4px;
  background: var(--sl-bg);
  color: var(--sl-text-secondary);
  cursor: pointer;
}

.action-btn:hover {
  background: var(--sl-bg-secondary);
}

.action-btn.danger:hover {
  background: #ff4d4f20;
  color: #ff4d4f;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 40px;
  color: var(--sl-text-secondary);
}

.empty-icon {
  width: 48px;
  height: 48px;
  color: var(--sl-success);
  margin-bottom: 12px;
}

/* 筛选栏 */
.filter-bar {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.filter-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
}

.filter-select {
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
}

.refresh-btn {
  padding: 8px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: transparent;
  color: var(--sl-text);
  cursor: pointer;
}

.spinning {
  animation: spin 1s linear infinite;
}

/* 检测表格 */
.detection-table {
  width: 100%;
}

.table-header {
  display: grid;
  grid-template-columns: 2fr 1.5fr 1fr 1.5fr 0.8fr 1.5fr 1fr;
  gap: 10px;
  padding: 10px;
  background: var(--sl-bg-secondary);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--sl-text-secondary);
}

.table-row {
  display: grid;
  grid-template-columns: 2fr 1.5fr 1fr 1.5fr 0.8fr 1.5fr 1fr;
  gap: 10px;
  padding: 10px;
  border-bottom: 1px solid var(--sl-border);
  font-size: 13px;
  align-items: center;
}

.player-uuid {
  font-size: 11px;
  color: var(--sl-text-secondary);
}

.type-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
}

.type-badge.hack { background: #ff4d4f20; color: #ff4d4f; }
.type-badge.mod { background: #faad1420; color: #faad14; }
.type-badge.exploit { background: #ff7a4520; color: #ff7a45; }
.type-badge.behavior { background: #1890ff20; color: #1890ff; }

.severity-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
}

.severity-badge.critical { background: #ff4d4f20; color: #ff4d4f; }
.severity-badge.high { background: #ff7a4520; color: #ff7a45; }
.severity-badge.medium { background: #faad1420; color: #faad14; }
.severity-badge.low { background: #52c41a20; color: #52c41a; }

.confidence-bar {
  width: 60px;
  height: 4px;
  background: var(--sl-border);
  border-radius: 2px;
  margin-right: 8px;
}

.confidence-fill {
  height: 100%;
  background: var(--sl-primary);
  border-radius: 2px;
}

.col-confidence {
  display: flex;
  align-items: center;
}

.risk-score {
  font-weight: 600;
}

.action-buttons {
  display: flex;
  gap: 4px;
}

.btn-icon {
  padding: 4px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
}

.btn-icon:hover {
  background: var(--sl-bg-secondary);
}

.btn-icon.warning:hover { color: #faad14; }
.btn-icon.danger:hover { color: #ff4d4f; }

/* 规则列表 */
.rules-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.rule-item {
  padding: 15px;
  background: var(--sl-bg-secondary);
  border-radius: 6px;
}

.rule-item.disabled {
  opacity: 0.5;
}

.rule-main {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.rule-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.rule-name {
  font-weight: 500;
}

.rule-type {
  padding: 2px 8px;
  background: var(--sl-bg);
  border-radius: 4px;
  font-size: 11px;
}

.rule-meta {
  display: flex;
  gap: 15px;
  font-size: 12px;
}

.rule-severity {
  padding: 2px 8px;
  border-radius: 4px;
}

.rule-severity.critical { background: #ff4d4f20; color: #ff4d4f; }
.rule-severity.high { background: #ff7a4520; color: #ff7a45; }
.rule-severity.medium { background: #faad1420; color: #faad14; }
.rule-severity.low { background: #52c41a20; color: #52c41a; }

.rule-description {
  margin: 0 0 10px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
}

.rule-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  background: var(--sl-primary);
  color: white;
  cursor: pointer;
}

/* 开关 */
.toggle {
  position: relative;
  width: 44px;
  height: 22px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--sl-border);
  transition: 0.3s;
  border-radius: 22px;
}

.toggle-slider::before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.toggle input:checked + .toggle-slider {
  background-color: var(--sl-primary);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(22px);
}

/* 处罚设置 */
.punishment-settings {
  display: flex;
  flex-direction: column;
  gap: 15px;
  margin-bottom: 20px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.setting-row label {
  font-size: 14px;
}

.setting-row input[type="number"] {
  width: 80px;
  padding: 8px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
}

.escalation-settings {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.escalation-row {
  display: flex;
  align-items: center;
  gap: 15px;
}

.escalation-row label {
  width: 100px;
  font-size: 14px;
}

.escalation-row select {
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
}

.escalation-row input {
  width: 80px;
  padding: 8px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
}

.save-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 12px;
  margin-top: 20px;
  border: none;
  border-radius: 6px;
  background: var(--sl-primary);
  color: white;
  cursor: pointer;
}

/* 设置 */
.settings-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 30px;
}

.setting-group h4 {
  margin: 0 0 15px 0;
  font-size: 14px;
  color: var(--sl-text-secondary);
}

.setting-group .setting-row {
  margin-bottom: 12px;
}

.setting-group input[type="text"] {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
}

/* 模态框 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--sl-card-bg);
  border-radius: 8px;
  width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  border-bottom: 1px solid var(--sl-border);
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
}

.close-btn {
  padding: 4px;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
}

.modal-body {
  padding: 20px;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 15px;
  margin-bottom: 20px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-item label {
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.uuid {
  font-family: monospace;
  font-size: 11px;
}

.detail-section {
  margin-bottom: 15px;
}

.detail-section label {
  display: block;
  font-size: 12px;
  color: var(--sl-text-secondary);
  margin-bottom: 8px;
}

.detail-section p {
  margin: 0;
  font-size: 14px;
}

.detail-section pre {
  margin: 5px 0;
  padding: 10px;
  background: var(--sl-bg);
  border-radius: 4px;
  font-size: 11px;
  overflow-x: auto;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 15px 20px;
  border-top: 1px solid var(--sl-border);
}

.btn {
  padding: 8px 16px;
  border: 1px solid var(--sl-border);
  border-radius: 4px;
  background: transparent;
  color: var(--sl-text);
  cursor: pointer;
}

.btn.warning {
  background: #faad14;
  border-color: #faad14;
  color: white;
}

.btn.danger {
  background: #ff4d4f;
  border-color: #ff4d4f;
  color: white;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
