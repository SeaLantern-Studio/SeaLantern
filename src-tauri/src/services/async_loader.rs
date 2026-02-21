use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// 异步加载任务类型
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LoadTaskType {
    PluginScan,
    ConfigLoad,
    JavaDetect,
    ServerListLoad,
    SettingsLoad,
    MarketDataFetch,
}

/// 异步加载任务状态
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LoadTaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

/// 异步加载任务
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoadTask {
    pub task_type: LoadTaskType,
    pub status: LoadTaskStatus,
    pub progress: f32,
    pub message: String,
}

impl LoadTask {
    #[allow(dead_code)]
    pub fn new(task_type: LoadTaskType) -> Self {
        Self {
            task_type,
            status: LoadTaskStatus::Pending,
            progress: 0.0,
            message: String::new(),
        }
    }
}

/// 异步加载器，用于管理应用程序启动时的异步初始化任务
#[allow(dead_code)]
pub struct AsyncLoader {
    tx: mpsc::Sender<LoadTask>,
    rx: mpsc::Receiver<LoadTask>,
    tasks: Vec<LoadTask>,
}

impl AsyncLoader {
    /// 创建新的异步加载器
    #[allow(dead_code)]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self { tx, rx, tasks: Vec::new() }
    }

    /// 添加一个异步任务
    #[allow(dead_code)]
    pub fn add_task(&mut self, task: LoadTask) {
        self.tasks.push(task);
    }

    /// 获取发送器，用于在任务中报告进度
    #[allow(dead_code)]
    pub fn get_sender(&self) -> mpsc::Sender<LoadTask> {
        self.tx.clone()
    }

    /// 获取所有任务的状态
    #[allow(dead_code)]
    pub fn get_tasks(&self) -> &[LoadTask] {
        &self.tasks
    }

    /// 检查是否所有任务都已完成
    #[allow(dead_code)]
    pub fn is_all_completed(&self) -> bool {
        self.tasks
            .iter()
            .all(|t| matches!(t.status, LoadTaskStatus::Completed | LoadTaskStatus::Failed(_)))
    }

    /// 获取总体进度 (0.0 - 1.0)
    #[allow(dead_code)]
    pub fn get_total_progress(&self) -> f32 {
        if self.tasks.is_empty() {
            return 1.0;
        }
        let total: f32 = self.tasks.iter().map(|t| t.progress).sum();
        total / self.tasks.len() as f32
    }

    /// 更新任务状态
    #[allow(dead_code)]
    pub fn update_task<F>(&mut self, task_type: LoadTaskType, updater: F)
    where
        F: FnOnce(&mut LoadTask),
    {
        if let Some(task) = self
            .tasks
            .iter_mut()
            .find(|t| std::mem::discriminant(&t.task_type) == std::mem::discriminant(&task_type))
        {
            updater(task);
        }
    }

    /// 处理接收到的任务更新
    #[allow(dead_code)]
    pub async fn process_updates(&mut self) {
        while let Ok(updated_task) = self.rx.try_recv() {
            if let Some(task) = self.tasks.iter_mut().find(|t| {
                std::mem::discriminant(&t.task_type)
                    == std::mem::discriminant(&updated_task.task_type)
            }) {
                *task = updated_task;
            }
        }
    }
}

impl Default for AsyncLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 异步执行一个加载任务
#[allow(dead_code)]
pub fn spawn_load_task<F>(
    task_type: LoadTaskType,
    tx: mpsc::Sender<LoadTask>,
    task_fn: F,
) -> JoinHandle<()>
where
    F: FnOnce(mpsc::Sender<LoadTask>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'static,
{
    tokio::spawn(async move {
        // 发送任务开始状态
        let mut task = LoadTask::new(task_type.clone());
        task.status = LoadTaskStatus::Running;
        task.message = "开始执行...".to_string();
        let _ = tx.send(task).await;

        // 执行任务
        task_fn(tx.clone()).await;
    })
}

/// 报告任务进度
#[allow(dead_code)]
pub async fn report_progress(
    tx: &mpsc::Sender<LoadTask>,
    task_type: LoadTaskType,
    progress: f32,
    message: impl Into<String>,
) {
    let mut task = LoadTask::new(task_type);
    task.status = LoadTaskStatus::Running;
    task.progress = progress.clamp(0.0, 1.0);
    task.message = message.into();
    let _ = tx.send(task).await;
}

/// 报告任务完成
#[allow(dead_code)]
pub async fn report_completed(tx: &mpsc::Sender<LoadTask>, task_type: LoadTaskType) {
    let mut task = LoadTask::new(task_type);
    task.status = LoadTaskStatus::Completed;
    task.progress = 1.0;
    task.message = "完成".to_string();
    let _ = tx.send(task).await;
}

/// 报告任务失败
#[allow(dead_code)]
pub async fn report_failed(
    tx: &mpsc::Sender<LoadTask>,
    task_type: LoadTaskType,
    error: impl Into<String>,
) {
    let mut task = LoadTask::new(task_type);
    task.status = LoadTaskStatus::Failed(error.into());
    task.progress = 1.0;
    task.message = "失败".to_string();
    let _ = tx.send(task).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_loader() {
        let mut loader = AsyncLoader::new();

        loader.add_task(LoadTask::new(LoadTaskType::PluginScan));
        loader.add_task(LoadTask::new(LoadTaskType::ConfigLoad));

        assert_eq!(loader.get_tasks().len(), 2);
        assert!(!loader.is_all_completed());

        // 模拟更新任务状态
        loader.update_task(LoadTaskType::PluginScan, |t| {
            t.status = LoadTaskStatus::Completed;
            t.progress = 1.0;
        });

        loader.update_task(LoadTaskType::ConfigLoad, |t| {
            t.status = LoadTaskStatus::Completed;
            t.progress = 1.0;
        });

        assert!(loader.is_all_completed());
        assert_eq!(loader.get_total_progress(), 1.0);
    }

    #[tokio::test]
    async fn test_report_progress() {
        let (tx, mut rx) = mpsc::channel(10);

        report_progress(&tx, LoadTaskType::PluginScan, 0.5, "扫描中...").await;

        if let Some(task) = rx.recv().await {
            assert!(matches!(task.status, LoadTaskStatus::Running));
            assert_eq!(task.progress, 0.5);
            assert_eq!(task.message, "扫描中...");
        }
    }
}
