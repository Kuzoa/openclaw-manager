import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { isTauri } from './tauri';
import { LogEntry } from './logger';

/**
 * Format logs to string
 */
const formatLogs = (logs: LogEntry[]): string => {
  return logs.map(log => {
    const time = log.timestamp.toLocaleTimeString('zh-CN', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
    const args = log.args.length > 0 ? ' ' + JSON.stringify(log.args) : '';
    return `[${time}] [${log.level.toUpperCase()}] [${log.module}] ${log.message}${args}`;
  }).join('\n');
};

/**
 * Export log file
 * @param logs Log entries to export
 * @param defaultFilename Default filename (optional)
 * @returns Promise<boolean> - true for success, false for user cancel
 */
export async function exportLogFile(
  logs: LogEntry[],
  defaultFilename?: string
): Promise<boolean> {
  const content = formatLogs(logs);
  const filename = defaultFilename || `openclaw-manager-logs-${new Date().toISOString().slice(0, 10)}.txt`;

  if (isTauri()) {
    // Tauri environment: use Dialog + Rust command
    const path = await save({
      filters: [{ name: 'Text', extensions: ['txt'] }],
      defaultPath: filename,
    });

    // User cancelled
    if (!path) {
      return false;
    }

    // Call Rust command to write file
    await invoke('export_logs', { path, content });
    return true;
  } else {
    // Non-Tauri environment: browser fallback (Blob + a.click())
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
    return true;
  }
}
