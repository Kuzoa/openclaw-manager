import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
import {
  Save,
  Loader2,
  Trash2,
  AlertTriangle,
  X,
  Globe,
  Download,
  Upload,
  RefreshCw,
  CheckCircle,
  ArrowUpCircle,
  Database,
  Clock,
  Server,
  FileJson,
  GitMerge,
  Languages
} from 'lucide-react';
import { appLogger } from '../../lib/logger';
import { isTauri } from '../../lib/tauri';
import { useTranslation } from 'react-i18next';

interface InstallResult {
  success: boolean;
  message: string;
  error?: string;
}

interface SettingsProps {
  onEnvironmentChange?: () => void;
}

interface BrowserConfig {
  enabled: boolean;
  color: string | null;
}

interface WebConfig {
  brave_api_key: string | null;
}

interface CompactionConfig {
  enabled: boolean;
  threshold: number | null;
  context_pruning: boolean;
  max_context_messages: number | null;
}

interface WorkspaceConfig {
  workspace: string | null;
  timezone: string | null;
  time_format: string | null;
  skip_bootstrap: boolean;
  bootstrap_max_chars: number | null;
}

interface GatewayConfig {
  port: number;
  log_level: string;
}

interface SubagentDefaults {
  max_spawn_depth: number | null;
  max_children_per_agent: number | null;
  max_concurrent: number | null;
  attachments_enabled: boolean | null;
  attachments_max_total_bytes: number | null;
}

interface PdfConfig {
  max_pages: number | null;
  max_bytes_mb: number | null;
}

interface MemoryConfig {
  provider: string | null;
}

export function Settings({ onEnvironmentChange }: SettingsProps) {
  const { t, i18n } = useTranslation('settings');
  const { t: tCommon } = useTranslation('common');
  
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  const [showUninstallConfirm, setShowUninstallConfirm] = useState(false);
  const [uninstalling, setUninstalling] = useState(false);
  const [uninstallResult, setUninstallResult] = useState<InstallResult | null>(null);

  // Manager self-update states
  const [managerChecking, setManagerChecking] = useState(false);
  const [managerUpdateAvailable, setManagerUpdateAvailable] = useState(false);
  const [managerUpdateVersion, setManagerUpdateVersion] = useState<string | null>(null);
  const [managerUpdateBody, setManagerUpdateBody] = useState<string | null>(null);
  const [managerDownloading, setManagerDownloading] = useState(false);
  const [managerDownloadProgress, setManagerDownloadProgress] = useState(0);
  const [managerUpdateDone, setManagerUpdateDone] = useState(false);
  const [managerUpdateError, setManagerUpdateError] = useState<string | null>(null);
  const [managerUpdateObj, setManagerUpdateObj] = useState<any>(null);
  const [managerCheckDone, setManagerCheckDone] = useState(false);

  // Config States
  const [browser, setBrowser] = useState<BrowserConfig>({ enabled: true, color: null });
  const [webConfig, setWebConfig] = useState<WebConfig>({ brave_api_key: null });
  const [compaction, setCompaction] = useState<CompactionConfig>({ enabled: false, threshold: null, context_pruning: false, max_context_messages: null });
  const [workspace, setWorkspace] = useState<WorkspaceConfig>({ workspace: null, timezone: null, time_format: null, skip_bootstrap: false, bootstrap_max_chars: null });
  const [gateway, setGateway] = useState<GatewayConfig>({ port: 3000, log_level: 'info' });
  const [subagentDefaults, setSubagentDefaults] = useState<SubagentDefaults>({ max_spawn_depth: null, max_children_per_agent: null, max_concurrent: null, attachments_enabled: null, attachments_max_total_bytes: null });
  const [toolsProfile, setToolsProfile] = useState<string>('messaging');
  const [pdfConfig, setPdfConfig] = useState<PdfConfig>({ max_pages: null, max_bytes_mb: null });
  const [memoryConfig, setMemoryConfig] = useState<MemoryConfig>({ provider: null });
  const [appVersion, setAppVersion] = useState<string>('...');

  const [validating, setValidating] = useState(false);
  const [validateStatus, setValidateStatus] = useState<{ success: boolean; message: string } | null>(null);

  // Load initial data
  useEffect(() => {
    const loadConfig = async () => {
      setLoading(true);
      try {
        const [br, web, comp, ws, gw, sub, profile, pdf, mem] = await Promise.all([
          invoke<BrowserConfig>('get_browser_config'),
          invoke<WebConfig>('get_web_config'),
          invoke<CompactionConfig>('get_compaction_config'),
          invoke<WorkspaceConfig>('get_workspace_config'),
          invoke<GatewayConfig>('get_gateway_config'),
          invoke<SubagentDefaults>('get_subagent_defaults'),
          invoke<string>('get_tools_profile'),
          invoke<PdfConfig>('get_pdf_config'),
          invoke<MemoryConfig>('get_memory_config'),
        ]);
        setBrowser(br);
        setWebConfig(web);
        setCompaction(comp);
        setWorkspace(ws);
        setGateway(gw);
        setSubagentDefaults(sub);
        setToolsProfile(profile);
        setPdfConfig(pdf);
        setMemoryConfig(mem);

        if (isTauri()) {
          const { getVersion } = await import('@tauri-apps/api/app');
          setAppVersion(await getVersion());
        }
      } catch (e) {
        appLogger.error('Failed to load settings', e);
      } finally {
        setLoading(false);
      }
    };
    loadConfig();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setSaveSuccess(false);
    try {
      await Promise.all([
        invoke('save_browser_config', { enabled: browser.enabled, color: browser.color }),
        invoke('save_web_config', { braveApiKey: webConfig.brave_api_key }),
        invoke('save_compaction_config', {
          enabled: compaction.enabled,
          threshold: compaction.threshold,
          contextPruning: compaction.context_pruning,
          maxContextMessages: compaction.max_context_messages
        }),
        invoke('save_workspace_config', {
          workspace: workspace.workspace,
          timezone: workspace.timezone,
          timeFormat: workspace.time_format,
          skipBootstrap: workspace.skip_bootstrap,
          bootstrapMaxChars: workspace.bootstrap_max_chars
        }),
        invoke('save_gateway_config', { port: gateway.port, logLevel: gateway.log_level }),
        invoke('save_subagent_defaults', { defaults: subagentDefaults }),
        invoke('save_tools_profile', { profile: toolsProfile }),
        invoke('save_pdf_config', { pdfConfig }),
        invoke('save_memory_config', { memoryConfig }),
      ]);

      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } catch (e) {
      console.error('Failed to save:', e);
      alert('Failed to save settings: ' + String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleValidate = async () => {
    setValidating(true);
    setValidateStatus(null);
    try {
      const currentConfig = await invoke<any>('get_config');
      const jsonStr = JSON.stringify(currentConfig, null, 2);

      await invoke<string>('validate_openclaw_config', { configJson: jsonStr });
      setValidateStatus({ success: true, message: 'Configuration is valid according to the CLI schema.' });
    } catch (e) {
      setValidateStatus({ success: false, message: String(e) });
    } finally {
      setValidating(false);
    }
  };

  const handleExport = async () => {
    try {
      const path = await save({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }],
        defaultPath: 'openclaw-config.json'
      });

      if (path) {
        await invoke('export_config', { path });
        alert('Configuration exported successfully!');
      }
    } catch (e) {
      console.error('Export failed:', e);
      alert('Failed to export configuration: ' + String(e));
    }
  };

  const handleImport = async () => {
    if (!confirm('Importing configuration will overwrite your current settings. Continue?')) return;

    try {
      const path = await open({
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });

      if (path) {
        await invoke('import_config', { path });
        alert('Configuration imported successfully! Please restart the manager to apply all changes.');
        window.location.reload();
      }
    } catch (e) {
      console.error('Import failed:', e);
      alert('Failed to import configuration: ' + String(e));
    }
  };

  const handleUninstall = async () => {
    setUninstalling(true);
    setUninstallResult(null);
    try {
      const result = await invoke<InstallResult>('uninstall_openclaw');
      setUninstallResult(result);
      if (result.success) {
        onEnvironmentChange?.();
        setTimeout(() => setShowUninstallConfirm(false), 2000);
      }
    } catch (e) {
      setUninstallResult({
        success: false,
        message: 'An error occurred during uninstallation',
        error: String(e),
      });
    } finally {
      setUninstalling(false);
    }
  };

  // Manager self-update: check for updates
  const checkManagerUpdate = async () => {
    if (!isTauri()) return;
    setManagerChecking(true);
    setManagerUpdateError(null);
    setManagerUpdateAvailable(false);
    setManagerCheckDone(false);
    setManagerUpdateDone(false);
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        setManagerUpdateAvailable(true);
        setManagerUpdateVersion(update.version);
        setManagerUpdateBody(update.body || null);
        setManagerUpdateObj(update);
      } else {
        setManagerCheckDone(true);
      }
    } catch (e: any) {
      appLogger.error('Manager update check failed', e);
      setManagerUpdateError(e?.message || String(e));
    } finally {
      setManagerChecking(false);
    }
  };

  // Manager self-update: download & install
  const downloadManagerUpdate = async () => {
    if (!managerUpdateObj) return;
    setManagerDownloading(true);
    setManagerDownloadProgress(0);
    setManagerUpdateError(null);
    try {
      let downloaded = 0;
      let contentLength = 1;
      await managerUpdateObj.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength || 1;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            setManagerDownloadProgress(Math.min(100, Math.round((downloaded / contentLength) * 100)));
            break;
          case 'Finished':
            setManagerDownloadProgress(100);
            break;
        }
      });
      setManagerUpdateDone(true);
    } catch (e: any) {
      appLogger.error('Manager update download failed', e);
      setManagerUpdateError(e?.message || String(e));
    } finally {
      setManagerDownloading(false);
    }
  };

  // Manager self-update: restart app
  const restartApp = async () => {
    try {
      const { relaunch } = await import('@tauri-apps/plugin-process');
      await relaunch();
    } catch (e: any) {
      appLogger.error('Relaunch failed', e);
    }
  };

  // Language change handler
  const handleLanguageChange = (lang: string) => {
    i18n.changeLanguage(lang);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="animate-spin text-claw-400" size={32} />
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2 pb-20">
      <div className="max-w-3xl space-y-6 mx-auto">

        {/* Language Settings - First Block */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-brand-500/20 flex items-center justify-center">
              <Languages size={20} className="text-brand-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('language.label')}</h3>
              <p className="text-xs text-gray-500">{t('language.description')}</p>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
            <div>
              <p className="text-sm text-white">{t('language.label')}</p>
              <p className="text-xs text-gray-500">{t('language.description')}</p>
            </div>
            <select
              value={i18n.language}
              onChange={e => handleLanguageChange(e.target.value)}
              className="input-base w-auto min-w-[140px]"
            >
              <option value="en">English</option>
              <option value="zh">中文</option>
            </select>
          </div>
        </div>

        {/* Compaction & Memory */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
              <Database size={20} className="text-purple-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('compaction.title')}</h3>
              <p className="text-xs text-gray-500">{t('compaction.description')}</p>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">{t('compaction.enable')}</p>
                <p className="text-xs text-gray-500">{t('compaction.enableDescription')}</p>
              </div>
              <input
                type="checkbox"
                checked={compaction.enabled}
                onChange={e => setCompaction({ ...compaction, enabled: e.target.checked })}
                className="w-5 h-5 rounded bg-dark-500 border-dark-400 text-claw-500 focus:ring-claw-500/50"
              />
            </div>

            {compaction.enabled && (
              <div className="pl-4 border-l-2 border-dark-600">
                <label className="block text-sm text-gray-400 mb-2">{t('compaction.threshold')}</label>
                <input
                  type="number"
                  value={compaction.threshold || ''}
                  onChange={e => setCompaction({ ...compaction, threshold: parseInt(e.target.value) || null })}
                  placeholder="e.g. 8000"
                  className="input-base"
                />
                <p className="text-xs text-gray-500 mt-1">{t('compaction.thresholdDescription')}</p>
              </div>
            )}

            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">{t('compaction.contextPruning')}</p>
                <p className="text-xs text-gray-500">{t('compaction.contextPruningDescription')}</p>
              </div>
              <input
                type="checkbox"
                checked={compaction.context_pruning}
                onChange={e => setCompaction({ ...compaction, context_pruning: e.target.checked })}
                className="w-5 h-5 rounded bg-dark-500 border-dark-400 text-claw-500 focus:ring-claw-500/50"
              />
            </div>

            {compaction.context_pruning && (
              <div className="pl-4 border-l-2 border-dark-600">
                <label className="block text-sm text-gray-400 mb-2">{t('compaction.maxMessages')}</label>
                <input
                  type="number"
                  value={compaction.max_context_messages || ''}
                  onChange={e => setCompaction({ ...compaction, max_context_messages: parseInt(e.target.value) || null })}
                  placeholder="e.g. 50"
                  className="input-base"
                />
                <p className="text-xs text-gray-500 mt-1">{t('compaction.maxMessagesDescription')}</p>
              </div>
            )}

            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">{t('compaction.localMemorySearch')}</p>
                <p className="text-xs text-gray-500">{t('compaction.localMemorySearchDescription')}</p>
              </div>
              <select
                value={memoryConfig.provider || ''}
                onChange={e => setMemoryConfig({ ...memoryConfig, provider: e.target.value || null })}
                className="input-base w-auto min-w-[120px]"
              >
                <option value="">{t('compaction.noneDisabled')}</option>
                <option value="ollama">Ollama</option>
              </select>
            </div>
          </div>
        </div>

        {/* Workspace */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center">
              <Clock size={20} className="text-orange-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('workspace.title')}</h3>
              <p className="text-xs text-gray-500">{t('workspace.description')}</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('workspace.timezone')}</label>
              <select
                value={workspace.timezone || 'Asia/Shanghai'}
                onChange={e => setWorkspace({ ...workspace, timezone: e.target.value })}
                className="input-base"
              >
                <option value="Asia/Shanghai">Asia/Shanghai</option>
                <option value="Asia/Hong_Kong">Asia/Hong_Kong</option>
                <option value="Asia/Tokyo">Asia/Tokyo</option>
                <option value="America/New_York">America/New_York</option>
                <option value="America/Los_Angeles">America/Los_Angeles</option>
                <option value="Europe/London">Europe/London</option>
                <option value="UTC">UTC</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('workspace.timeFormat')}</label>
                              <select
                                value={workspace.time_format || ''}
                                onChange={e => setWorkspace({ ...workspace, time_format: e.target.value || null })}
                                className="input-base"
                              >
                                <option value="">{t('workspace.default24h')}</option>
                                <option value="12h">{t('workspace.12hFormat')}</option>
                                <option value="24h">{t('workspace.24hFormat')}</option>
                              </select>            </div>
          </div>
        </div>

        {/* Gateway Settings */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center">
              <Server size={20} className="text-cyan-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('gateway.title')}</h3>
              <p className="text-xs text-gray-500">{t('gateway.description')}</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('gateway.port')}</label>
              <input
                type="number"
                value={gateway.port}
                onChange={e => setGateway({ ...gateway, port: parseInt(e.target.value) || 3000 })}
                className="input-base"
              />
              <p className="text-xs text-yellow-500/80 mt-1 flex items-center gap-1">
                <AlertTriangle size={12} /> {tCommon('status.requiresRestart')}
              </p>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('gateway.logLevel')}</label>
              <select
                value={gateway.log_level}
                onChange={e => setGateway({ ...gateway, log_level: e.target.value })}
                className="input-base"
              >
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warn</option>
                <option value="error">Error</option>
              </select>
            </div>
          </div>
        </div>

        {/* Subagent Defaults */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-indigo-500/20 flex items-center justify-center">
              <GitMerge size={20} className="text-indigo-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('subagent.title')}</h3>
              <p className="text-xs text-gray-500">{t('subagent.description')}</p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('subagent.maxSpawnDepth')}</label>
              <input
                type="number"
                min={0}
                max={10}
                value={subagentDefaults.max_spawn_depth ?? ''}
                onChange={e => setSubagentDefaults({ ...subagentDefaults, max_spawn_depth: e.target.value ? parseInt(e.target.value) : null })}
                className="input-base"
                placeholder="2"
              />
              <p className="text-xs text-gray-600 mt-1">{t('subagent.maxSpawnDepthDescription')}</p>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('subagent.maxChildrenPerAgent')}</label>
              <input
                type="number"
                min={0}
                max={50}
                value={subagentDefaults.max_children_per_agent ?? ''}
                onChange={e => setSubagentDefaults({ ...subagentDefaults, max_children_per_agent: e.target.value ? parseInt(e.target.value) : null })}
                className="input-base"
                placeholder="5"
              />
              <p className="text-xs text-gray-600 mt-1">{t('subagent.maxChildrenPerAgentDescription')}</p>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('subagent.maxConcurrent')}</label>
              <input
                type="number"
                min={0}
                max={100}
                value={subagentDefaults.max_concurrent ?? ''}
                onChange={e => setSubagentDefaults({ ...subagentDefaults, max_concurrent: e.target.value ? parseInt(e.target.value) : null })}
                className="input-base"
                placeholder="8"
              />
              <p className="text-xs text-gray-600 mt-1">{t('subagent.maxConcurrentDescription')}</p>
            </div>
          </div>

          <div className="mt-4 pt-4 border-t border-dark-600">
            <div className="flex items-center justify-between mb-4">
              <div>
                <p className="text-sm text-white">{t('subagent.inlineAttachments')}</p>
                <p className="text-xs text-gray-500">{t('subagent.inlineAttachmentsDescription')}</p>
              </div>
              <input
                type="checkbox"
                checked={subagentDefaults.attachments_enabled || false}
                onChange={e => setSubagentDefaults({ ...subagentDefaults, attachments_enabled: e.target.checked })}
                className="w-5 h-5 rounded bg-dark-500 border-dark-400 text-claw-500 focus:ring-claw-500/50"
              />
            </div>

            {subagentDefaults.attachments_enabled && (
              <div className="pl-4 border-l-2 border-dark-600">
                <label className="block text-sm text-gray-400 mb-2">{t('subagent.maxTotalSize')}</label>
                <input
                  type="number"
                  value={subagentDefaults.attachments_max_total_bytes || ''}
                  onChange={e => setSubagentDefaults({ ...subagentDefaults, attachments_max_total_bytes: parseInt(e.target.value) || null })}
                  placeholder="e.g. 5242880 (5MB)"
                  className="input-base"
                />
                <p className="text-xs text-gray-500 mt-1">{t('subagent.maxTotalSizeDescription')}</p>
              </div>
            )}
          </div>
        </div>

        {/* Configuration Management */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-slate-500/20 flex items-center justify-center">
              <FileJson size={20} className="text-slate-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('configManagement.title')}</h3>
              <p className="text-xs text-gray-500">{t('configManagement.description')}</p>
            </div>
          </div>

          <div className="flex gap-4">
            <button
              onClick={handleExport}
              className="flex-1 flex items-center justify-center gap-2 p-3 bg-dark-600 hover:bg-dark-500 rounded-lg transition-colors text-sm text-white border border-dark-500 hover:border-dark-400"
            >
              <Download size={16} />
              {tCommon('actions.export')} {t('configManagement.title')}
            </button>
            <button
              onClick={handleImport}
              className="flex-1 flex items-center justify-center gap-2 p-3 bg-dark-600 hover:bg-dark-500 rounded-lg transition-colors text-sm text-white border border-dark-500 hover:border-dark-400"
            >
              <Upload size={16} />
              {tCommon('actions.import')} {t('configManagement.title')}
            </button>
          </div>
          <div className="mt-4 pt-4 border-t border-dark-600">
            <button
              onClick={handleValidate}
              disabled={validating}
              className="w-full flex items-center justify-center gap-2 p-3 bg-dark-600 hover:bg-dark-500 rounded-lg transition-colors text-sm text-white border border-dark-500 hover:border-dark-400"
            >
              {validating ? <Loader2 size={16} className="animate-spin" /> : <CheckCircle size={16} />}
              {t('configManagement.validateSchema')}
            </button>
            {validateStatus && (
              <div className={`mt-3 p-3 rounded-lg text-sm border ${validateStatus.success ? 'bg-green-900/20 border-green-800/30 text-green-400' : 'bg-red-900/20 border-red-800/30 text-red-400 whitespace-pre-line'}`}>
                {validateStatus.message}
              </div>
            )}
          </div>
        </div>

        {/* Browser Control */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
              <Globe size={20} className="text-blue-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('browser.title')}</h3>
              <p className="text-xs text-gray-500">{t('browser.description')}</p>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg mb-4">
            <div>
              <p className="text-sm text-white">{t('browser.enable')}</p>
              <p className="text-xs text-gray-500">{t('browser.enableDescription')}</p>
            </div>
            <input
              type="checkbox"
              checked={browser.enabled}
              onChange={e => setBrowser({ ...browser, enabled: e.target.checked })}
              className="w-5 h-5 rounded bg-dark-500 border-dark-400 text-claw-500 focus:ring-claw-500/50"
            />
          </div>

          {browser.enabled && (
            <div className="flex items-center justify-between p-4 bg-dark-600 rounded-lg">
              <div>
                <p className="text-sm text-white">{t('browser.chromeColor')}</p>
                <p className="text-xs text-gray-500">{t('browser.chromeColorDescription')}</p>
              </div>
              <div className="flex items-center gap-3">
                <input
                  type="color"
                  value={browser.color || '#000000'}
                  onChange={e => setBrowser({ ...browser, color: e.target.value })}
                  className="w-8 h-8 rounded overflow-hidden cursor-pointer border-0 p-0"
                />
                <span className="text-sm font-mono text-gray-400">{browser.color || t('browser.default')}</span>
              </div>
            </div>
          )}
        </div>

        {/* Tools & Security */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center">
              <Server size={20} className="text-red-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('tools.title')}</h3>
              <p className="text-xs text-gray-500">{t('tools.description')}</p>
            </div>
          </div>

          <div className="space-y-6">
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('tools.securityProfile')}</label>
              <select
                value={toolsProfile}
                onChange={e => setToolsProfile(e.target.value)}
                className="input-base"
              >
                <option value="messaging">Messaging (Safest)</option>
                <option value="minimal">Minimal</option>
                <option value="coding">Coding</option>
                <option value="full">Full Access</option>
              </select>
              <p className="text-xs text-gray-500 mt-1">{t('tools.securityProfileDescription')}</p>
            </div>

            <div className="pt-4 border-t border-dark-600">
              <h4 className="text-sm font-medium text-white mb-4">{t('tools.nativePdfSupport')}</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-2">{t('tools.maxPages')}</label>
                  <input
                    type="number"
                    value={pdfConfig.max_pages || ''}
                    onChange={e => setPdfConfig({ ...pdfConfig, max_pages: parseInt(e.target.value) || null })}
                    placeholder="e.g. 10"
                    className="input-base"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-2">{t('tools.maxSizeMB')}</label>
                  <input
                    type="number"
                    value={pdfConfig.max_bytes_mb || ''}
                    onChange={e => setPdfConfig({ ...pdfConfig, max_bytes_mb: parseFloat(e.target.value) || null })}
                    placeholder="e.g. 5"
                    className="input-base"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Web Search Config */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center">
              <Globe size={20} className="text-orange-400" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-white">{t('webSearch.title')}</h3>
              <p className="text-xs text-gray-500">{t('webSearch.description')}</p>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">{t('webSearch.braveApiKey')}</label>
              <input
                type="password"
                value={webConfig.brave_api_key || ''}
                onChange={e => setWebConfig({ ...webConfig, brave_api_key: e.target.value || null })}
                placeholder="BSA-..."
                className="input-base"
              />
              <p className="text-xs text-gray-500 mt-1">{t('webSearch.braveApiKeyDescription')}</p>
            </div>
          </div>
        </div>

        {/* Manager Update */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center">
              <ArrowUpCircle size={20} className="text-emerald-400" />
            </div>
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-white">{t('managerUpdate.title')}</h3>
              <p className="text-xs text-gray-500">{t('managerUpdate.description')}</p>
            </div>
            <span className="text-xs font-mono text-gray-500 bg-dark-600 px-2 py-1 rounded">v{appVersion}</span>
          </div>

          <div className="space-y-4">
            {!managerUpdateAvailable && !managerUpdateDone && (
              <button
                onClick={checkManagerUpdate}
                disabled={managerChecking}
                className="w-full flex items-center gap-3 p-4 bg-dark-600 rounded-lg hover:bg-dark-500 transition-colors text-left disabled:opacity-50"
              >
                {managerChecking ? (
                  <Loader2 size={18} className="text-emerald-400 animate-spin" />
                ) : (
                  <RefreshCw size={18} className="text-emerald-400" />
                )}
                <div className="flex-1">
                  <p className="text-sm text-white">{managerChecking ? t('managerUpdate.checking') : t('managerUpdate.check')}</p>
                  <p className="text-xs text-gray-500">{t('managerUpdate.checkDescription')}</p>
                </div>
              </button>
            )}

            {managerCheckDone && !managerUpdateAvailable && (
              <div className="flex items-center gap-3 p-4 bg-emerald-900/20 rounded-lg border border-emerald-800/30">
                <CheckCircle size={18} className="text-emerald-400" />
                <p className="text-sm text-emerald-300">{t('managerUpdate.upToDate')}</p>
              </div>
            )}

            {managerUpdateAvailable && !managerUpdateDone && (
              <div className="p-4 bg-dark-600 rounded-lg space-y-3">
                <div className="flex items-center gap-2">
                  <Download size={16} className="text-emerald-400" />
                  <span className="text-sm font-medium text-white">{t('managerUpdate.updateAvailable', { version: managerUpdateVersion })}</span>
                </div>
                {managerUpdateBody && (
                  <p className="text-xs text-gray-400 whitespace-pre-line max-h-32 overflow-y-auto">{managerUpdateBody}</p>
                )}

                {managerDownloading && (
                  <div className="space-y-1">
                    <div className="w-full bg-dark-500 rounded-full h-2">
                      <div
                        className="bg-gradient-to-r from-emerald-500 to-claw-500 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${managerDownloadProgress}%` }}
                      />
                    </div>
                    <p className="text-xs text-gray-500 text-right">{managerDownloadProgress}%</p>
                  </div>
                )}

                {!managerDownloading && (
                  <button
                    onClick={downloadManagerUpdate}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-gradient-to-r from-emerald-600 to-claw-600 hover:from-emerald-500 hover:to-claw-500 text-white text-sm font-medium rounded-lg transition-all"
                  >
                    <Download size={16} />
                    {t('managerUpdate.downloadInstall')}
                  </button>
                )}
              </div>
            )}

            {managerUpdateDone && (
              <div className="p-4 bg-emerald-900/20 rounded-lg border border-emerald-800/30 space-y-3">
                <div className="flex items-center gap-2">
                  <CheckCircle size={16} className="text-emerald-400" />
                  <span className="text-sm font-medium text-emerald-300">{t('managerUpdate.installSuccess')}</span>
                </div>
                <button
                  onClick={restartApp}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-lg transition-colors"
                >
                  <RefreshCw size={16} />
                  {t('managerUpdate.restartNow')}
                </button>
              </div>
            )}

            {managerUpdateError && (
              <div className="flex items-start gap-2 p-3 bg-red-900/20 rounded-lg border border-red-800/30">
                <AlertTriangle size={16} className="text-red-400 mt-0.5 shrink-0" />
                <p className="text-xs text-red-300">{managerUpdateError}</p>
              </div>
            )}
          </div>
        </div>

        {/* Danger Zone */}
        <div className="bg-dark-700 rounded-2xl p-6 border border-red-900/30 opacity-80 hover:opacity-100 transition-opacity">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center">
              <AlertTriangle size={20} className="text-red-400" />
            </div>
            <h3 className="text-lg font-semibold text-white">{t('dangerZone.title')}</h3>
          </div>
          <button
            onClick={() => setShowUninstallConfirm(true)}
            className="w-full flex items-center gap-3 p-3 bg-red-950/30 rounded-lg hover:bg-red-900/40 transition-colors text-left border border-red-900/30"
          >
            <Trash2 size={18} className="text-red-400" />
            <div className="flex-1">
              <p className="text-sm text-red-300">{tCommon('actions.uninstall')} OpenClaw</p>
            </div>
          </button>
        </div>

        {/* Global Save Button (Floating) */}
        <div className="fixed bottom-4 right-4 z-40">
          <button
            onClick={handleSave}
            disabled={saving || saveSuccess}
            className={`shadow-lg flex items-center gap-1.5 px-4 py-2 rounded-full text-sm font-medium transition-all ${saveSuccess
              ? 'bg-green-600 text-white hover:bg-green-700'
              : 'btn-primary'
              }`}
          >
            {saving ? (
              <Loader2 size={16} className="animate-spin" />
            ) : saveSuccess ? (
              <CheckCircle size={16} />
            ) : (
              <Save size={16} />
            )}
            {saveSuccess ? tCommon('status.success') + '!' : tCommon('actions.save') + ' ' + t('title')}
          </button>
        </div>
      </div>

      {/* Uninstall Modal */}
      {showUninstallConfirm && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500 max-w-md w-full shadow-2xl">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold text-white">{tCommon('actions.uninstall')} OpenClaw</h3>
              <button onClick={() => setShowUninstallConfirm(false)}><X size={20} className="text-gray-400 hover:text-white" /></button>
            </div>

            {!uninstallResult ? (
              <>
                <p className="text-gray-300 mb-4">{t('dangerZone.uninstallConfirm')} <span className="text-red-400 font-semibold">{t('dangerZone.permanentlyDelete')}</span> {t('dangerZone.entireFolder')}</p>
                <p className="text-yellow-400/80 text-xs mb-6 flex items-center gap-2"><AlertTriangle size={14} /> {t('dangerZone.cannotBeUndone')}</p>
                <div className="flex gap-3">
                  <button onClick={() => setShowUninstallConfirm(false)} className="flex-1 btn-secondary">{tCommon('actions.cancel')}</button>
                  <button onClick={handleUninstall} disabled={uninstalling} className="flex-1 btn-primary bg-red-600 hover:bg-red-500 flex justify-center gap-2">
                    {uninstalling ? <Loader2 className="animate-spin" size={16} /> : <Trash2 size={16} />}
                    {tCommon('actions.uninstall')}
                  </button>
                </div>
              </>
            ) : (
              <div className={`p-4 rounded-lg bg-${uninstallResult.success ? 'green' : 'red'}-900/30 border border-${uninstallResult.success ? 'green' : 'red'}-800`}>
                <p className={`text-${uninstallResult.success ? 'green' : 'red'}-300 text-sm`}>{uninstallResult.message}</p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}