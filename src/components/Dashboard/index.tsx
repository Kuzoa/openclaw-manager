import { useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { StatusCard } from './StatusCard';
import { QuickActions } from './QuickActions';
import { SystemInfo } from './SystemInfo';
import { Setup } from '../Setup';
import { isTauri } from '../../lib/tauri';
import { useAppStore } from '../../stores/appStore';
import type { EnvironmentStatus } from '../../types';

interface DashboardProps {
  environment: EnvironmentStatus | null;
  onSetupComplete: () => void;
}

export function Dashboard({ environment, onSetupComplete }: DashboardProps) {
  // Read service status from store (polling is handled by useService in App.tsx)
  const serviceStatus = useAppStore((state) => state.serviceStatus);
  const [actionLoading, setActionLoading] = useState(false);

  const handleStart = async () => {
    if (!isTauri()) return;
    setActionLoading(true);
    try {
      await invoke('start_service');
    } catch (e) {
      console.error('Start failed:', e);
    } finally {
      setActionLoading(false);
    }
  };

  const handleStop = async () => {
    if (!isTauri()) return;
    setActionLoading(true);
    try {
      await invoke('stop_service');
    } catch (e) {
      console.error('Stop failed:', e);
    } finally {
      setActionLoading(false);
    }
  };

  const handleRestart = async () => {
    if (!isTauri()) return;
    setActionLoading(true);
    try {
      await invoke('restart_service');
    } catch (e) {
      console.error('Restart failed:', e);
    } finally {
      setActionLoading(false);
    }
  };

  const handleKillAll = async () => {
    if (!isTauri()) return;
    setActionLoading(true);
    try {
      await invoke<string>('kill_all_port_processes');
    } catch (e) {
      console.error('Kill All failed:', e);
    } finally {
      setActionLoading(false);
    }
  };

  const containerVariants = {
    hidden: { opacity: 0 },
    show: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
      },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    show: { opacity: 1, y: 0 },
  };

  // Check if environment is ready
  const needsSetup = environment && !environment.ready;

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2">
      <motion.div
        variants={containerVariants}
        initial="hidden"
        animate="show"
        className="space-y-6"
      >
        {/* Environment setup wizard (only shown when needed) */}
        {needsSetup && (
          <motion.div variants={itemVariants}>
            <Setup onComplete={onSetupComplete} embedded />
          </motion.div>
        )}

        {/* Service status card */}
        <motion.div variants={itemVariants}>
          <StatusCard status={serviceStatus} loading={false} />
        </motion.div>

        {/* Quick actions */}
        <motion.div variants={itemVariants}>
          <QuickActions
            status={serviceStatus}
            loading={actionLoading}
            onStart={handleStart}
            onStop={handleStop}
            onRestart={handleRestart}
            onKillAll={handleKillAll}
          />
        </motion.div>

        {/* System info */}
        <motion.div variants={itemVariants}>
          <SystemInfo />
        </motion.div>
      </motion.div>
    </div>
  );
}
