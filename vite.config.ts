import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";

const host = process.env.TAURI_DEV_HOST;
const rootDir = process.cwd();

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@src": path.resolve(rootDir, "src"),
      "@api": path.resolve(rootDir, "src/api"),
      "@assets": path.resolve(rootDir, "src/assets"),
      "@components": path.resolve(rootDir, "src/components"),
      "@composables": path.resolve(rootDir, "src/composables"),
      "@data": path.resolve(rootDir, "src/data"),
      "@language": path.resolve(rootDir, "src/language"),
      "@router": path.resolve(rootDir, "src/router"),
      "@stores": path.resolve(rootDir, "src/stores"),
      "@styles": path.resolve(rootDir, "src/styles"),
      "@themes": path.resolve(rootDir, "src/themes"),
      "@src-tauri": path.resolve(rootDir, "src-tauri"),
      "@type": path.resolve(rootDir, "src/types"),
      "@utils": path.resolve(rootDir, "src/utils"),
      "@views": path.resolve(rootDir, "src/views"),
    },
  },
  build: {
    target: "esnext",
    minify: "terser",
    terserOptions: {
      compress: {
        drop_console: false,
        drop_debugger: true,
      },
    },
    rollupOptions: {
      output: {
        manualChunks: {
          "vue-vendor": ["vue", "vue-router", "pinia"],
          "tauri-vendor": ["@tauri-apps/api", "@tauri-apps/plugin-dialog"],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 5174,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
