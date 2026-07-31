import { tauriInvoke } from "@api/tauri";

export type SchedulerTaskType = "Restart" | "Backup" | "Command";

export interface ScheduledTask {
  id: string;
  name: string;
  task_type: SchedulerTaskType;
  cron_expression: string;
  command?: string | null;
  enabled: boolean;
  last_run?: string | null;
  next_run?: string | null;
}

export const schedulerApi = {
  async getAllTasks(): Promise<ScheduledTask[]> {
    return tauriInvoke<ScheduledTask[]>("get_all_tasks");
  },

  async createTask(params: {
    name: string;
    taskType: SchedulerTaskType;
    cronExpression: string;
    command?: string | null;
  }): Promise<ScheduledTask> {
    return tauriInvoke<ScheduledTask>("create_task", {
      name: params.name,
      task_type: params.taskType,
      cron_expression: params.cronExpression,
      command: params.command ?? null,
    });
  },

  async updateTask(params: {
    id: string;
    name: string;
    taskType: SchedulerTaskType;
    cronExpression: string;
    command?: string | null;
    enabled: boolean;
  }): Promise<ScheduledTask> {
    return tauriInvoke<ScheduledTask>("update_task", {
      id: params.id,
      name: params.name,
      task_type: params.taskType,
      cron_expression: params.cronExpression,
      command: params.command ?? null,
      enabled: params.enabled,
    });
  },

  async deleteTask(id: string): Promise<void> {
    return tauriInvoke<void>("delete_task", { id });
  },

  async toggleTask(id: string): Promise<ScheduledTask> {
    return tauriInvoke<ScheduledTask>("toggle_task", { id });
  },

  async runTaskNow(id: string): Promise<void> {
    return tauriInvoke<void>("run_task_now", { id });
  },
};
