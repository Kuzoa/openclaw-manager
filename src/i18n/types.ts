import 'i18next';

// Import all namespace types for type safety
declare module 'i18next' {
  interface CustomTypeOptions {
    resources: {
      common: typeof import('./locales/en/common.json');
      layout: typeof import('./locales/en/layout.json');
      app: typeof import('./locales/en/app.json');
      settings: typeof import('./locales/en/settings.json');
      channels: typeof import('./locales/en/channels.json');
      agents: typeof import('./locales/en/agents.json');
      ai: typeof import('./locales/en/ai.json');
      mcp: typeof import('./locales/en/mcp.json');
      skills: typeof import('./locales/en/skills.json');
      setup: typeof import('./locales/en/setup.json');
      dashboard: typeof import('./locales/en/dashboard.json');
      logs: typeof import('./locales/en/logs.json');
      service: typeof import('./locales/en/service.json');
      testing: typeof import('./locales/en/testing.json');
    };
    defaultNS: 'common';
    returnNull: false;
  }
}
