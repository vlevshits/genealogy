/// <reference types="vitest/config" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  test: {
    exclude: ["**/node_modules/**", "**/dist/**", "**/e2e/**"],
  },
  plugins: [
    react(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules')) {
            if (id.includes('lucide-react')) {
              return 'vendor-icons';
            }
            return 'vendor';
          }
          if (id.includes('src/i18n/')) {
            return 'i18n-locales';
          }
          if (id.includes('src/lib/tree-layouts/')) {
            return 'domain-tree';
          }
          if (
            id.includes('src/pages/PrintExportPage') ||
            id.includes('src/components/print/')
          ) {
            return 'domain-print';
          }
          if (
            id.includes('src/pages/AiStoryIntakePage') ||
            id.includes('src/components/stories/')
          ) {
            return 'domain-stories';
          }
          if (
            id.includes('src/pages/DataQualityPage') ||
            id.includes('src/pages/PersonMergePage') ||
            id.includes('src/components/quality/PlaceHierarchy') ||
            id.includes('src/components/quality/BatchAutoFix')
          ) {
            return 'domain-quality';
          }
        },
      },
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
});
