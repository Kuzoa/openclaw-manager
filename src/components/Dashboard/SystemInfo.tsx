import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';
import {
  CheckCircle2,
  Loader2,
  Download,
  ExternalLink,
  Cpu,
  GitBranch,
  Package,
  Shield,
  RefreshCw,
  Server,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '../../stores/appStore';

interface InstallResult {
  success: boolean;
  message: string;
  error: string | null;
}

interface Requirement {
  id: string;
  name: string;
  description: string;
  icon: React.ReactNode;
  installed: boolean;
  version: string | null;
  versionOk?: boolean;
  versionNote?: string;
  installAction?: () => void;
  downloadUrl?: string;
  canAutoInstall: boolean;
}

export function SystemInfo() {
  const { t } = useTranslation('dashboard');
  
  // Get environment state from store
  const environment = useAppStore((state) => state.environment);
  const isCheckingEnvironment = useAppStore((state) => state.isCheckingEnvironment);
  const environmentError = useAppStore((state) => state.environmentError);
  const refreshEnvironment = useAppStore((state) => state.refreshEnvironment);
  
  const [installing, setInstalling] = useState<string | null>(null);
  const [localError, setLocalError] = useState<string | null>(null);

  const handleRefresh = async () => {
    await refreshEnvironment();
  };

  const handleInstallNodejs = async () => {
    setInstalling('nodejs');
    setLocalError(null);
    try {
      const result = await invoke<InstallResult>('install_nodejs');
      if (result.success) {
        await refreshEnvironment();
      } else {
        setLocalError(result.error || result.message);
      }
    } catch (e) {
      setLocalError(`${t('systemInfo.installFailed', { name: 'Node.js' })}: ${e}`);
    } finally {
      setInstalling(null);
    }
  };

  const handleInstallOpenclaw = async () => {
    setInstalling('openclaw');
    setLocalError(null);
    try {
      const result = await invoke<InstallResult>('install_openclaw');
      if (result.success) {
        await invoke<InstallResult>('init_openclaw_config');
        await refreshEnvironment();
      } else {
        setLocalError(result.error || result.message);
      }
    } catch (e) {
      setLocalError(`${t('systemInfo.installFailed', { name: 'OpenClaw' })}: ${e}`);
    } finally {
      setInstalling(null);
    }
  };

  const handleInstallGateway = async () => {
    setInstalling('gateway');
    setLocalError(null);
    try {
      await invoke<string>('install_gateway_service');
      // Gateway install opens an elevated terminal — user needs to complete it there
      // Don't auto-refresh; user clicks Refresh when done
    } catch (e) {
      setLocalError(`${t('systemInfo.installFailed', { name: t('systemInfo.requirements.gateway.name') })}: ${e}`);
    } finally {
      setInstalling(null);
    }
  };

  const handleOpenUrl = async (url: string) => {
    try {
      await open(url);
    } catch {
      window.open(url, '_blank');
    }
  };

  // Display loading state
  if (isCheckingEnvironment && !environment) {
    return (
      <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
        <h3 className="text-lg font-semibold text-white mb-4">{t('systemInfo.title')}</h3>
        <div className="flex items-center justify-center py-8">
          <Loader2 className="w-8 h-8 text-claw-400 animate-spin" />
        </div>
      </div>
    );
  }

  if (!environment) {
    return (
      <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
        <h3 className="text-lg font-semibold text-white mb-4">{t('systemInfo.title')}</h3>
        <p className="text-gray-400 text-sm">{t('systemInfo.detectError')}</p>
        {environmentError && (
          <p className="text-red-400 text-xs mt-2">{environmentError}</p>
        )}
      </div>
    );
  }

  const requirements: Requirement[] = [
    {
      id: 'nodejs',
      name: t('systemInfo.requirements.nodejs.name'),
      description: t('systemInfo.requirements.nodejs.description'),
      icon: <Cpu size={18} />,
      installed: environment.node_installed && environment.node_version_ok,
      version: environment.node_version,
      versionOk: environment.node_version_ok,
      versionNote: environment.node_installed && !environment.node_version_ok
        ? t('systemInfo.requirements.nodejs.versionNote')
        : undefined,
      installAction: handleInstallNodejs,
      downloadUrl: 'https://nodejs.org/en/download',
      canAutoInstall: true,
    },
    {
      id: 'git',
      name: t('systemInfo.requirements.git.name'),
      description: t('systemInfo.requirements.git.description'),
      icon: <GitBranch size={18} />,
      installed: environment.git_installed,
      version: environment.git_version,
      downloadUrl: 'https://git-scm.com/downloads',
      canAutoInstall: false,
    },
    {
      id: 'openclaw',
      name: t('systemInfo.requirements.openclaw.name'),
      description: t('systemInfo.requirements.openclaw.description'),
      icon: <Package size={18} />,
      installed: environment.openclaw_installed,
      version: environment.openclaw_version,
      installAction: handleInstallOpenclaw,
      canAutoInstall: true,
    },
    ...(environment.openclaw_installed ? [{
      id: 'gateway',
      name: t('systemInfo.requirements.gateway.name'),
      description: t('systemInfo.requirements.gateway.description'),
      icon: <Server size={18} />,
      installed: environment.gateway_service_installed,
      version: null,
      installAction: handleInstallGateway,
      canAutoInstall: true,
    }] : []),
  ];

  const installedCount = requirements.filter(r => r.installed).length;
  const totalCount = requirements.length;
  const allReady = installedCount === totalCount;
  const progressPercent = Math.round((installedCount / totalCount) * 100);

  return (
    <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
      {/* Header */}
      <div className="flex items-center justify-between mb-5">
        <div className="flex items-center gap-3">
          <div className={`w-9 h-9 rounded-lg flex items-center justify-center ${allReady ? 'bg-green-500/20' : 'bg-amber-500/20'
            }`}>
            <Shield size={18} className={allReady ? 'text-green-400' : 'text-amber-400'} />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white">{t('systemInfo.title')}</h3>
            <p className="text-xs text-gray-500">
              {allReady
                ? t('systemInfo.allReady')
                : t('systemInfo.progress', { installed: installedCount, total: totalCount })}
            </p>
          </div>
        </div>
        <button
          onClick={handleRefresh}
          disabled={isCheckingEnvironment}
          className="p-2 text-gray-400 hover:text-white hover:bg-dark-600 rounded-lg transition-colors"
          title={t('systemInfo.refresh')}
        >
          <RefreshCw size={16} className={isCheckingEnvironment ? 'animate-spin' : ''} />
        </button>
      </div>

      {/* Progress bar */}
      <div className="mb-5">
        <div className="w-full h-1.5 bg-dark-600 rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full transition-all duration-500 ${allReady
              ? 'bg-green-500'
              : progressPercent > 50
                ? 'bg-amber-500'
                : 'bg-red-500'
              }`}
            style={{ width: `${progressPercent}%` }}
          />
        </div>
      </div>

      {/* Requirements list */}
      <div className="space-y-3">
        {requirements.map((req) => (
          <div
            key={req.id}
            className={`flex items-center justify-between p-3 rounded-xl border transition-colors ${req.installed
              ? 'bg-green-500/5 border-green-500/10'
              : 'bg-red-500/5 border-red-500/15'
              }`}
          >
            <div className="flex items-center gap-3">
              <div className={`w-8 h-8 rounded-lg flex items-center justify-center ${req.installed ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'
                }`}>
                {req.installed ? <CheckCircle2 size={16} /> : req.icon}
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-white">{req.name}</span>
                  {req.installed && req.version && (
                    <span className="text-xs text-gray-500 font-mono">{req.version}</span>
                  )}
                </div>
                <p className="text-xs text-gray-500">
                  {req.versionNote || req.description}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-2">
              {req.installed ? (
                <span className="text-xs text-green-400 font-medium px-2 py-1 bg-green-500/10 rounded-md">
                  {t('systemInfo.ready')}
                </span>
              ) : (
                <>
                  {req.canAutoInstall && req.installAction && (
                    <button
                      onClick={req.installAction}
                      disabled={installing !== null}
                      className="flex items-center gap-1.5 px-3 py-1.5 bg-claw-600 hover:bg-claw-700 text-white rounded-lg transition-colors text-xs font-medium disabled:opacity-50"
                    >
                      {installing === req.id ? (
                        <>
                          <Loader2 size={12} className="animate-spin" />
                          <span>{t('systemInfo.installing')}</span>
                        </>
                      ) : (
                        <>
                          <Download size={12} />
                          <span>{t('systemInfo.install')}</span>
                        </>
                      )}
                    </button>
                  )}
                  {req.downloadUrl && (
                    <button
                      onClick={() => handleOpenUrl(req.downloadUrl!)}
                      className="flex items-center gap-1.5 px-3 py-1.5 text-gray-300 hover:text-white hover:bg-dark-500 rounded-lg transition-colors text-xs"
                      title={`${t('systemInfo.download')} ${req.name}`}
                    >
                      <ExternalLink size={12} />
                      <span>{t('systemInfo.download')}</span>
                    </button>
                  )}
                </>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Error */}
      {(localError || environmentError) && (
        <div className="mt-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
          <p className="text-red-400 text-xs">{localError || environmentError}</p>
        </div>
      )}
    </div>
  );
}