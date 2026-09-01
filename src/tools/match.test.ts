import { describe, expect, it } from "vitest";
import { isPath, isUrl } from "@/tools/match";

describe("isUrl", () => {
  it("接受 http / https 链接,忽略首尾空白", () => {
    expect(isUrl("https://example.com")).toBe(true);
    expect(isUrl("http://example.com/path?q=1")).toBe(true);
    expect(isUrl("  https://example.com  ")).toBe(true);
  });

  it("拒绝其他协议与普通文本", () => {
    expect(isUrl("ftp://example.com")).toBe(false);
    expect(isUrl("example.com")).toBe(false);
    expect(isUrl("随便一段文字")).toBe(false);
  });

  it("协议头存在但 URL 不合法时拒绝(new URL 抛错兜底)", () => {
    expect(isUrl("https://")).toBe(false);
  });
});

describe("isPath", () => {
  it("接受盘符开头的本地路径,正反斜杠均可,盘符大小写不敏感", () => {
    expect(isPath("C:\\Users\\zach")).toBe(true);
    expect(isPath("d:/codes/rust")).toBe(true);
  });

  it("接受 UNC 网络路径", () => {
    expect(isPath("\\\\server\\share\\file.txt")).toBe(true);
  });

  it("拒绝相对路径与普通文本", () => {
    expect(isPath("./relative/path")).toBe(false);
    expect(isPath("file.txt")).toBe(false);
    expect(isPath("剪贴板")).toBe(false);
  });
});
