import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  root: 'www',
  publicDir: '../public',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    open: true,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './www'),
    },
  },
});
