<script setup lang="ts">
/**
 * 前端基础设施测试面板
 * 覆盖 i18n / 工具函数 / 错误处理 / store 等纯前端能力
 */
import { ref, computed } from "vue";
import { i18n } from "@language";
import { useToast } from "cmzya-modern-ui";
import {
  handleError,
  formatError,
  AppError,
  ErrorType,
  validatePlayerName,
  safeJsonParse,
  retry,
} from "@utils/errorHandler";
import { useServerStore } from "@stores/serverStore";
import { useSettingsStore } from "@stores/settingsStore";

const toast = useToast();
const serverStore = useServerStore();
const settingsStore = useSettingsStore();

// i18n 测试
const i18nKey = ref("common.home");
const i18nResult = computed(() => {
  try {
    return i18n.t(i18nKey.value);
  } catch {
    return "(无效 key)";
  }
});
const currentLocale = computed(() => i18n.getLocale());

// 错误处理测试
const errorInput = ref("测试错误");
function throwPlainError(): never {
  throw new Error(errorInput.value || "普通错误");
}
function throwAppError(): never {
  throw new AppError(errorInput.value || "应用错误", ErrorType.SERVER, "测试上下文");
}
function throwString(): never {
  throw errorInput.value || "字符串错误";
}
function runErrorTest(fn: () => never) {
  try {
    fn();
  } catch (e) {
    const msg = handleError(e, "DevTest");
    toast.error(msg);
  }
}

// 玩家名校验测试
const playerName = ref("Steve");
const playerValidation = computed(() => validatePlayerName(playerName.value));

// JSON 解析测试
const jsonInput = ref('{"a":1,"b":"x"}');
const jsonResult = computed(() => safeJsonParse(jsonInput.value, null));

// retry 测试
const retryCount = ref(3);
const retryResult = ref<string>("");
const retryRunning = ref(false);
async function runRetryTest() {
  retryRunning.value = true;
  retryResult.value = "";
  let attempt = 0;
  try {
    const result = await retry(
      async () => {
        attempt++;
        if (attempt < retryCount.value) {
          throw new Error(`第 ${attempt} 次失败`);
        }
        return `第 ${attempt} 次成功`;
      },
      retryCount.value,
      300,
    );
    retryResult.value = result;
    toast.success(`重试成功: ${result}`);
  } catch (e) {
    retryResult.value = handleError(e, "RetryTest");
    toast.error(retryResult.value);
  } finally {
    retryRunning.value = false;
  }
}

// store 状态快照
const storeSnapshot = computed(() => ({
  server: {
    count: serverStore.servers.length,
    currentId: serverStore.currentServerId,
    loading: serverStore.loading,
  },
  settings: {
    loaded: settingsStore.isLoaded,
    theme: settingsStore.theme,
    fontSize: settingsStore.fontSize,
    developerMode: settingsStore.settings.developer_mode,
  },
}));
</script>

<template>
  <div class="dev-panel">
    <!-- i18n -->
    <cmz-card title="i18n 国际化" padding="md">
      <div class="test-row">
        <span class="test-label">当前语言</span>
        <code>{{ currentLocale }}</code>
      </div>
      <div class="test-row">
        <span class="test-label">测试 key</span>
        <cmz-input v-model="i18nKey" placeholder="common.home" />
      </div>
      <div class="test-row">
        <span class="test-label">翻译结果</span>
        <code class="test-result">{{ i18nResult }}</code>
      </div>
    </cmz-card>

    <!-- 错误处理 -->
    <cmz-card title="错误处理" padding="md">
      <div class="test-row">
        <span class="test-label">错误消息</span>
        <cmz-input v-model="errorInput" />
      </div>
      <div class="test-actions">
        <cmz-button size="sm" @click="runErrorTest(throwPlainError)">抛 Error</cmz-button>
        <cmz-button size="sm" @click="runErrorTest(throwAppError)">抛 AppError</cmz-button>
        <cmz-button size="sm" @click="runErrorTest(throwString)">抛字符串</cmz-button>
      </div>
      <div class="test-row">
        <span class="test-label">formatError(普通 Error)</span>
        <code class="test-result">{{ formatError(new Error("示例")) }}</code>
      </div>
    </cmz-card>

    <!-- 玩家名校验 -->
    <cmz-card title="玩家名校验" padding="md">
      <div class="test-row">
        <span class="test-label">玩家名</span>
        <cmz-input v-model="playerName" />
      </div>
      <div class="test-row">
        <span class="test-label">校验结果</span>
        <code :class="{ 'test-error': !playerValidation.valid }">
          {{ playerValidation.valid ? "通过" : playerValidation.error }}
        </code>
      </div>
    </cmz-card>

    <!-- JSON 解析 -->
    <cmz-card title="JSON 解析" padding="md">
      <div class="test-row">
        <span class="test-label">输入</span>
        <cmz-input v-model="jsonInput" />
      </div>
      <div class="test-row">
        <span class="test-label">解析结果</span>
        <code class="test-result">{{ jsonResult }}</code>
      </div>
    </cmz-card>

    <!-- retry -->
    <cmz-card title="retry 重试" padding="md">
      <div class="test-row">
        <span class="test-label">重试次数</span>
        <cmz-input v-model.number="retryCount" type="number" />
      </div>
      <div class="test-actions">
        <cmz-button size="sm" :loading="retryRunning" @click="runRetryTest">执行重试</cmz-button>
      </div>
      <div v-if="retryResult" class="test-row">
        <span class="test-label">结果</span>
        <code class="test-result">{{ retryResult }}</code>
      </div>
    </cmz-card>

    <!-- store 快照 -->
    <cmz-card title="Store 状态快照" padding="md">
      <pre class="test-snapshot">{{ JSON.stringify(storeSnapshot, null, 2) }}</pre>
    </cmz-card>
  </div>
</template>

<style scoped>
.dev-panel {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
.test-row {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  margin-bottom: var(--sl-space-sm);
}
.test-label {
  min-width: 140px;
  color: var(--sl-text-secondary);
  font-size: 13px;
}
.test-result {
  color: var(--sl-primary);
  word-break: break-all;
}
.test-error {
  color: #f87171;
}
.test-actions {
  display: flex;
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-sm);
}
.test-snapshot {
  margin: 0;
  padding: var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-sm);
  font-size: 12px;
  line-height: 1.6;
  overflow-x: auto;
}
</style>
