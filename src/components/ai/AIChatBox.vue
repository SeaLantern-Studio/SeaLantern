<template>
  <div class="ai-chat-box">
    <!-- 消息列表 -->
    <div class="message-list" ref="messageListRef">
      <div
        v-for="(message, index) in messages"
        :key="index"
        class="message"
        :class="message.role"
      >
        <div class="message-avatar">
          <BotIcon v-if="message.role === 'assistant'" />
          <UserIcon v-else />
        </div>
        <div class="message-content">
          <div class="message-header">
            <span class="message-role">{{ message.role === 'assistant' ? 'AI' : '你' }}</span>
            <span class="message-time">{{ formatTime(message.timestamp) }}</span>
          </div>
          <div class="message-text" v-html="formatMessage(message.content)"></div>
        </div>
      </div>
      
      <!-- 加载中 -->
      <div v-if="loading" class="message assistant">
        <div class="message-avatar">
          <BotIcon />
        </div>
        <div class="message-content">
          <div class="loading-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="input-area">
      <textarea
        v-model="inputText"
        :placeholder="placeholder"
        :disabled="loading"
        @keydown.enter.exact.prevent="sendMessage"
        @keydown.enter.shift.exact="inputText += '\n'"
        rows="1"
        ref="inputRef"
      ></textarea>
      <button
        class="send-btn"
        :disabled="loading || !inputText.trim()"
        @click="sendMessage"
      >
        <SendIcon />
      </button>
    </div>

    <!-- 快捷操作 -->
    <div v-if="quickActions.length > 0" class="quick-actions">
      <button
        v-for="action in quickActions"
        :key="action.id"
        class="quick-action-btn"
        @click="handleQuickAction(action)"
      >
        {{ action.label }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue';
import { Bot as BotIcon, User as UserIcon, Send as SendIcon } from 'lucide-vue-next';

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

export interface QuickAction {
  id: string;
  label: string;
  prompt: string;
}

const props = withDefaults(
  defineProps<{
    messages: ChatMessage[];
    loading?: boolean;
    placeholder?: string;
    quickActions?: QuickAction[];
  }>(),
  {
    loading: false,
    placeholder: '输入消息...',
    quickActions: () => [],
  }
);

const emit = defineEmits<{
  (e: 'send', message: string): void;
  (e: 'quickAction', action: QuickAction): void;
}>();

const inputText = ref('');
const messageListRef = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLTextAreaElement | null>(null);

// 发送消息
function sendMessage() {
  if (!inputText.value.trim() || props.loading) return;
  
  emit('send', inputText.value.trim());
  inputText.value = '';
  
  // 自动调整高度
  if (inputRef.value) {
    inputRef.value.style.height = 'auto';
  }
}

// 处理快捷操作
function handleQuickAction(action: QuickAction) {
  emit('quickAction', action);
  emit('send', action.prompt);
}

// 格式化时间
function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
}

// XSS 过滤：转义 HTML 特殊字符
function escapeHtml(text: string): string {
  const map: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;',
  };
  return text.replace(/[&<>"']/g, (char) => map[char]);
}

// 格式化消息 (带 XSS 过滤的 Markdown 支持)
function formatMessage(content: string): string {
  // 先转义 HTML 防止 XSS
  let processed = escapeHtml(content);
  
  // 然后应用 Markdown 格式化
  return processed
    .replace(/```(\w*)\n?([\s\S]*?)```/g, '<pre><code class="language-$1">$2</code></pre>')
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    .replace(/\n/g, '<br>');
}

// 自动滚动到底部
watch(
  () => props.messages.length,
  () => {
    nextTick(() => {
      if (messageListRef.value) {
        messageListRef.value.scrollTop = messageListRef.value.scrollHeight;
      }
    });
  }
);

// 自动调整输入框高度
watch(inputText, () => {
  if (inputRef.value) {
    inputRef.value.style.height = 'auto';
    inputRef.value.style.height = Math.min(inputRef.value.scrollHeight, 150) + 'px';
  }
});
</script>

<style scoped>
.ai-chat-box {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--sl-card-bg);
  border-radius: 8px;
  overflow: hidden;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.message {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.message.user {
  flex-direction: row-reverse;
}

.message-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.message.assistant .message-avatar {
  background: var(--sl-primary);
  color: white;
}

.message.user .message-avatar {
  background: var(--sl-bg-secondary);
  color: var(--sl-text);
}

.message-content {
  max-width: 80%;
  padding: 12px 16px;
  border-radius: 12px;
}

.message.assistant .message-content {
  background: var(--sl-bg-secondary);
  border-bottom-left-radius: 4px;
}

.message.user .message-content {
  background: var(--sl-primary);
  color: white;
  border-bottom-right-radius: 4px;
}

.message-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
  gap: 8px;
}

.message-role {
  font-size: 12px;
  font-weight: 500;
}

.message-time {
  font-size: 11px;
  opacity: 0.7;
}

.message-text {
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
}

.message-text :deep(pre) {
  background: var(--sl-bg);
  padding: 8px 12px;
  border-radius: 4px;
  overflow-x: auto;
  margin: 8px 0;
}

.message-text :deep(code) {
  font-family: monospace;
  font-size: 13px;
}

.message-text :deep(code:not(pre code)) {
  background: var(--sl-bg);
  padding: 2px 6px;
  border-radius: 3px;
}

.loading-dots {
  display: flex;
  gap: 4px;
  padding: 4px 0;
}

.loading-dots span {
  width: 8px;
  height: 8px;
  background: var(--sl-text-secondary);
  border-radius: 50%;
  animation: bounce 1.4s infinite ease-in-out both;
}

.loading-dots span:nth-child(1) { animation-delay: -0.32s; }
.loading-dots span:nth-child(2) { animation-delay: -0.16s; }

@keyframes bounce {
  0%, 80%, 100% { transform: scale(0); }
  40% { transform: scale(1); }
}

.input-area {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--sl-border);
  background: var(--sl-bg);
}

.input-area textarea {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid var(--sl-border);
  border-radius: 8px;
  background: var(--sl-input-bg);
  color: var(--sl-text);
  font-size: 14px;
  resize: none;
  min-height: 40px;
  max-height: 150px;
  line-height: 1.4;
}

.input-area textarea:focus {
  outline: none;
  border-color: var(--sl-primary);
}

.input-area textarea:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.send-btn {
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 8px;
  background: var(--sl-primary);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.send-btn:hover:not(:disabled) {
  background: var(--sl-primary-dark);
}

.send-btn:disabled {
  background: var(--sl-disabled);
  cursor: not-allowed;
}

.quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px 16px 12px;
  border-top: 1px solid var(--sl-border);
}

.quick-action-btn {
  padding: 6px 12px;
  border: 1px solid var(--sl-border);
  border-radius: 16px;
  background: transparent;
  color: var(--sl-text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.quick-action-btn:hover {
  background: var(--sl-bg-secondary);
  color: var(--sl-text);
  border-color: var(--sl-primary);
}
</style>
