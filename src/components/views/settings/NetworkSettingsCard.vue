<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { systemApi, type IPv6TestResult } from "@api/system";
import { settingsApi, type ProxySettings } from "@api/settings";
import { dispatchSettingsUpdate } from "@stores/settingsStore";
import { i18n } from "@language";
import { handleError } from "@utils/errorHandler";
import { useToast } from "cmzya-modern-ui";

// 代理模式枚举，与后端 ProxySettings 联合类型对应
type ProxyMode = "adaptive" | "preserve" | "manual" | "disabled";

const props = defineProps<{
  proxy: ProxySettings;
}>();

const toast = useToast();

const testing = ref(false);
const showDetail = ref(false);
const result = ref<IPv6TestResult | null>(null);

// 本地编辑态，与 prop 同步
const proxyMode = ref<ProxyMode>(props.proxy.mode);
const proxyUrl = ref(props.proxy.mode === "manual" ? props.proxy.proxy_url : "");
const applying = ref(false);

// prop 更新时同步本地，避免外部重置后 UI 残留旧值
watch(
  () => props.proxy,
  (p) => {
    proxyMode.value = p.mode;
    proxyUrl.value = p.mode === "manual" ? p.proxy_url : "";
  },
  { deep: true },
);

const proxyModeOptions = computed(() => [
  { label: i18n.t("settings.proxy_mode_adaptive"), value: "adaptive" },
  { label: i18n.t("settings.proxy_mode_preserve"), value: "preserve" },
  { label: i18n.t("settings.proxy_mode_manual"), value: "manual" },
  { label: i18n.t("settings.proxy_mode_disabled"), value: "disabled" },
]);

async function testIPv6() {
  testing.value = true;
  result.value = null;
  showDetail.value = false;
  try {
    result.value = await systemApi.testIPv6Connectivity();
  } catch (e) {
    result.value = { supported: false, message: String(e) };
  } finally {
    testing.value = false;
  }
}

// 走局部更新接口应用代理，同时广播事件让其它视图同步
async function applyProxy() {
  if (applying.value) return;
  applying.value = true;
  try {
    const next: ProxySettings =
      proxyMode.value === "manual"
        ? { mode: "manual", proxy_url: proxyUrl.value.trim() }
        : { mode: proxyMode.value };
    const res = await settingsApi.updatePartial({ proxy: next });
    // 广播变更，SettingsView 监听后会刷新本地 settings
    dispatchSettingsUpdate(res.changed_groups, res.settings);
    toast.success(i18n.t("settings.proxy_applied"));
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    applying.value = false;
  }
}
</script>

<template>
  <cmz-card
    variant="glass"
    :title="i18n.t('settings.network')"
    :subtitle="i18n.t('settings.network_desc')"
  >
    <div class="sl-settings-group">
      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.ipv6_test") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.ipv6_test_desc") }}</span>
        </div>
        <cmz-button size="sm" :loading="testing" @click="testIPv6">
          {{ testing ? i18n.t("settings.ipv6_testing") : i18n.t("settings.ipv6_test_btn") }}
        </cmz-button>
      </div>

      <div v-if="result" class="test-result" :class="result.supported ? 'success' : 'error'">
        <span class="result-icon">{{ result.supported ? "✓" : "✗" }}</span>
        <div class="result-body">
          <span class="result-text">{{
            result.supported
              ? i18n.t("settings.ipv6_supported")
              : i18n.t("settings.ipv6_not_supported")
          }}</span>
          <span v-if="result.message" class="result-message">{{ result.message }}</span>
          <button
            v-if="!result.supported && (result.detail || result.targets)"
            class="detail-toggle"
            @click="showDetail = !showDetail"
          >
            {{
              showDetail ? i18n.t("settings.ipv6_detail_hide") : i18n.t("settings.ipv6_detail_show")
            }}
          </button>
          <div v-if="showDetail && !result.supported" class="result-detail-panel">
            <div v-if="result.error_kind" class="detail-row">
              <span class="detail-label">{{ i18n.t("settings.ipv6_error_kind_label") }}</span>
              <span class="detail-value">{{ result.error_kind }}</span>
            </div>
            <div v-if="result.detail" class="detail-row">
              <span class="detail-label">{{ i18n.t("settings.ipv6_raw_error_label") }}</span>
              <span class="detail-value">{{ result.detail }}</span>
            </div>
            <div v-if="result.targets && result.targets.length" class="detail-targets">
              <div class="detail-targets-title">
                {{ i18n.t("settings.ipv6_test_targets_label") }}
              </div>
              <div v-for="t in result.targets" :key="t.address" class="detail-target">
                <span class="target-name">{{ t.target }}</span>
                <span class="target-addr">{{ t.address }}</span>
                <span class="target-error">{{ t.error }} ({{ t.kind }})</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 代理设置：模式选择 + 自定义地址输入 + 应用按钮 -->
      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.proxy") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.proxy_desc") }}</span>
        </div>
        <cmz-select
          :model-value="proxyMode"
          :options="proxyModeOptions"
          @update:model-value="(v: string | number) => (proxyMode = v as ProxyMode)"
        />
      </div>

      <div v-if="proxyMode === 'manual'" class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.proxy_url") }}</span>
          <span class="settings-entry-desc">{{ i18n.t("settings.proxy_url_desc") }}</span>
        </div>
        <cmz-input
          :model-value="proxyUrl"
          :placeholder="i18n.t('settings.proxy_url_placeholder')"
          class="proxy-url-input"
          @update:model-value="(v: string) => (proxyUrl = v)"
        />
      </div>

      <div class="settings-entry">
        <div class="settings-entry-info">
          <span class="settings-entry-title">{{ i18n.t("settings.proxy_mode_" + proxyMode) }}</span>
          <span class="settings-entry-desc">{{
            i18n.t("settings.proxy_mode_" + proxyMode + "_desc")
          }}</span>
        </div>
        <cmz-button size="sm" :disabled="applying" @click="applyProxy">
          {{ applying ? i18n.t("settings.proxy_applying") : i18n.t("settings.proxy_apply") }}
        </cmz-button>
      </div>
    </div>
  </cmz-card>
</template>

<style scoped>
.test-result {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-md);
  padding: var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  font-size: var(--sl-font-size-sm);
}

.test-result.success {
  background: var(--sl-success-bg);
  border: 1px solid var(--sl-success);
  color: var(--sl-success);
}

.test-result.error {
  background: var(--sl-error-bg);
  border: 1px solid var(--sl-error);
  color: var(--sl-error);
}

.result-icon {
  font-size: var(--sl-font-size-base);
  font-weight: 600;
  flex-shrink: 0;
}

.result-body {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
  flex: 1;
  min-width: 0;
}

.result-text {
  font-weight: 500;
}

.result-message {
  display: block;
  font-size: var(--sl-font-size-sm);
  opacity: 0.9;
  word-break: break-all;
}

.detail-toggle {
  align-self: flex-start;
  padding: 2px 8px;
  font-size: var(--sl-font-size-xs);
  color: inherit;
  background: none;
  border: 1px solid currentColor;
  border-radius: var(--sl-radius-sm);
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.detail-toggle:hover {
  opacity: 1;
}

.result-detail-panel {
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  background: rgba(0, 0, 0, 0.08);
  border-radius: var(--sl-radius-sm);
  font-size: var(--sl-font-size-xs);
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.detail-row {
  display: flex;
  gap: var(--sl-space-sm);
  flex-wrap: wrap;
}

.detail-label {
  font-weight: 600;
  white-space: nowrap;
}

.detail-value {
  word-break: break-all;
  opacity: 0.9;
}

.detail-targets-title {
  font-weight: 600;
  margin-top: var(--sl-space-xs);
}

.detail-target {
  display: flex;
  flex-direction: column;
  padding: var(--sl-space-xs);
  margin-top: 2px;
  background: rgba(0, 0, 0, 0.05);
  border-radius: var(--sl-radius-xs);
  gap: 1px;
}

.target-name {
  font-weight: 500;
}

.target-addr {
  opacity: 0.7;
}

.target-error {
  color: inherit;
  opacity: 0.85;
}

/* 自定义代理地址输入框宽度约束，和 settings-entry 内的 select 保持一致 */
.proxy-url-input {
  width: 100%;
  min-width: 200px;
  max-width: 320px;
  flex-shrink: 0;
  margin-left: auto;
}
</style>
