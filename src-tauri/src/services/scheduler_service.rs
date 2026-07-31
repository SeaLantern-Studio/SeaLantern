use crate::models::scheduler::{ScheduledTask, TaskType};
use crate::services::global;
use chrono::{DateTime, Utc};
use cron::Schedule;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;
use tokio::time::interval;
use uuid::Uuid;

const TASKS_FILE: &str = "scheduler_tasks.json";

#[derive(Debug, Clone)]
pub struct SchedulerService {
    tasks: Arc<Mutex<Vec<ScheduledTask>>>,
    data_dir: PathBuf,
    running_tasks: Arc<Mutex<HashSet<String>>>,
}

impl SchedulerService {
    pub fn new() -> Self {
        Self::new_with_options(None, true)
    }

    pub fn new_with_options(data_dir: Option<PathBuf>, start_background_loop: bool) -> Self {
        let data_dir = data_dir.unwrap_or_else(|| {
            PathBuf::from(crate::utils::path::get_or_create_app_data_dir()).join("data")
        });
        let _ = fs::create_dir_all(&data_dir);
        let tasks = load_tasks(&data_dir);
        let service = Self {
            tasks: Arc::new(Mutex::new(tasks)),
            data_dir: data_dir.clone(),
            running_tasks: Arc::new(Mutex::new(HashSet::new())),
        };
        if start_background_loop {
            service.start_background_loop();
        }
        service
    }

    pub fn add_task(&self, mut task: ScheduledTask) -> Result<ScheduledTask, String> {
        if task.id.is_empty() {
            task.id = Uuid::new_v4().to_string();
        }

        let next_run = compute_next_run(&task.cron_expression)?;
        task.next_run = Some(next_run);
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| format!("任务锁被污染: {e}"))?;
        tasks.push(task.clone());
        save_tasks(&self.data_dir, &tasks)?;
        Ok(task)
    }

    pub fn remove_task(&self, id: &str) -> Result<(), String> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| format!("任务锁被污染: {e}"))?;
        let original_len = tasks.len();
        tasks.retain(|task| task.id != id);
        if tasks.len() == original_len {
            return Err(format!("未找到任务: {id}"));
        }
        save_tasks(&self.data_dir, &tasks)?;
        Ok(())
    }

    pub fn update_task(&self, task: ScheduledTask) -> Result<ScheduledTask, String> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| format!("任务锁被污染: {e}"))?;
        let index = tasks
            .iter()
            .position(|existing| existing.id == task.id)
            .ok_or_else(|| format!("未找到任务: {}", task.id))?;
        let existing = tasks[index].clone();
        let mut updated = task;
        updated.last_run = existing.last_run;
        updated.next_run =
            if updated.cron_expression != existing.cron_expression || existing.next_run.is_none() {
                Some(compute_next_run(&updated.cron_expression)?)
            } else {
                existing.next_run
            };
        tasks[index] = updated.clone();
        save_tasks(&self.data_dir, &tasks)?;
        Ok(updated)
    }

    pub fn toggle_task(&self, id: &str) -> Result<ScheduledTask, String> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| format!("任务锁被污染: {e}"))?;
        let index = tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| format!("未找到任务: {id}"))?;
        tasks[index].enabled = !tasks[index].enabled;
        let task = tasks[index].clone();
        save_tasks(&self.data_dir, &tasks)?;
        Ok(task)
    }

    pub fn get_all_tasks(&self) -> Vec<ScheduledTask> {
        self.tasks.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn run_task_now(&self, id: &str) -> Result<(), String> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| format!("任务锁被污染: {e}"))?;
        let index = tasks
            .iter()
            .position(|item| item.id == id)
            .ok_or_else(|| format!("未找到任务: {id}"))?;
        let now = Utc::now();
        tasks[index].last_run = Some(now);
        tasks[index].next_run = Some(compute_next_run(&tasks[index].cron_expression)?);
        let task = tasks[index].clone();
        save_tasks(&self.data_dir, &tasks)?;
        drop(tasks);
        self.execute_task(task)
    }

    fn start_background_loop(&self) {
        let tasks = Arc::clone(&self.tasks);
        let running_tasks = Arc::clone(&self.running_tasks);
        tauri::async_runtime::spawn(async move {
            let mut interval = interval(StdDuration::from_secs(15));
            loop {
                interval.tick().await;
                if let Err(err) = Self::tick(&tasks, &running_tasks).await {
                    eprintln!("定时任务检查失败: {err}");
                }
            }
        });
    }

    async fn tick(
        tasks: &Arc<Mutex<Vec<ScheduledTask>>>,
        running_tasks: &Arc<Mutex<HashSet<String>>>,
    ) -> Result<(), String> {
        let now = Utc::now();
        let pending_tasks = {
            let tasks_guard = tasks.lock().map_err(|e| format!("任务锁被污染: {e}"))?;
            tasks_guard
                .iter()
                .filter(|task| {
                    task.enabled && task.next_run.is_some_and(|next_run| next_run <= now)
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        let mut dirty = false;

        for task in pending_tasks {
            let result = Self::dispatch_task(&task, running_tasks).await;
            if let Err(err) = result {
                eprintln!("定时任务执行失败: {err}");
                continue;
            }

            let mut tasks_guard = tasks.lock().map_err(|e| format!("任务锁被污染: {e}"))?;
            let current = tasks_guard
                .iter_mut()
                .find(|item| item.id == task.id)
                .ok_or_else(|| format!("未找到任务: {}", task.id))?;
            current.last_run = Some(now);
            current.next_run = Some(compute_next_run(&current.cron_expression)?);
            dirty = true;
        }

        if dirty {
            let tasks_guard = tasks.lock().map_err(|e| format!("任务锁被污染: {e}"))?;
            save_tasks_from_locked(&tasks_guard)?;
        }

        Ok(())
    }

    fn execute_task(&self, task: ScheduledTask) -> Result<(), String> {
        let running_tasks = Arc::clone(&self.running_tasks);
        tauri::async_runtime::spawn(async move {
            if let Err(err) = Self::dispatch_task(&task, &running_tasks).await {
                eprintln!("任务执行失败: {err}");
            }
        });
        Ok(())
    }

    async fn dispatch_task(
        task: &ScheduledTask,
        running_tasks: &Arc<Mutex<HashSet<String>>>,
    ) -> Result<(), String> {
        let task_id = task.id.clone();
        {
            let mut running = running_tasks
                .lock()
                .map_err(|e| format!("任务锁被污染: {e}"))?;
            if running.contains(&task_id) {
                return Err(format!("任务 {} 正在执行，已跳过", task_id));
            }
            running.insert(task_id.clone());
        }

        let result = execute_task_internal(task).await;

        {
            let mut running = running_tasks
                .lock()
                .map_err(|e| format!("任务锁被污染: {e}"))?;
            running.remove(&task_id);
        }
        result
    }
}

fn load_tasks(data_dir: &PathBuf) -> Vec<ScheduledTask> {
    let path = data_dir.join(TASKS_FILE);
    if !path.exists() {
        let _ = save_tasks(data_dir, &Vec::new());
        return Vec::new();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_tasks(data_dir: &PathBuf, tasks: &[ScheduledTask]) -> Result<(), String> {
    let path = data_dir.join(TASKS_FILE);
    let json = serde_json::to_string_pretty(tasks).map_err(|e| format!("序列化任务失败: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("保存任务失败: {e}"))
}

fn save_tasks_from_locked(
    tasks: &std::sync::MutexGuard<'_, Vec<ScheduledTask>>,
) -> Result<(), String> {
    let data_dir = PathBuf::from(crate::utils::path::get_or_create_app_data_dir()).join("data");
    save_tasks(&data_dir, tasks)
}

async fn execute_task_internal(task: &ScheduledTask) -> Result<(), String> {
    let server_manager = global::server_manager();
    println!("[Scheduler] 执行任务: {} ({})", task.name, task.task_type.as_str());

    match task.task_type {
        TaskType::Restart => {
            server_manager
                .stop_server("default")
                .map_err(|e| format!("停止服务器失败: {e}"))?;
            server_manager
                .start_server("default")
                .map_err(|e| format!("启动服务器失败: {e}"))?;
            Ok(())
        }
        TaskType::Backup => {
            // TODO: 调用现有备份服务
            println!("[Scheduler] Backup task placeholder executed for {}", task.name);
            Ok(())
        }
        TaskType::Command => {
            let command = task.command.as_deref().unwrap_or_default();
            if command.is_empty() {
                return Err("命令任务缺少 command 内容".to_string());
            }
            server_manager
                .send_command("default", command)
                .map_err(|e| format!("发送控制台命令失败: {e}"))?;
            Ok(())
        }
    }
}

fn compute_next_run(cron_expression: &str) -> Result<DateTime<Utc>, String> {
    let normalized = normalize_cron_expression(cron_expression)?;
    let schedule =
        Schedule::from_str(&normalized).map_err(|e| format!("无效的 cron 表达式: {e}"))?;
    let mut upcoming = schedule.upcoming(Utc);
    upcoming
        .next()
        .ok_or_else(|| "无法计算下次执行时间".to_string())
}

fn normalize_cron_expression(cron_expression: &str) -> Result<String, String> {
    let trimmed = cron_expression.trim();
    let field_count = trimmed.split_whitespace().count();
    match field_count {
        5 => Ok(format!("0 {trimmed}")),
        6 => Ok(trimmed.to_string()),
        _ => Err(format!("无效的 cron 表达式: {cron_expression}")),
    }
}

impl TaskType {
    fn as_str(&self) -> &'static str {
        match self {
            TaskType::Restart => "restart",
            TaskType::Backup => "backup",
            TaskType::Command => "command",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tempfile::tempdir;

    #[test]
    fn parses_valid_cron_expression() {
        let next = compute_next_run("0 4 * * *").expect("should parse");
        assert!(next > Utc::now());
    }

    #[test]
    fn update_task_preserves_enabled_and_run_history() {
        let temp_dir = tempdir().expect("create temp dir");
        let service =
            SchedulerService::new_with_options(Some(temp_dir.path().to_path_buf()), false);
        let original_last_run = Utc::now() - Duration::hours(2);
        let original_next_run = Utc::now() + Duration::hours(1);
        let original = ScheduledTask {
            id: "task-1".to_string(),
            name: "测试任务".to_string(),
            task_type: TaskType::Restart,
            cron_expression: "0 4 * * *".to_string(),
            command: None,
            enabled: false,
            last_run: Some(original_last_run),
            next_run: Some(original_next_run),
        };

        service.add_task(original.clone()).expect("should add task");

        let updated = service
            .update_task(ScheduledTask {
                id: original.id.clone(),
                name: "更新后的任务".to_string(),
                task_type: TaskType::Backup,
                cron_expression: "0 5 * * *".to_string(),
                command: None,
                enabled: true,
                last_run: None,
                next_run: None,
            })
            .expect("should update task");

        assert!(updated.enabled);
        assert_eq!(updated.last_run, Some(original_last_run));
        assert!(updated.next_run.is_some());
    }

    #[test]
    fn run_task_now_updates_last_run_and_next_run() {
        let temp_dir = tempdir().expect("create temp dir");
        let service =
            SchedulerService::new_with_options(Some(temp_dir.path().to_path_buf()), false);
        let original_next_run = Utc::now() + Duration::hours(1);
        let task = ScheduledTask {
            id: "task-2".to_string(),
            name: "立即执行任务".to_string(),
            task_type: TaskType::Backup,
            cron_expression: "0 4 * * *".to_string(),
            command: None,
            enabled: true,
            last_run: None,
            next_run: Some(original_next_run),
        };

        service.add_task(task.clone()).expect("should add task");

        service.run_task_now("task-2").expect("should run task now");

        let tasks = service.get_all_tasks();
        let updated = tasks.iter().find(|item| item.id == "task-2").unwrap();
        assert!(updated.last_run.is_some());
        assert!(updated.next_run.is_some());
        assert!(updated.next_run.unwrap() > original_next_run);
    }
}
