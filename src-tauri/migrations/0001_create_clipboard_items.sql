-- 剪贴板历史条目：text / image / files 三类共用一张表，
-- 类型专属字段只在对应 kind 下有值，其余为 NULL
CREATE TABLE clipboard_items (
    -- 主键
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 内容类型
    kind           TEXT    NOT NULL CHECK (kind IN ('text', 'image', 'files')),
    -- [text] 文本内容
    text_content   TEXT,
    -- [image] 原图落盘路径（图片二进制不进库）
    image_path     TEXT,
    -- [image] 列表缩略图落盘路径
    thumbnail_path TEXT,
    -- [image] 原图像素宽度
    image_width    INTEGER,
    -- [image] 原图像素高度
    image_height   INTEGER,
    -- [files] 文件/文件夹绝对路径列表（JSON 字符串数组）
    file_paths     TEXT,
    -- 内容 blake3 hash（text: 文本字节 / image: 像素数据 / files: 路径列表），全局去重键
    content_hash   TEXT    NOT NULL UNIQUE,
    -- 是否收藏（收藏项不参与容量清理）
    is_favorite    INTEGER NOT NULL DEFAULT 0 CHECK (is_favorite IN (0, 1)),
    -- 首次记录时间（epoch 毫秒）
    created_at     INTEGER NOT NULL,
    -- 最近一次复制/使用时间（epoch 毫秒），列表按此倒序
    last_used_at   INTEGER NOT NULL
);

CREATE INDEX idx_clipboard_items_last_used_at ON clipboard_items (last_used_at DESC);
