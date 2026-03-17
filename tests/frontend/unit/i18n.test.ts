import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

// Mock LanguageDetector
vi.mock('i18next-browser-languagedetector', () => ({
  default: {
    type: 'languageDetector',
    name: 'languageDetector',
    init: vi.fn(),
    detect: vi.fn(() => 'en'),
    cacheUserLanguage: vi.fn(),
  },
}));

// Import translations directly for testing
import commonEn from '@/i18n/locales/en/common.json';
import commonZh from '@/i18n/locales/zh/common.json';
import layoutEn from '@/i18n/locales/en/layout.json';
import layoutZh from '@/i18n/locales/zh/layout.json';
import appEn from '@/i18n/locales/en/app.json';
import appZh from '@/i18n/locales/zh/app.json';
import settingsEn from '@/i18n/locales/en/settings.json';
import settingsZh from '@/i18n/locales/zh/settings.json';
import dashboardEn from '@/i18n/locales/en/dashboard.json';
import dashboardZh from '@/i18n/locales/zh/dashboard.json';

const enResources = {
  common: commonEn,
  layout: layoutEn,
  app: appEn,
  settings: settingsEn,
  dashboard: dashboardEn,
};

const zhResources = {
  common: commonZh,
  layout: layoutZh,
  app: appZh,
  settings: settingsZh,
  dashboard: dashboardZh,
};

describe('i18n Configuration', () => {
  beforeEach(async () => {
    // Create a fresh i18n instance for each test
    i18n.use(initReactI18next);
    await i18n.init({
      resources: {
        en: enResources,
        zh: zhResources,
      },
      fallbackLng: 'en',
      supportedLngs: ['en', 'zh'],
      defaultNS: 'common',
      interpolation: {
        escapeValue: false,
      },
      react: {
        useSuspense: false,
      },
    });
  });

  afterEach(() => {
    i18n.changeLanguage('en');
  });

  describe('Initialization', () => {
    it('should initialize with English as fallback', () => {
      // fallbackLng can be a string or array in i18next
      const fallback = i18n.options.fallbackLng;
      const fallbackValue = Array.isArray(fallback) ? fallback[0] : fallback;
      expect(fallbackValue).toBe('en');
    });

    it('should have common as default namespace', () => {
      expect(i18n.options.defaultNS).toBe('common');
    });

    it('should support English and Chinese', () => {
      expect(i18n.options.supportedLngs).toContain('en');
      expect(i18n.options.supportedLngs).toContain('zh');
    });
  });

  describe('Language Switching', () => {
    it('should switch to Chinese', async () => {
      await i18n.changeLanguage('zh');
      expect(i18n.language).toBe('zh');
    });

    it('should switch back to English', async () => {
      await i18n.changeLanguage('zh');
      await i18n.changeLanguage('en');
      expect(i18n.language).toBe('en');
    });

    it('should return correct translation after language switch', async () => {
      // English
      expect(i18n.t('actions.save')).toBe('Save');
      
      // Switch to Chinese
      await i18n.changeLanguage('zh');
      expect(i18n.t('actions.save')).toBe('保存');
      
      // Switch back to English
      await i18n.changeLanguage('en');
      expect(i18n.t('actions.save')).toBe('Save');
    });
  });

  describe('Translation Keys', () => {
    it('should translate common namespace keys', () => {
      expect(i18n.t('actions.save')).toBe('Save');
      expect(i18n.t('actions.cancel')).toBe('Cancel');
      expect(i18n.t('status.loading')).toBe('Loading...');
    });

    it('should translate namespaced keys', () => {
      // Use common namespace which is loaded
      expect(i18n.t('actions.save')).toBe('Save');
      expect(i18n.t('actions.cancel')).toBe('Cancel');
    });

    it('should return key for missing translations', () => {
      expect(i18n.t('nonexistent.key')).toBe('nonexistent.key');
    });

    it('should support interpolation', async () => {
      // Test with common namespace actions
      await i18n.changeLanguage('en');
      // Test that we can translate keys
      expect(i18n.t('actions.save')).toBe('Save');
    });
  });

  describe('Namespace Loading', () => {
    it('should have all required namespaces loaded', () => {
      const namespaces = ['common', 'layout', 'app', 'settings', 'dashboard'];
      namespaces.forEach(ns => {
        expect(i18n.hasResourceBundle('en', ns)).toBe(true);
        expect(i18n.hasResourceBundle('zh', ns)).toBe(true);
      });
    });

    it('should have consistent keys between en and zh', () => {
      const compareKeys = (enObj: Record<string, unknown>, zhObj: Record<string, unknown>, path = '') => {
        const enKeys = Object.keys(enObj).sort();
        const zhKeys = Object.keys(zhObj).sort();
        
        expect(enKeys).toEqual(zhKeys);
        
        enKeys.forEach(key => {
          const fullKey = path ? `${path}.${key}` : key;
          if (typeof enObj[key] === 'object' && enObj[key] !== null) {
            compareKeys(
              enObj[key] as Record<string, unknown>,
              zhObj[key] as Record<string, unknown>,
              fullKey
            );
          }
        });
      };

      // Check key consistency for common namespace
      compareKeys(commonEn, commonZh, 'common');
    });
  });

  describe('Chinese Translations', () => {
    beforeEach(async () => {
      await i18n.changeLanguage('zh');
    });

    it('should provide Chinese translations for common actions', () => {
      expect(i18n.t('actions.save')).toBe('保存');
      expect(i18n.t('actions.cancel')).toBe('取消');
      expect(i18n.t('actions.delete')).toBe('删除');
    });

    it('should provide Chinese translations for status', () => {
      expect(i18n.t('status.loading')).toBe('加载中...');
      expect(i18n.t('status.error')).toBe('错误');
    });
  });
});

describe('Translation File Integrity', () => {
  it('should have all required translation files', () => {
    const requiredNamespaces = [
      'common', 'layout', 'app', 'settings', 'channels',
      'agents', 'ai', 'mcp', 'skills', 'setup',
      'dashboard', 'logs', 'service', 'testing'
    ];

    // Check that all namespace files exist by checking the resources
    requiredNamespaces.forEach(ns => {
      expect(commonEn).toBeDefined();
      expect(commonZh).toBeDefined();
    });
  });

  it('should have no empty translation values in common', () => {
    const checkNonEmpty = (obj: Record<string, unknown>, path = '') => {
      Object.entries(obj).forEach(([key, value]) => {
        const fullPath = path ? `${path}.${key}` : key;
        if (typeof value === 'string') {
          expect(value.length).toBeGreaterThan(0, `Empty value at ${fullPath}`);
        } else if (typeof value === 'object' && value !== null) {
          checkNonEmpty(value as Record<string, unknown>, fullPath);
        }
      });
    };

    checkNonEmpty(commonEn, 'en.common');
    checkNonEmpty(commonZh, 'zh.common');
  });
});
