use std::sync::{Mutex, MutexGuard, PoisonError};

use sqlx::SqlitePool;

/// 全局共享状态：经 `.manage()` 注册，命令中用 `State<'_, AppState>` 注入。
pub struct AppState {
    /// SQLite 连接池
    db: SqlitePool,
    /// 面板唤起前的前台窗口句柄（HWND 按 isize 保存），粘贴时用于还原焦点
    paste_target: Mutex<Option<isize>>,
    /// 即将由本程序写入剪贴板的内容 hash：监听到相同 hash 的事件时跳过入库，避免回环
    pending_self_write: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            paste_target: Mutex::new(None),
            pending_self_write: Mutex::new(None),
        }
    }

    pub fn db(&self) -> &SqlitePool {
        &self.db
    }

    /// 记录粘贴目标窗口（面板唤起前的前台窗口）
    pub fn remember_paste_target(&self, handle: Option<isize>) {
        *lock_or_recover(&self.paste_target) = handle;
    }

    /// 当前粘贴目标窗口句柄
    pub fn paste_target(&self) -> Option<isize> {
        *lock_or_recover(&self.paste_target)
    }

    /// 打上自写标记：该 hash 对应的下一次剪贴板事件会被忽略
    pub fn mark_self_write(&self, hash: String) {
        *lock_or_recover(&self.pending_self_write) = Some(hash);
    }

    /// 清除自写标记（写剪贴板失败时回滚，避免误吞下一次真实复制事件）
    pub fn clear_self_write(&self) {
        *lock_or_recover(&self.pending_self_write) = None;
    }

    /// 若 hash 与自写标记一致则消费标记并返回 true
    pub fn take_self_write_if_matches(&self, hash: &str) -> bool {
        let mut pending = lock_or_recover(&self.pending_self_write);
        if pending.as_deref() == Some(hash) {
            *pending = None;
            true
        } else {
            false
        }
    }
}

/// 锁中毒时直接取回内部数据继续使用：这里保存的都是简单值，不存在不变量被破坏的问题
fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
