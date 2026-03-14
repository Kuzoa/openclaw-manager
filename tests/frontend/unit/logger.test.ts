import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createLogger, logStore, setLogLevel } from '@/lib/logger';

describe('Logger', () => {
  let consoleLogSpy: ReturnType<typeof vi.spyOn>;
  let consoleWarnSpy: ReturnType<typeof vi.spyOn>;
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    // Clear log store before each test
    logStore.clear();
    
    // Reset localStorage mock and set default log level
    localStorage.clear();
    localStorage.setItem('LOG_LEVEL', 'debug');
    
    // Setup console spies
    consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    consoleLogSpy.mockRestore();
    consoleWarnSpy.mockRestore();
    consoleErrorSpy.mockRestore();
  });

  describe('createLogger', () => {
    it('should return a Logger instance with the correct module name', () => {
      const logger = createLogger('TestModule');
      expect(logger).toBeDefined();
      expect(logger.debug).toBeTypeOf('function');
      expect(logger.info).toBeTypeOf('function');
      expect(logger.warn).toBeTypeOf('function');
      expect(logger.error).toBeTypeOf('function');
    });
  });

  describe('Logger methods', () => {
    it('should call console.log for debug level', () => {
      const logger = createLogger('TestModule');
      logger.debug('test message');
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it('should call console.log for info level', () => {
      const logger = createLogger('TestModule');
      logger.info('test message');
      expect(consoleLogSpy).toHaveBeenCalled();
    });

    it('should call console.warn for warn level', () => {
      const logger = createLogger('TestModule');
      logger.warn('test warning');
      expect(consoleWarnSpy).toHaveBeenCalled();
    });

    it('should call console.error for error level', () => {
      const logger = createLogger('TestModule');
      logger.error('test error');
      expect(consoleErrorSpy).toHaveBeenCalled();
    });
  });

  describe('logStore', () => {
    it('should add log entry with generated id', () => {
      const initialLogs = logStore.getAll().length;
      
      logStore.add({
        timestamp: new Date(),
        level: 'info',
        module: 'Test',
        message: 'test message',
        args: [],
      });

      const logs = logStore.getAll();
      expect(logs).toHaveLength(initialLogs + 1);
      // id should be a positive number
      expect(logs[logs.length - 1].id).toBeGreaterThan(0);
      expect(logs[logs.length - 1].message).toBe('test message');
    });

    it('should return a copy of all logs', () => {
      logStore.add({
        timestamp: new Date(),
        level: 'info',
        module: 'Test',
        message: 'message 1',
        args: [],
      });

      const logs1 = logStore.getAll();
      const logs2 = logStore.getAll();
      
      expect(logs1).not.toBe(logs2); // Different references
      expect(logs1).toEqual(logs2); // Same content
    });

    it('should clear all logs', () => {
      logStore.add({
        timestamp: new Date(),
        level: 'info',
        module: 'Test',
        message: 'message',
        args: [],
      });

      expect(logStore.getAll().length).toBeGreaterThan(0);
      logStore.clear();
      expect(logStore.getAll()).toHaveLength(0);
    });

    it('should notify subscribers on add', () => {
      const listener = vi.fn();
      logStore.subscribe(listener);
      
      logStore.add({
        timestamp: new Date(),
        level: 'info',
        module: 'Test',
        message: 'test',
        args: [],
      });

      expect(listener).toHaveBeenCalled();
    });

    it('should notify subscribers on clear', () => {
      const listener = vi.fn();
      logStore.subscribe(listener);
      
      logStore.clear();
      expect(listener).toHaveBeenCalled();
    });

    it('should unsubscribe correctly', () => {
      const listener = vi.fn();
      const unsubscribe = logStore.subscribe(listener);
      
      unsubscribe();
      logStore.add({
        timestamp: new Date(),
        level: 'info',
        module: 'Test',
        message: 'test',
        args: [],
      });

      expect(listener).not.toHaveBeenCalled();
    });

    it('should trim logs when exceeding maxLogs (500)', () => {
      // Clear first to start fresh
      logStore.clear();
      
      // Add 502 logs
      for (let i = 0; i < 502; i++) {
        logStore.add({
          timestamp: new Date(),
          level: 'info',
          module: 'Test',
          message: `message ${i}`,
          args: [],
        });
      }

      const logs = logStore.getAll();
      expect(logs).toHaveLength(500);
      // Should keep the most recent logs
      expect(logs[0].message).toBe('message 2'); // First kept message
      expect(logs[499].message).toBe('message 501'); // Last message
    });
  });

  describe('Log level filtering', () => {
    it('should log all levels when set to debug', () => {
      // setLogLevel also calls console.log, so we need to reset spy after calling it
      setLogLevel('debug');
      consoleLogSpy.mockClear(); // Clear the setLogLevel console.log call
      
      const logger = createLogger('Test');

      logger.debug('debug msg');
      logger.info('info msg');
      logger.warn('warn msg');
      logger.error('error msg');

      expect(consoleLogSpy).toHaveBeenCalledTimes(2); // debug + info
      expect(consoleWarnSpy).toHaveBeenCalledTimes(1);
      expect(consoleErrorSpy).toHaveBeenCalledTimes(1);
    });

    it('should not log debug when set to info', () => {
      setLogLevel('info');
      consoleLogSpy.mockClear();
      
      const logger = createLogger('Test');

      logger.debug('debug msg');
      logger.info('info msg');
      logger.warn('warn msg');
      logger.error('error msg');

      expect(consoleLogSpy).toHaveBeenCalledTimes(1); // only info
      expect(consoleWarnSpy).toHaveBeenCalledTimes(1);
      expect(consoleErrorSpy).toHaveBeenCalledTimes(1);
    });

    it('should not log debug/info when set to warn', () => {
      setLogLevel('warn');
      consoleLogSpy.mockClear();
      
      const logger = createLogger('Test');

      logger.debug('debug msg');
      logger.info('info msg');
      logger.warn('warn msg');
      logger.error('error msg');

      expect(consoleLogSpy).not.toHaveBeenCalled(); // debug and info use console.log
      expect(consoleWarnSpy).toHaveBeenCalledTimes(1);
      expect(consoleErrorSpy).toHaveBeenCalledTimes(1);
    });

    it('should only log error when set to error', () => {
      setLogLevel('error');
      consoleLogSpy.mockClear();
      
      const logger = createLogger('Test');

      logger.debug('debug msg');
      logger.info('info msg');
      logger.warn('warn msg');
      logger.error('error msg');

      expect(consoleLogSpy).not.toHaveBeenCalled();
      expect(consoleWarnSpy).not.toHaveBeenCalled();
      expect(consoleErrorSpy).toHaveBeenCalledTimes(1);
    });
  });
});
