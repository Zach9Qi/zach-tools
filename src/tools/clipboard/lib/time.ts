/** 相对时间的分段边界(毫秒) */
const MINUTE = 60 * 1000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/**
 * epoch 毫秒 → 列表用的相对时间文案:
 * 1 分钟内「刚刚」,1 小时内「n 分钟前」,当天内「n 小时前」,
 * 7 天内「n 天前」,更早显示「月/日」。
 */
export function formatRelativeTime(timestamp: number, now = Date.now()): string {
  const elapsed = now - timestamp;
  if (elapsed < MINUTE) {
    return "刚刚";
  }
  if (elapsed < HOUR) {
    return `${Math.floor(elapsed / MINUTE)} 分钟前`;
  }
  if (elapsed < DAY) {
    return `${Math.floor(elapsed / HOUR)} 小时前`;
  }
  if (elapsed < 7 * DAY) {
    return `${Math.floor(elapsed / DAY)} 天前`;
  }
  const date = new Date(timestamp);
  return `${date.getMonth() + 1}/${date.getDate()}`;
}
