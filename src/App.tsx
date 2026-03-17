import React, { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { Sidebar } from './components/Layout/Sidebar';
import { Header } from './components/Layout/Header';
import { useTranslation, Trans } from 'react-i18next';

import { appLogger } from './lib/logger';
import { isTauri } from './lib/tauri';
import { Download, X, Loader2, CheckCircle, AlertCircle } from 'lucide-react';
import { useAppStore } from './stores/appStore';
import { useService } from './hooks/useService';

// Lazy loaded page components
const Dashboard = React.lazy(() => import('./components/Dashboard').then(module => ({ default: module.Dashboard })));
const AIConfig = React.lazy(() => import('./components/AIConfig').then(module => ({ default: module.AIConfig })));
const Channels = React.lazy(() => import('./components/Channels').then(module => ({ default: module.Channels })));
const MCP = React.lazy(() => import('./components/MCP').then(module => ({ default: module.MCP })));
const Skills = React.lazy(() => import('./components/Skills').then(module => ({ default: module.Skills })));
const Settings = React.lazy(() => import('./components/Settings').then(module => ({ default: module.Settings })));
const Logs = React.lazy(() => import('./components/Logs').then(module => ({ default: module.Logs })));
const Agents = React.lazy(() => import('./components/Agents').then(module => ({ default: module.Agents })));

export type PageType = 'dashboard' | 'mcp' | 'skills' | 'ai' | 'channels' | 'agents' | 'logs' | 'settings';

interface UpdateInfo {
  update_available: boolean;
  current_version: string | null;
  latest_version: string | null;
  error: string | null;
}

interface UpdateResult {
  success: boolean;
  message: string;
  error?: string;
}

// Error Boundary Component
function ErrorBoundary({ children }: { children: React.ReactNode }) {
  const { t } = useTranslation('app');
  const [hasError, setHasError] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const handleError = (event: ErrorEvent) => {
      setHasError(true);
      setError(event.error);
      appLogger.error('ErrorBoundary caught error', { error: event.error });
    };

    window.addEventListener('error', handleError);
    return () => window.removeEventListener('error', handleError);
  }, []);

  if (hasError) {
    return (
      <div className="p-8 text-center">
        <AlertCircle size={48} className="mx-auto text-red-400 mb-4" />
        <h2 className="text-xl font-bold text-white mb-2">{t('errorBoundary.title')}</h2>
        <p className="text-red-200 mb-4">{error?.message}</p>
        <button
          onClick={() => setHasError(false)}
          className="px-4 py-2 bg-dark-700 hover:bg-dark-600 rounded-lg text-white text-sm"
        >
          {t('errorBoundary.tryAgain')}
        </button>
      </div>
    );
  }

  return <>{children}</>;
}

function App() {
  const { t: tApp } = useTranslation('app');
  const [currentPage, setCurrentPage] = useState<PageType>('dashboard');

  // Initialize service status polling (useService manages serviceStatus in store)
  useService();

  // Get state from store
  const environment = useAppStore((state) => state.environment);
  const serviceStatus = useAppStore((state) => state.serviceStatus);
  const checkEnvironment = useAppStore((state) => state.checkEnvironment);
  const refreshEnvironment = useAppStore((state) => state.refreshEnvironment);
  const notifications = useAppStore((state) => state.notifications);
  const removeNotification = useAppStore((state) => state.removeNotification);

  // Update related state
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateBanner, setShowUpdateBanner] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [updateResult, setUpdateResult] = useState<UpdateResult | null>(null);

  // Manager Update state
  const [managerUpdateAvailable, setManagerUpdateAvailable] = useState(false);
  const [managerUpdateVersion, setManagerUpdateVersion] = useState<string | null>(null);
  const [showManagerUpdateBanner, setShowManagerUpdateBanner] = useState(false);
  const [managerUpdating, setManagerUpdating] = useState(false);
  const [managerUpdateProgress, setManagerUpdateProgress] = useState(0);
  const [managerUpdateResult, setManagerUpdateResult] = useState<UpdateResult | null>(null);
  const [managerUpdateObj, setManagerUpdateObj] = useState<any>(null);

  // Security check state - derived from environment.is_secure
  const showSecurityBanner = environment?.is_secure === false;

  // Check for updates
  const checkUpdate = useCallback(async () => {
    if (!isTauri()) return;

    appLogger.info('Checking for OpenClaw updates...');
    try {
      const info = await invoke<UpdateInfo>('check_openclaw_update');
      appLogger.info('Update check result', info);
      setUpdateInfo(info);
      if (info.update_available) {
        setShowUpdateBanner(true);
      }
    } catch (e) {
      appLogger.error('Update check failed', e);
    }
  }, []);

  // Check Manager Update
  const checkManagerUpdate = useCallback(async () => {
    if (!isTauri()) return;
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        setManagerUpdateAvailable(true);
        setManagerUpdateVersion(update.version);
        setManagerUpdateObj(update);
        setShowManagerUpdateBanner(true);
      }
    } catch (e) {
      appLogger.error('Manager update check failed', e);
    }
  }, []);

  // Perform update
  const handleUpdate = async () => {
    setUpdating(true);
    setUpdateResult(null);
    try {
      const result = await invoke<UpdateResult>('update_openclaw');
      setUpdateResult(result);
      if (result.success) {
        // Refresh environment after successful update (invalidates cache and re-checks)
        await refreshEnvironment();
        // Close notification after 3 seconds
        setTimeout(() => {
          setShowUpdateBanner(false);
          setUpdateResult(null);
        }, 3000);
      }
    } catch (e) {
      setUpdateResult({
        success: false,
        message: 'Error occurred during update',
        error: String(e),
      });
    } finally {
      setUpdating(false);
    }
  };

  // Perform Manager Update (from banner)
  const handleManagerUpdate = async () => {
    if (!managerUpdateObj) return;
    setManagerUpdating(true);
    setManagerUpdateProgress(0);
    setManagerUpdateResult(null);
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
            setManagerUpdateProgress(Math.min(100, Math.round((downloaded / contentLength) * 100)));
            break;
          case 'Finished':
            setManagerUpdateProgress(100);
            break;
        }
      });
      setManagerUpdateResult({ success: true, message: tApp('managerUpdate.success') });

      // Restart app after 2 seconds
      setTimeout(async () => {
        try {
          const { relaunch } = await import('@tauri-apps/plugin-process');
          await relaunch();
        } catch (err) {
          appLogger.error('Relaunch failed', err);
        }
      }, 2000);
    } catch (e: any) {
      appLogger.error('Manager update download failed', e);
      setManagerUpdateResult({ success: false, message: tApp('managerUpdate.failed'), error: e?.message || String(e) });
      setManagerUpdating(false);
    }
  };

  useEffect(() => {
    appLogger.info('🦞 App component mounted');
    checkEnvironment();
  }, [checkEnvironment]);

  // Check for updates after environment check completes (non-blocking)
  useEffect(() => {
    if (!isTauri() || !environment) return;

    // Environment check completed, delay update checks to avoid blocking
    appLogger.info('Environment check completed, scheduling update checks...');
    const timer1 = setTimeout(() => { checkUpdate(); }, 3000);
    const timer2 = setTimeout(() => { checkManagerUpdate(); }, 5000);
    return () => { clearTimeout(timer1); clearTimeout(timer2); };
  }, [environment, checkUpdate, checkManagerUpdate]);

  // Service status polling is handled by useService hook in Dashboard

  // Auto-remove notifications after 3 seconds
  useEffect(() => {
    if (notifications.length > 0) {
      const timer = setTimeout(() => {
        const oldestNotification = notifications[0];
        if (oldestNotification) {
          removeNotification(oldestNotification.id);
        }
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [notifications, removeNotification]);

  const handleSetupComplete = useCallback(() => {
    appLogger.info('Setup wizard completed');
    refreshEnvironment(); // Refresh environment after setup
  }, [refreshEnvironment]);

  // Page navigation handler
  const handleNavigate = (page: PageType) => {
    appLogger.action('Page navigation', { from: currentPage, to: page });
    setCurrentPage(page);
  };

  const renderPage = () => {
    const pageVariants = {
      initial: { opacity: 0, x: 20 },
      animate: { opacity: 1, x: 0 },
      exit: { opacity: 0, x: -20 },
    };

    const pages: Record<PageType, JSX.Element> = {
      dashboard: <Dashboard environment={environment} onSetupComplete={handleSetupComplete} />,
      mcp: <MCP />,
      skills: <Skills />,
      ai: <AIConfig />,
      channels: <Channels />,
      agents: <Agents />,
      logs: <Logs />,
      settings: <Settings onEnvironmentChange={refreshEnvironment} />,
    };

    return (
      <AnimatePresence mode="wait">
        <motion.div
          key={currentPage}
          variants={pageVariants}
          initial="initial"
          animate="animate"
          exit="exit"
          transition={{ duration: 0.2 }}
          className="h-full"
        >
          {pages[currentPage]}
        </motion.div>
      </AnimatePresence>
    );
  };

  const LoadingSpinner = () => {
    const { t } = useTranslation('app');
    return (
      <div className="flex h-full items-center justify-center">
        <div className="relative z-10 text-center">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-xl bg-gradient-to-br from-brand-500 to-purple-600 mb-4 animate-pulse shadow-lg shadow-purple-900/20">
            <span className="text-3xl">🦞</span>
          </div>
          <p className="text-dark-400 font-medium">{t('loading.component')}</p>
        </div>
      </div>
    );
  };

  // Main interface
  return (
    <div className="flex h-screen bg-dark-900 overflow-hidden">
      {/* Background decoration */}
      <div className="fixed inset-0 bg-gradient-radial pointer-events-none" />

      {/* Security Banner (High Priority) */}
      <AnimatePresence>
        {showSecurityBanner && environment && (
          <motion.div
            initial={{ opacity: 0, y: -50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -50 }}
            className="fixed top-0 left-0 right-0 z-[60] bg-gradient-to-r from-red-600 to-orange-600 shadow-lg"
          >
            <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <AlertCircle size={20} className="text-white" />
                <div>
                  <p className="text-sm font-bold text-white">
                    <Trans t={tApp} i18nKey="securityBanner.title" values={{ version: environment.openclaw_version }}>
                      Security Warning: Your OpenClaw version is insecure.
                    </Trans>
                  </p>
                  <p className="text-xs text-white/90">
                    {tApp('securityBanner.warning')}
                  </p>
                </div>
              </div>
              <button
                onClick={() => setShowUpdateBanner(true)}
                className="px-4 py-1.5 bg-white/20 hover:bg-white/30 text-white text-sm font-medium rounded-lg transition-colors"
              >
                {tApp('securityBanner.updateNow')}
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Update banner */}
      <AnimatePresence>
        {showUpdateBanner && updateInfo?.update_available && (
          <motion.div
            initial={{ opacity: 0, y: -50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -50 }}
            className="fixed top-0 left-0 right-0 z-50 bg-gradient-to-r from-claw-600 to-purple-600 shadow-lg"
          >
            <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-3">
                {updateResult?.success ? (
                  <CheckCircle size={20} className="text-green-300" />
                ) : updateResult && !updateResult.success ? (
                  <AlertCircle size={20} className="text-red-300" />
                ) : (
                  <Download size={20} className="text-white" />
                )}
                <div>
                  {updateResult ? (
                    <p className={`text-sm font-medium ${updateResult.success ? 'text-green-100' : 'text-red-100'}`}>
                      {updateResult.message}
                    </p>
                  ) : (
                    <>
                      <p className="text-sm font-medium text-white">
                        <Trans t={tApp} i18nKey="updateBanner.title" values={{ version: updateInfo.latest_version }}>
                          New version available: OpenClaw {{ version: updateInfo.latest_version }}
                        </Trans>
                      </p>
                      <p className="text-xs text-white/70">
                        <Trans t={tApp} i18nKey="updateBanner.currentVersion" values={{ version: updateInfo.current_version }}>
                          Current version: {{ version: updateInfo.current_version }}
                        </Trans>
                      </p>
                    </>
                  )}
                </div>
              </div>

              <div className="flex items-center gap-2">
                {!updateResult && (
                  <button
                    onClick={handleUpdate}
                    disabled={updating}
                    className="px-4 py-1.5 bg-white/20 hover:bg-white/30 text-white text-sm font-medium rounded-lg transition-colors flex items-center gap-2 disabled:opacity-50"
                  >
                    {updating ? (
                      <>
                        <Loader2 size={14} className="animate-spin" />
                        {tApp('updateBanner.updating')}
                      </>
                    ) : (
                      <>
                        <Download size={14} />
                        {tApp('updateBanner.updateNow')}
                      </>
                    )}
                  </button>
                )}
                <button
                  onClick={() => {
                    setShowUpdateBanner(false);
                    setUpdateResult(null);
                  }}
                  className="p-1.5 hover:bg-white/20 rounded-lg transition-colors text-white/70 hover:text-white"
                >
                  <X size={16} />
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Manager update banner */}
      <AnimatePresence>
        {showManagerUpdateBanner && managerUpdateAvailable && (
          <motion.div
            initial={{ opacity: 0, y: -50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -50 }}
            className="fixed top-0 left-0 right-0 z-[45] bg-gradient-to-r from-emerald-600 to-teal-600 shadow-lg"
          >
            <div className="max-w-4xl mx-auto px-4 py-3 flex items-center justify-between">
              <div className="flex items-center gap-3 w-1/2">
                {managerUpdateResult?.success ? (
                  <CheckCircle size={20} className="text-green-300 shrink-0" />
                ) : managerUpdateResult && !managerUpdateResult.success ? (
                  <AlertCircle size={20} className="text-red-300 shrink-0" />
                ) : (
                  <Download size={20} className="text-white shrink-0" />
                )}
                <div className="flex-1">
                  {managerUpdateResult ? (
                    <p className={`text-sm font-medium ${managerUpdateResult.success ? 'text-green-100' : 'text-red-100'}`}>
                      {managerUpdateResult.message}
                    </p>
                  ) : (
                    <>
                      <div className="flex justify-between items-center pr-4">
                        <p className="text-sm font-medium text-white">
                          <Trans t={tApp} i18nKey="managerUpdate.title" values={{ version: managerUpdateVersion }}>
                            New version available: Manager v{{ version: managerUpdateVersion }}
                          </Trans>
                        </p>
                        {managerUpdating && (
                          <span className="text-xs text-white/80">{managerUpdateProgress}%</span>
                        )}
                      </div>
                      {managerUpdating && (
                        <div className="w-full bg-black/20 rounded-full h-1 mt-1.5 mr-4 max-w-[200px]">
                          <div
                            className="bg-white h-1 rounded-full transition-all duration-300"
                            style={{ width: `${managerUpdateProgress}%` }}
                          />
                        </div>
                      )}
                    </>
                  )}
                </div>
              </div>

              <div className="flex items-center gap-2">
                {!managerUpdateResult && (
                  <button
                    onClick={handleManagerUpdate}
                    disabled={managerUpdating}
                    className="px-4 py-1.5 bg-white/20 hover:bg-white/30 text-white text-sm font-medium rounded-lg transition-colors flex items-center gap-2 disabled:opacity-50"
                  >
                    {managerUpdating ? (
                      <>
                        <Loader2 size={14} className="animate-spin" />
                        {tApp('updateBanner.updating')}
                      </>
                    ) : (
                      <>
                        <Download size={14} />
                        {tApp('updateBanner.updateNow')}
                      </>
                    )}
                  </button>
                )}
                <button
                  onClick={() => {
                    setShowManagerUpdateBanner(false);
                    setManagerUpdateResult(null);
                  }}
                  className="p-1.5 hover:bg-white/20 rounded-lg transition-colors text-white/70 hover:text-white"
                >
                  <X size={16} />
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Sidebar */}
      <Sidebar currentPage={currentPage} onNavigate={handleNavigate} serviceStatus={serviceStatus} />

      {/* Main content area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header (macOS drag area) */}
        <Header currentPage={currentPage} />

        {/* Page content */}
        <main className="flex-1 overflow-hidden p-6 relative">
          <ErrorBoundary>
            <React.Suspense fallback={<LoadingSpinner />}>
              {renderPage()}
            </React.Suspense>
          </ErrorBoundary>
        </main>
      </div>

      {/* Notification toasts */}
      <div className="fixed top-4 right-4 z-50 flex flex-col gap-2">
        <AnimatePresence>
          {notifications.map((notification) => (
            <motion.div
              key={notification.id}
              initial={{ opacity: 0, x: 100 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 100 }}
              className={`px-4 py-3 rounded-lg shadow-lg flex items-center gap-3 min-w-[250px] ${
                notification.type === 'success' ? 'bg-green-600' :
                notification.type === 'error' ? 'bg-red-600' :
                notification.type === 'warning' ? 'bg-yellow-600' :
                'bg-blue-600'
              }`}
            >
              {notification.type === 'success' && <CheckCircle size={18} className="text-white" />}
              {notification.type === 'error' && <AlertCircle size={18} className="text-white" />}
              <div className="flex-1">
                <p className="text-sm font-medium text-white">{notification.title}</p>
                {notification.message && (
                  <p className="text-xs text-white/80 mt-0.5">{notification.message}</p>
                )}
              </div>
              <button
                onClick={() => removeNotification(notification.id)}
                className="p-1 hover:bg-white/20 rounded transition-colors text-white/70 hover:text-white"
              >
                <X size={14} />
              </button>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </div>
  );
}

export default App;
