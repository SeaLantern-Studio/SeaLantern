//! 服务器日志管线模块：统一负责日志读流、来源标注、SQLite 持久化、事件推送和历史读取。
//! ServerManager 只负责流程编排，日志实现细节都收在子模块里。

mod db;
mod output;
mod reader;
mod state;
mod writer;

pub use state::{ServerLogEventHandler, ServerLogProcessor};

pub fn set_server_log_event_handler(handler: ServerLogEventHandler) -> Result<(), String> {
    writer::set_server_log_event_handler(handler)
}

#[allow(dead_code)] // 外部调用
pub fn add_server_log_processor(processor: ServerLogProcessor) -> Result<(), String> {
    output::add_server_log_processor(processor)
}

#[allow(dead_code)] // 外部调用
pub fn clear_server_log_processors() -> Result<(), String> {
    output::clear_server_log_processors()
}

pub fn init_db(server_path: &std::path::Path) -> Result<(), String> {
    writer::init_db(server_path)
}

pub fn shutdown_writer(server_id: &str) {
    writer::shutdown_writer(server_id)
}

pub fn append_sealantern_log(server_id: &str, message: &str) -> Result<(), String> {
    writer::append_sealantern_log(server_id, message)
}

#[allow(dead_code)] // 外部调用
pub fn append_server_log(server_id: &str, message: &str) -> Result<(), String> {
    writer::append_server_log(server_id, message)
}

pub fn get_logs(server_id: &str, since: usize, recent_limit: Option<usize>) -> Vec<String> {
    reader::get_logs(server_id, since, recent_limit)
}

pub fn get_all_logs() -> Vec<(String, Vec<String>)> {
    reader::get_all_logs()
}

pub fn spawn_server_output_reader<R>(server_id: String, reader: R)
where
    R: std::io::Read + Send + 'static,
{
    output::spawn_server_output_reader(server_id, reader)
}
