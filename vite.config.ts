import { defineConfig } from "vite";
import vitePluginWasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [vitePluginWasm()],
});
