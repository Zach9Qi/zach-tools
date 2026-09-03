import { afterEach, describe, expect, it, vi } from "vitest";
import { toAssetUrl } from "@/tools/clipboard/lib/api";

const IMAGE_PATH = "C:\\Users\\zach\\AppData\\Local\\zach-tools\\clipboard-images\\abc.png";

describe("toAssetUrl", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("非 Tauri 运行时(浏览器预览)一律返回空串,不触碰 Tauri IPC", () => {
    expect(toAssetUrl(IMAGE_PATH)).toBe("");
    expect(toAssetUrl(null)).toBe("");
    expect(toAssetUrl("")).toBe("");
  });

  it("Tauri 运行时把绝对路径交给 convertFileSrc 转成 asset 协议 URL", () => {
    const convertFileSrc = vi.fn((path: string) => `http://asset.localhost/${path}`);
    // @tauri-apps/api 的 convertFileSrc 走 window.__TAURI_INTERNALS__,Node 里没有 window,一并补上
    vi.stubGlobal("__TAURI_INTERNALS__", { convertFileSrc });
    vi.stubGlobal("window", globalThis);

    expect(toAssetUrl(IMAGE_PATH)).toBe(`http://asset.localhost/${IMAGE_PATH}`);
    expect(convertFileSrc).toHaveBeenCalledWith(IMAGE_PATH, "asset");
  });

  it("Tauri 运行时路径为空(非 image 条目)仍返回空串,不调用 convertFileSrc", () => {
    const convertFileSrc = vi.fn();
    vi.stubGlobal("__TAURI_INTERNALS__", { convertFileSrc });
    vi.stubGlobal("window", globalThis);

    expect(toAssetUrl(null)).toBe("");
    expect(convertFileSrc).not.toHaveBeenCalled();
  });
});
