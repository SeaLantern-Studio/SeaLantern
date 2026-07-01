import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import { fileURLToPath } from "url";

const host = process.env.TAURI_DEV_HOST;
const configDir = path.dirname(fileURLToPath(import.meta.url));
const frontendDir = configDir;
const rootDir = path.resolve(frontendDir, "..");

export default defineConfig({
  root: frontendDir,
  plugins: [vue()],
  resolve: {
    alias: {
      "@next-src": path.resolve(frontendDir, "next-src"),
      "@src": path.resolve(frontendDir, "src"),
      "@api": path.resolve(frontendDir, "src/api"),
      "@assets": path.resolve(frontendDir, "src/assets"),
      "@components": path.resolve(frontendDir, "src/components"),
      "@composables": path.resolve(frontendDir, "src/composables"),
      "@data": path.resolve(frontendDir, "src/data"),
      "@language": path.resolve(frontendDir, "src/language"),
      "@router": path.resolve(frontendDir, "src/router"),
      "@shared": path.resolve(rootDir, "shared"),
      "@stores": path.resolve(frontendDir, "src/stores"),
      "@styles": path.resolve(frontendDir, "src/styles"),
      "@themes": path.resolve(frontendDir, "src/themes"),
      "@tauri-host": path.resolve(rootDir, "backend/tauri-host"),
      "@type": path.resolve(frontendDir, "src/types"),
      "@utils": path.resolve(frontendDir, "src/utils"),
      "@views": path.resolve(frontendDir, "src/views"),
    },
  },
  build: {
    outDir: path.resolve(rootDir, "dist"),
    emptyOutDir: true,
    target: "esnext",
    minify: "oxc",
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules")) {
            if (id.includes("vue") || id.includes("vue-router") || id.includes("pinia")) {
              return "vue-vendor";
            }
            if (id.includes("@tauri-apps")) {
              return "tauri-vendor";
            }
            if (id.includes("@xterm/") || id.includes("xterm")) {
              return "xterm-vendor";
            }
            if (id.includes("@codemirror/") || id.includes("@lezer/")) {
              return "codemirror-vendor";
            }
            if (id.includes("echarts") || id.includes("vue-echarts")) {
              return "echarts-vendor";
            }
            if (id.includes("@headlessui") || id.includes("reka-ui")) {
              return "ui-vendor";
            }
            if (id.includes("@vueuse") || id.includes("dompurify") || id.includes("@lucide/vue")) {
              return "utils-vendor";
            }
          }
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
      ignored: ["**/backend/tauri-host/**"],
    },
  },
});
