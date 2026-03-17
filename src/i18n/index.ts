import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Import all translation files
import commonEn from './locales/en/common.json';
import commonZh from './locales/zh/common.json';
import layoutEn from './locales/en/layout.json';
import layoutZh from './locales/zh/layout.json';
import appEn from './locales/en/app.json';
import appZh from './locales/zh/app.json';
import settingsEn from './locales/en/settings.json';
import settingsZh from './locales/zh/settings.json';
import channelsEn from './locales/en/channels.json';
import channelsZh from './locales/zh/channels.json';
import agentsEn from './locales/en/agents.json';
import agentsZh from './locales/zh/agents.json';
import aiEn from './locales/en/ai.json';
import aiZh from './locales/zh/ai.json';
import mcpEn from './locales/en/mcp.json';
import mcpZh from './locales/zh/mcp.json';
import skillsEn from './locales/en/skills.json';
import skillsZh from './locales/zh/skills.json';
import setupEn from './locales/en/setup.json';
import setupZh from './locales/zh/setup.json';
import dashboardEn from './locales/en/dashboard.json';
import dashboardZh from './locales/zh/dashboard.json';
import logsEn from './locales/en/logs.json';
import logsZh from './locales/zh/logs.json';
import serviceEn from './locales/en/service.json';
import serviceZh from './locales/zh/service.json';
import testingEn from './locales/en/testing.json';
import testingZh from './locales/zh/testing.json';

// Define all resources
const resources = {
  en: {
    common: commonEn,
    layout: layoutEn,
    app: appEn,
    settings: settingsEn,
    channels: channelsEn,
    agents: agentsEn,
    ai: aiEn,
    mcp: mcpEn,
    skills: skillsEn,
    setup: setupEn,
    dashboard: dashboardEn,
    logs: logsEn,
    service: serviceEn,
    testing: testingEn,
  },
  zh: {
    common: commonZh,
    layout: layoutZh,
    app: appZh,
    settings: settingsZh,
    channels: channelsZh,
    agents: agentsZh,
    ai: aiZh,
    mcp: mcpZh,
    skills: skillsZh,
    setup: setupZh,
    dashboard: dashboardZh,
    logs: logsZh,
    service: serviceZh,
    testing: testingZh,
  },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
      lookupLocalStorage: 'language',
    },
    fallbackLng: 'en',
    supportedLngs: ['en', 'zh'],
    defaultNS: 'common',
    interpolation: {
      escapeValue: false, // React already handles XSS
      format: (value, format, lng) => {
        if (value instanceof Date) {
          switch (format) {
            case 'time':
              return value.toLocaleTimeString(lng, {
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
                hour12: false,
              });
            case 'timeWithMs':
              return (
                value.toLocaleTimeString(lng, {
                  hour: '2-digit',
                  minute: '2-digit',
                  second: '2-digit',
                  hour12: false,
                }) +
                '.' +
                String(value.getMilliseconds()).padStart(3, '0')
              );
            case 'date':
              return value.toLocaleDateString(lng);
            case 'datetime':
              return value.toLocaleString(lng);
            default:
              return value.toLocaleString(lng);
          }
        }
        return value;
      },
    },
    react: {
      useSuspense: false,
    },
  });

export default i18n;
