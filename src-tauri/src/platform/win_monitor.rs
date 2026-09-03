//! Windows 剪贴板监听：独立线程创建 message-only 窗口并注册
//! `AddClipboardFormatListener`，在 Win32 消息循环中捕获 `WM_CLIPBOARDUPDATE`，
//! 按「文本优先、其次图片」读取内容后发往入库通道。

use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, GetClipboardSequenceNumber,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    TranslateMessage, HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLIPBOARDUPDATE,
    WNDCLASSW,
};

use super::ClipboardCapture;

/// 剪贴板被其他进程占用时的重试次数
const READ_RETRIES: u32 = 3;
/// 重试基础退避时长（按次数线性放大）
const READ_RETRY_DELAY: Duration = Duration::from_millis(30);

/// 启动剪贴板监听线程：进程生命周期内常驻，随进程退出回收。
pub fn spawn_monitor(tx: UnboundedSender<ClipboardCapture>) {
    let spawned = std::thread::Builder::new()
        .name("clipboard-monitor".into())
        .spawn(move || {
            if let Err(err) = run_message_loop(&tx) {
                log::error!("剪贴板监听线程异常退出: {err}");
            }
        });
    if let Err(err) = spawned {
        log::error!("剪贴板监听线程启动失败: {err}");
    }
}

fn run_message_loop(tx: &UnboundedSender<ClipboardCapture>) -> windows::core::Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("zach_tools_clipboard_monitor");
        let class = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err(windows::core::Error::from_thread());
        }

        // HWND_MESSAGE 父窗口 = message-only 窗口：不可见，只收消息
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!(""),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance.into()),
            None,
        )?;
        AddClipboardFormatListener(hwnd)?;
        log::info!("剪贴板监听已启动");

        // 序列号去抖：同一次复制可能触发多条 WM_CLIPBOARDUPDATE
        let mut last_sequence = GetClipboardSequenceNumber();
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_CLIPBOARDUPDATE {
                let sequence = GetClipboardSequenceNumber();
                if sequence == last_sequence {
                    continue;
                }
                last_sequence = sequence;

                if let Some(capture) = read_capture() {
                    if tx.send(capture).is_err() {
                        log::warn!("剪贴板入库通道已关闭，监听线程退出");
                        return Ok(());
                    }
                }
            } else {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
        }
        Ok(())
    }
}

/// 读取剪贴板内容：有非空文本取文本，否则尝试位图，两者皆无（文件列表等）返回 None。
/// 文本与图片并存（如 Excel 复制单元格）只记文本，与 uTools 的读取顺序一致。
/// 被占用时小退避重试；重试耗尽返回 None 不报错。
fn read_capture() -> Option<ClipboardCapture> {
    for attempt in 1..=READ_RETRIES {
        let attempt_result = arboard::Clipboard::new().and_then(|mut clipboard| {
            match clipboard.get_text() {
                Ok(text) if !text.is_empty() => return Ok(Some(ClipboardCapture::Text(text))),
                // 空文本视同无文本，继续看有没有图片
                Ok(_) | Err(arboard::Error::ContentNotAvailable) => {}
                Err(err) => return Err(err),
            }
            match clipboard.get_image() {
                Ok(image) => Ok(Some(ClipboardCapture::Image(image))),
                Err(arboard::Error::ContentNotAvailable) => Ok(None),
                Err(err) => Err(err),
            }
        });
        match attempt_result {
            Ok(capture) => return capture,
            Err(err) => {
                log::debug!("读取剪贴板失败（第 {attempt} 次）: {err}");
                std::thread::sleep(READ_RETRY_DELAY * attempt);
            }
        }
    }
    None
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
