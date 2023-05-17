import { defineConfig } from "vite";
import vitePluginWasm from "vite-plugin-wasm";
import qiankun from 'vite-plugin-qiankun';

export default defineConfig({
  base: 'http://localhost:5173',
  server: {
    port: 5173,
    cors: true,
    origin: 'http://localhost:5173'
  },
  plugins: [vitePluginWasm(), qiankun('kline', { useDevMode: true })],
});
