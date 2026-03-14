use crate::utils::file;

/// Export logs to file
/// path: target file path (selected by user through Dialog)
/// content: formatted log content string
#[tauri::command]
pub async fn export_logs(path: String, content: String) -> Result<String, String> {
    log::info!("[Logs] Exporting logs to: {}", path);

    file::write_file(&path, &content).map_err(|e| format!("无法写入文件: {}", e))?;

    Ok(format!("日志已导出到 {}", path))
}
