import { describe, expect, it } from "vitest";
import { formatRelativeTime } from "@/tools/clipboard/lib/time";

const MINUTE = 60 * 1000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** 固定「当前时间」,用本地时区构造保证跨时区可复现 */
const NOW = new Date(2026, 1, 1, 12, 0, 0).getTime(); // 2026-02-01 12:00 本地时间

describe("formatRelativeTime", () => {
  it("1 分钟内显示「刚刚」,含边界前一毫秒", () => {
    expect(formatRelativeTime(NOW, NOW)).toBe("刚刚");
    expect(formatRelativeTime(NOW - (MINUTE - 1), NOW)).toBe("刚刚");
  });

  it("满 1 分钟进入「n 分钟前」,不足 1 小时向下取整", () => {
    expect(formatRelativeTime(NOW - MINUTE, NOW)).toBe("1 分钟前");
    expect(formatRelativeTime(NOW - (HOUR - 1), NOW)).toBe("59 分钟前");
  });

  it("满 1 小时进入「n 小时前」,当天内向下取整", () => {
    expect(formatRelativeTime(NOW - HOUR, NOW)).toBe("1 小时前");
    expect(formatRelativeTime(NOW - (DAY - 1), NOW)).toBe("23 小时前");
  });

  it("满 1 天进入「n 天前」,7 天内有效", () => {
    expect(formatRelativeTime(NOW - DAY, NOW)).toBe("1 天前");
    expect(formatRelativeTime(NOW - (7 * DAY - 1), NOW)).toBe("6 天前");
  });

  it("满 7 天显示「月/日」", () => {
    const timestamp = new Date(2026, 0, 15, 8, 30, 0).getTime(); // 2026-01-15 本地时间
    expect(formatRelativeTime(timestamp, NOW)).toBe("1/15");
  });
});
