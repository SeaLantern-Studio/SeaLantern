<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { Dialog, DialogPanel, DialogTitle, Switch } from "@headlessui/vue";
import { CalendarClock, Plus, Play, Trash2, Pencil } from "lucide-vue-next";
import { useToast } from "@composables/useToast";
import { schedulerApi, type ScheduledTask, type SchedulerTaskType } from "@api/scheduler";
import SLButton from "@components/common/SLButton.vue";

const toast = useToast();
const tasks = ref<ScheduledTask[]>([]);
const isDialogOpen = ref(false);
const isEditing = ref(false);
const form = ref({
  id: "",
  name: "",
  taskType: "Restart" as SchedulerTaskType,
  cronExpression: "0 4 * * *",
  command: "",
});

const taskTypeOptions: Array<{ value: SchedulerTaskType; label: string }> = [
  { value: "Restart", label: "重启服务器" },
  { value: "Backup", label: "备份世界" },
  { value: "Command", label: "执行命令" },
];

const cronPresets = [
  { label: "每天凌晨 4 点", value: "0 4 * * *" },
  { label: "每小时", value: "0 * * * *" },
  { label: "每天中午 12 点", value: "0 12 * * *" },
  { label: "每 30 分钟", value: "*/30 * * * *" },
];

const showCommandField = computed(() => form.value.taskType === "Command");

async function loadTasks() {
  try {
    tasks.value = await schedulerApi.getAllTasks();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : "加载任务失败");
  }
}

function openCreateDialog() {
  isEditing.value = false;
  form.value = {
    id: "",
    name: "",
    taskType: "Restart",
    cronExpression: "0 4 * * *",
    command: "",
  };
  isDialogOpen.value = true;
}

function openEditDialog(task: ScheduledTask) {
  isEditing.value = true;
  form.value = {
    id: task.id,
    name: task.name,
    taskType: task.task_type,
    cronExpression: task.cron_expression,
    command: task.command ?? "",
  };
  isDialogOpen.value = true;
}

async function handleSubmit() {
  if (!form.value.name.trim()) {
    toast.error("请输入任务名称");
    return;
  }
  if (!form.value.cronExpression.trim()) {
    toast.error("请输入 Cron 表达式");
    return;
  }
  if (form.value.taskType === "Command" && !form.value.command.trim()) {
    toast.error("命令任务需要填写命令内容");
    return;
  }

  try {
    if (isEditing.value) {
      await schedulerApi.updateTask({
        id: form.value.id,
        name: form.value.name,
        taskType: form.value.taskType,
        cronExpression: form.value.cronExpression,
        command: form.value.command || null,
      });
      toast.success("任务已更新");
    } else {
      await schedulerApi.createTask({
        name: form.value.name,
        taskType: form.value.taskType,
        cronExpression: form.value.cronExpression,
        command: form.value.command || null,
      });
      toast.success("任务已创建");
    }
    isDialogOpen.value = false;
    await loadTasks();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : "保存任务失败");
  }
}

async function handleDelete(task: ScheduledTask) {
  try {
    await schedulerApi.deleteTask(task.id);
    toast.success("任务已删除");
    await loadTasks();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : "删除任务失败");
  }
}

async function handleToggle(task: ScheduledTask) {
  try {
    await schedulerApi.toggleTask(task.id);
    await loadTasks();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : "切换状态失败");
  }
}

async function handleRunNow(task: ScheduledTask) {
  try {
    await schedulerApi.runTaskNow(task.id);
    toast.success("任务已触发");
  } catch (error) {
    toast.error(error instanceof Error ? error.message : "立即执行失败");
  }
}

function formatTime(value?: string | null) {
  return value ? new Date(value).toLocaleString() : "—";
}

function getTaskTypeLabel(type: SchedulerTaskType) {
  return taskTypeOptions.find((item) => item.value === type)?.label ?? type;
}

onMounted(() => {
  loadTasks();
});
</script>

<template>
  <div class="scheduler-page">
    <div class="page-header">
      <div>
        <h1 class="page-title">定时任务</h1>
        <p class="page-description">安排服务器重启、世界备份和控制台命令执行。</p>
      </div>
      <SLButton variant="primary" size="sm" @click="openCreateDialog">
        <Plus :size="16" />
        新增任务
      </SLButton>
    </div>

    <section class="panel">
      <div class="panel-header">
        <div class="panel-title-wrap">
          <CalendarClock :size="18" />
          <h2 class="panel-title">任务列表</h2>
        </div>
      </div>

      <div class="table-wrap">
        <table class="task-table">
          <thead>
            <tr>
              <th>名称</th>
              <th>类型</th>
              <th>Cron</th>
              <th>状态</th>
              <th>上次执行</th>
              <th>下次执行</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="task in tasks" :key="task.id">
              <td>{{ task.name }}</td>
              <td>{{ getTaskTypeLabel(task.task_type) }}</td>
              <td>{{ task.cron_expression }}</td>
              <td>
                <Switch
                  :model-value="task.enabled"
                  @update:model-value="() => handleToggle(task)"
                  class="scheduler-switch"
                >
                  <span class="switch-label">{{ task.enabled ? "启用" : "禁用" }}</span>
                </Switch>
              </td>
              <td>{{ formatTime(task.last_run) }}</td>
              <td>{{ formatTime(task.next_run) }}</td>
              <td>
                <div class="action-group">
                  <button class="icon-button" @click="handleRunNow(task)" title="立即执行">
                    <Play :size="16" />
                  </button>
                  <button class="icon-button" @click="openEditDialog(task)" title="编辑">
                    <Pencil :size="16" />
                  </button>
                  <button class="icon-button danger" @click="handleDelete(task)" title="删除">
                    <Trash2 :size="16" />
                  </button>
                </div>
              </td>
            </tr>
            <tr v-if="tasks.length === 0">
              <td colspan="7" class="empty-state">暂无任务，点击新增任务开始配置。</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <Dialog :open="isDialogOpen" @close="isDialogOpen = false" class="dialog-root">
      <div class="dialog-backdrop" />
      <div class="dialog-shell">
        <DialogPanel class="dialog-panel">
          <DialogTitle class="dialog-title">{{ isEditing ? "编辑任务" : "新增任务" }}</DialogTitle>
          <div class="form-grid">
            <label class="field">
              <span>任务名称</span>
              <input v-model="form.name" placeholder="例如：凌晨重启服务器" />
            </label>

            <label class="field">
              <span>任务类型</span>
              <select v-model="form.taskType">
                <option v-for="option in taskTypeOptions" :key="option.value" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>

            <label class="field">
              <span>Cron 表达式</span>
              <input v-model="form.cronExpression" placeholder="0 4 * * *" />
            </label>

            <div class="preset-list">
              <button v-for="preset in cronPresets" :key="preset.value" class="preset-chip" @click="form.cronExpression = preset.value">
                {{ preset.label }}
              </button>
            </div>

            <label v-if="showCommandField" class="field">
              <span>执行命令</span>
              <textarea v-model="form.command" rows="4" placeholder="例如：say 服务器将要重启" />
            </label>
          </div>

          <div class="dialog-actions">
            <SLButton variant="secondary" size="sm" @click="isDialogOpen = false">取消</SLButton>
            <SLButton variant="primary" size="sm" @click="handleSubmit">保存</SLButton>
          </div>
        </DialogPanel>
      </div>
    </Dialog>
  </div>
</template>

<style scoped>
.scheduler-page {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.page-title {
  font-size: 1.4rem;
  font-weight: 700;
  color: var(--sl-text-primary);
}

.page-description {
  margin-top: 0.25rem;
  color: var(--sl-text-secondary);
}

.panel {
  border-radius: var(--sl-radius-xl);
  background: var(--sl-surface, rgba(255, 255, 255, 0.65));
  border: 1px solid var(--sl-border, rgba(255, 255, 255, 0.12));
  padding: 1rem;
  box-shadow: var(--sl-shadow-md);
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.panel-title-wrap {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.panel-title {
  font-size: 1rem;
  font-weight: 600;
}

.table-wrap {
  overflow-x: auto;
}

.task-table {
  width: 100%;
  border-collapse: collapse;
}

.task-table th,
.task-table td {
  padding: 0.8rem 0.6rem;
  border-bottom: 1px solid var(--sl-border, rgba(255,255,255,0.1));
  text-align: left;
}

.task-table th {
  color: var(--sl-text-secondary);
  font-size: 0.85rem;
}

.empty-state {
  text-align: center;
  color: var(--sl-text-secondary);
  padding: 1.5rem 0;
}

.action-group {
  display: flex;
  gap: 0.3rem;
}

.icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 999px;
  border: 1px solid var(--sl-border, rgba(255,255,255,0.14));
  background: transparent;
  color: var(--sl-text-primary);
  cursor: pointer;
}

.icon-button.danger {
  color: #f87171;
}

.form-grid {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  color: var(--sl-text-secondary);
}

.field input,
.field select,
.field textarea {
  border: 1px solid var(--sl-border, rgba(255,255,255,0.14));
  border-radius: var(--sl-radius-md);
  background: rgba(255,255,255,0.05);
  color: var(--sl-text-primary);
  padding: 0.65rem 0.75rem;
}

.preset-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.preset-chip {
  border: 1px solid var(--sl-border, rgba(255,255,255,0.14));
  border-radius: 999px;
  padding: 0.35rem 0.6rem;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
}

.dialog-root {
  position: relative;
  z-index: 50;
}

.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.5);
  backdrop-filter: blur(4px);
}

.dialog-shell {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.dialog-panel {
  width: min(100%, 560px);
  border-radius: var(--sl-radius-xl);
  background: var(--sl-surface, rgba(255, 255, 255, 0.88));
  border: 1px solid var(--sl-border, rgba(255, 255, 255, 0.16));
  padding: 1rem;
  box-shadow: var(--sl-shadow-lg);
}

.dialog-title {
  margin-bottom: 0.75rem;
  font-size: 1.1rem;
  font-weight: 600;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 1rem;
}

.scheduler-switch {
  display: inline-flex;
  align-items: center;
}

.switch-label {
  margin-left: 0.5rem;
  color: var(--sl-text-secondary);
}
</style>
