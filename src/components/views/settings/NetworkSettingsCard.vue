<script setup lang="ts">
import { ref } from "vue";
import { systemApi, type IPv6TestResult } from "@api/system";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";

const testing = ref(false);
const showDetail = ref(false);
const result = ref<IPv6TestResult | null>(null);

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
</script>

<template>
  <div class="settings-card">
    <div class="card-header">
      <h3>{{ i18n.t("settings.network") }}</h3>
      <p class="card-desc">{{ i18n.t("settings.network_desc") }}</p>
    </div>

    <div class="card-content">
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-label">{{ i18n.t("settings.ipv6_test") }}</span>
          <span class="setting-desc">{{ i18n.t("settings.ipv6_test_desc") }}</span>
        </div>
        <div class="setting-control">
          <SLButton variant="primary" size="sm" :loading="testing" @click="testIPv6">
            {{ testing ? i18n.t("settings.ipv6_testing") : i18n.t("settings.ipv6_test_btn") }}
          </SLButton>
        </div>
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
    </div>
  </div>
</template>

<style scoped>
.settings-card {
  background: var(--sl-card-bg);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-lg);
  overflow: hidden;
}

.card-header {
  padding: var(--sl-space-lg);
  border-bottom: 1px solid var(--sl-border);
}

.card-header h3 {
  margin: 0 0 var(--sl-space-xs) 0;
  font-size: var(--sl-font-size-lg);
  font-weight: 600;
  color: var(--sl-text);
}

.card-desc {
  margin: 0;
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
}

.card-content {
  padding: var(--sl-space-lg);
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-lg);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.setting-label {
  font-size: var(--sl-font-size-base);
  font-weight: 500;
  color: var(--sl-text);
}

.setting-desc {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
}

.setting-control {
  flex-shrink: 0;
}

.test-result {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-lg);
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
</style>
