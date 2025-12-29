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
    fs: {
      // Allow serving files from the assets directory
      allow: ['..'],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './www'),
      '@assets': path.resolve(__dirname, './assets'),
    },
  },
});
