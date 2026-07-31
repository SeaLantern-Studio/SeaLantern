use crate::models::scheduler::{ScheduledTask, TaskType};
use crate::services::global;

fn service() -> &'static crate::services::scheduler_service::SchedulerService {
    global::scheduler_service()
}

#[tauri::command]
pub fn get_all_tasks() -> Vec<ScheduledTask> {
    service().get_all_tasks()
}

#[tauri::command]
pub fn create_task(
    name: String,
    task_type: TaskType,
    cron_expression: String,
    command: Option<String>,
) -> Result<ScheduledTask, String> {
    let task = ScheduledTask {
        id: String::new(),
        name,
        task_type,
        cron_expression,
        command,
        enabled: true,
        last_run: None,
        next_run: None,
    };
    service().add_task(task)
}

#[tauri::command]
pub fn update_task(
    id: String,
    name: String,
    task_type: TaskType,
    cron_expression: String,
    command: Option<String>,
    enabled: bool,
) -> Result<ScheduledTask, String> {
    let task = ScheduledTask {
        id,
        name,
        task_type,
        cron_expression,
        command,
        enabled,
        last_run: None,
        next_run: None,
    };
    service().update_task(task)
}

#[tauri::command]
pub fn delete_task(id: String) -> Result<(), String> {
    service().remove_task(&id)
}

#[tauri::command]
pub fn toggle_task(id: String) -> Result<ScheduledTask, String> {
    service().toggle_task(&id)
}

#[tauri::command]
pub fn run_task_now(id: String) -> Result<(), String> {
    service().run_task_now(&id)
}
