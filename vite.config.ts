import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import Icons from "unplugin-icons/vite";


const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    tailwindcss(),
    Icons({
      compiler: "vue3",
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  // 以下选项专为 Tauri 开发准备，仅在 `tauri dev` / `tauri build` 时生效
  //
  // 1. 关闭清屏，避免盖住 Rust 编译错误
  clearScreen: false,
  // 2. Tauri 需要固定端口，被占用时直接失败
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. 不要监听 src-tauri，后端由 cargo 自己编译
      ignored: ["**/src-tauri/**"],
    },
  },
}));
