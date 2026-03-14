import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, '../../src'),
    },
  },
  test: {
    environment: 'jsdom',
    include: ['tests/frontend/**/*.test.ts'],
    setupFiles: [path.resolve(__dirname, './setup.ts')],
  },
});
