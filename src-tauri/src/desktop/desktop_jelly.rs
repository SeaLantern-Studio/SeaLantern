//! 「桌面图标弹弹弹」Windows 实现。
//!
//! 把参考项目 DesktopJelly 的核心能力迁移为 Rust：读取资源管理器桌面图标的
//! 名称、位置和图标，隐藏原图标列表，在桌面宿主窗口上创建分层子窗口，
//! 以固定底边做 Squash & Stretch 动画；退出时恢复原图标。

#![allow(unsafe_op_in_unsafe_fn)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Default)]
pub struct DesktopJellyState {
    session: Mutex<Option<JellySession>>,
}

struct JellySession {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl DesktopJellyState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_running(&self) -> bool {
        self.session
            .lock()
            .map(|session| session.is_some())
            .unwrap_or(false)
    }

    pub fn start(&self) -> Result<(), String> {
        let mut guard = self
            .session
            .lock()
            .map_err(|_| "桌面弹弹弹状态锁已损坏".to_owned())?;

        if let Some(session) = guard.as_ref() {
            if session
                .join
                .as_ref()
                .is_some_and(|join| !join.is_finished())
            {
                return Ok(());
            }
        }
        *guard = None;

        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = Arc::clone(&stop);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<Result<(), String>>(1);

        let join = std::thread::Builder::new()
            .name("sealantern-desktop-jelly".to_owned())
            .spawn(move || {
                if let Err(error) = platform::run(stop_for_thread, ready_tx.clone()) {
                    let _ = ready_tx.send(Err(error));
                }
            })
            .map_err(|error| format!("无法创建桌面弹弹弹线程：{error}"))?;

        match ready_rx.recv_timeout(std::time::Duration::from_secs(6)) {
            Ok(Ok(())) => {
                *guard = Some(JellySession { stop, join: Some(join) });
                Ok(())
            }
            Ok(Err(error)) => {
                stop.store(true, Ordering::Release);
                let _ = join.join();
                Err(error)
            }
            Err(_) => {
                stop.store(true, Ordering::Release);
                let _ = join.join();
                Err("桌面弹弹弹启动超时，请确认桌面图标处于显示状态".to_owned())
            }
        }
    }

    pub fn stop(&self) {
        let session = self
            .session
            .lock()
            .ok()
            .and_then(|mut session| session.take());
        if let Some(mut session) = session {
            session.stop.store(true, Ordering::Release);
            if let Some(join) = session.join.take() {
                let _ = join.join();
            }
        }
    }
}

impl Drop for DesktopJellyState {
    fn drop(&mut self) {
        self.stop();
    }
}

#[tauri::command]
pub fn desktop_jelly_start(state: tauri::State<'_, DesktopJellyState>) -> Result<(), String> {
    let result = state.start();
    if let Err(error) = &result {
        tracing::error!(%error, "failed to start desktop icon jelly");
    } else {
        tracing::info!("desktop icon jelly started");
    }
    result
}

#[tauri::command]
pub fn desktop_jelly_stop(state: tauri::State<'_, DesktopJellyState>) -> Result<(), String> {
    state.stop();
    tracing::info!("desktop icon jelly stopped");
    Ok(())
}

#[tauri::command]
pub fn desktop_jelly_status(state: tauri::State<'_, DesktopJellyState>) -> bool {
    state.is_running()
}

pub fn run_restore_watcher_if_requested() -> bool {
    platform::run_restore_watcher_if_requested()
}

#[cfg(not(windows))]
mod platform {
    use super::*;

    pub fn run(
        _stop: Arc<AtomicBool>,
        _ready: std::sync::mpsc::SyncSender<Result<(), String>>,
    ) -> Result<(), String> {
        Err("桌面图标弹弹弹目前仅支持 Windows 10/11。".to_owned())
    }

    pub fn run_restore_watcher_if_requested() -> bool {
        false
    }
}

#[cfg(windows)]
mod platform {
    use super::*;

    use std::collections::HashMap;
    use std::ffi::{OsString, c_void};
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::ptr::{null, null_mut};
    use std::sync::mpsc::SyncSender;
    use std::time::{Duration, Instant};

    use fontdue::{Font, FontSettings};
    use image::{Rgba, RgbaImage, imageops::FilterType};
    use windows_sys::Win32::Foundation::{
        COLORREF, CloseHandle, GetLastError, HANDLE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE,
        WPARAM,
    };
    use windows_sys::Win32::Graphics::Gdi::{
        AC_SRC_ALPHA, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, CreateCompatibleDC,
        CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, HBITMAP, HDC, HGDIOBJ,
        MapWindowPoints, ReleaseDC, SelectObject,
    };
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Memory::{
        MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
    };
    use windows_sys::Win32::System::Threading::{
        INFINITE, OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject,
    };
    use windows_sys::Win32::UI::Controls::{
        ImageList_GetIcon, LVIF_IMAGE, LVITEMW, LVM_GETIMAGELIST, LVM_GETITEMCOUNT,
        LVM_GETITEMRECT, LVM_GETITEMTEXTW, LVM_GETITEMW, LVSIL_NORMAL,
    };
    use windows_sys::Win32::UI::Shell::{
        FOLDERID_Desktop, FOLDERID_PublicDesktop, SHGetKnownFolderPath, ShellExecuteW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CS_DBLCLKS, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EnumWindows,
        FindWindowExW, GWL_STYLE, GWLP_USERDATA, GetIconInfo, GetParent, GetWindowLongPtrW,
        GetWindowRect, GetWindowThreadProcessId, HWND_BOTTOM, ICONINFO, IDC_ARROW, IsWindow,
        IsWindowVisible, LoadCursorW, MSG, PM_REMOVE, PeekMessageW, RegisterClassW, SW_HIDE,
        SW_SHOW, SW_SHOWNOACTIVATE, SendMessageTimeoutW, SetParent, SetWindowLongPtrW,
        SetWindowPos, ShowWindow, TranslateMessage, ULW_ALPHA, UpdateLayeredWindow, WM_ERASEBKGND,
        WM_LBUTTONDBLCLK, WNDCLASSW, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_POPUP,
    };

    const CLASS_NAME: &str = "SeaLanternDesktopJellyLayer";
    const SPEED_CYCLES_PER_SECOND: f64 = 1.1;
    const STRENGTH: f64 = 0.38;
    const FRAME_INTERVAL: Duration = Duration::from_micros(16_667);

    #[derive(Clone, Copy, Debug)]
    struct RectI {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    impl RectI {
        fn width(self) -> i32 {
            self.right - self.left
        }
        fn height(self) -> i32 {
            self.bottom - self.top
        }
        fn contains(self, x: i32, y: i32) -> bool {
            x >= self.left && x < self.right && y >= self.top && y < self.bottom
        }
        fn union(self, other: RectI) -> RectI {
            RectI {
                left: self.left.min(other.left),
                top: self.top.min(other.top),
                right: self.right.max(other.right),
                bottom: self.bottom.max(other.bottom),
            }
        }
    }

    struct SourceItem {
        name: String,
        path: Option<String>,
        rect: RectI,
        icon: RgbaImage,
    }

    struct LayerItem {
        path: Option<String>,
        rect: RectI,
        icon: RgbaImage,
        label: Option<(RgbaImage, i32, i32)>,
    }

    impl LayerItem {
        fn size(&self) -> i32 {
            self.rect.height().clamp(16, 256)
        }
    }

    struct Layer {
        hwnd: HWND,
        is_child: bool,
        bounds: RectI,
        width: i32,
        height: i32,
        dib: HBITMAP,
        mem_dc: HDC,
        old_object: HGDIOBJ,
        pixels: *mut u32,
        items: Vec<LayerItem>,
        hits: Vec<RectI>,
    }

    impl Layer {
        fn create(
            parent: HWND,
            bounds: RectI,
            data: Vec<SourceItem>,
            font: Option<&Font>,
        ) -> Result<Box<Layer>, String> {
            let width = bounds.width().max(1);
            let height = bounds.height().max(1);
            let mut info = BITMAPINFO::default();
            info.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..Default::default()
            };

            unsafe {
                let mut pixels: *mut c_void = null_mut();
                let dib =
                    CreateDIBSection(null_mut(), &info, DIB_RGB_COLORS, &mut pixels, null_mut(), 0);
                if dib.is_null() || pixels.is_null() {
                    return Err(format!("创建桌面动画位图失败（错误码 {}）", GetLastError()));
                }
                let mem_dc = CreateCompatibleDC(null_mut());
                if mem_dc.is_null() {
                    DeleteObject(dib);
                    return Err("创建桌面动画绘图上下文失败".to_owned());
                }
                let old_object = SelectObject(mem_dc, dib);
                let mut items = Vec::with_capacity(data.len());
                for item in data {
                    let sprite = sprite_bounds(&item);
                    let label = font.and_then(|font| {
                        render_label(
                            font,
                            &item.name,
                            sprite.width().saturating_sub(16).max(40) as u32,
                        )
                        .map(|image| {
                            let x = item.rect.left + item.rect.width() / 2
                                - image.width() as i32 / 2
                                - bounds.left;
                            let y = item.rect.bottom - bounds.top + 4;
                            (image, x, y)
                        })
                    });
                    items.push(LayerItem {
                        path: item.path,
                        rect: item.rect,
                        icon: item.icon,
                        label,
                    });
                }

                let mut layer = Box::new(Layer {
                    hwnd: null_mut(),
                    is_child: true,
                    bounds,
                    width,
                    height,
                    dib,
                    mem_dc,
                    old_object,
                    pixels: pixels as *mut u32,
                    hits: vec![RectI { left: 0, top: 0, right: 0, bottom: 0 }; items.len()],
                    items,
                });

                let class = wide(CLASS_NAME);
                let instance = GetModuleHandleW(null());
                let mut hwnd = CreateWindowExW(
                    WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                    class.as_ptr(),
                    class.as_ptr(),
                    WS_CHILD,
                    bounds.left,
                    bounds.top,
                    width,
                    height,
                    parent,
                    null_mut(),
                    instance,
                    null(),
                );
                if hwnd.is_null() {
                    let child_error = GetLastError();
                    tracing::warn!(
                        error = child_error,
                        "desktop jelly: child window creation failed, trying SetParent fallback"
                    );
                    hwnd = CreateWindowExW(
                        WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                        class.as_ptr(),
                        class.as_ptr(),
                        WS_POPUP,
                        bounds.left,
                        bounds.top,
                        width,
                        height,
                        null_mut(),
                        null_mut(),
                        instance,
                        null(),
                    );
                    if hwnd.is_null() {
                        return Err(format!(
                            "创建桌面动画窗口失败（子窗口错误码 {}，顶层错误码 {}）",
                            child_error,
                            GetLastError()
                        ));
                    }
                    let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
                    SetWindowLongPtrW(hwnd, GWL_STYLE, ((style & !WS_POPUP) | WS_CHILD) as isize);
                    let previous = SetParent(hwnd, parent);
                    let attached = GetParent(hwnd) == parent;
                    tracing::debug!(previous = ?previous, attached, parent = ?parent, "desktop jelly: SetParent fallback result");
                    if !attached {
                        let mut host_rect = RECT::default();
                        GetWindowRect(parent, &mut host_rect);
                        SetWindowPos(
                            hwnd,
                            HWND_BOTTOM,
                            host_rect.left + bounds.left,
                            host_rect.top + bounds.top,
                            width,
                            height,
                            0x14,
                        );
                        layer.is_child = false;
                    } else {
                        SetWindowPos(
                            hwnd,
                            null_mut(),
                            bounds.left,
                            bounds.top,
                            width,
                            height,
                            0x14,
                        );
                    }
                }
                layer.hwnd = hwnd;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, &mut *layer as *mut Layer as isize);
                Ok(layer)
            }
        }

        unsafe fn show(&mut self) -> Result<(), String> {
            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            let insert_after = if self.is_child {
                null_mut()
            } else {
                HWND_BOTTOM
            };
            SetWindowPos(self.hwnd, insert_after, 0, 0, 0, 0, 0x53);
            if IsWindowVisible(self.hwnd) == 0 {
                return Err(format!("桌面动画窗口显示失败（错误码 {}）", GetLastError()));
            }
            self.render(0.0)?;
            Ok(())
        }

        fn render(&mut self, phase: f64) -> Result<(), String> {
            unsafe {
                std::ptr::write_bytes(self.pixels, 0, (self.width * self.height) as usize);
            }
            for item in &self.items {
                if let Some((label, x, y)) = &item.label {
                    draw_rgba(self.pixels, self.width, self.height, label, *x, *y);
                }
            }
            let wave = (phase * std::f64::consts::TAU).cos();
            let sy = 1.0 + STRENGTH * wave;
            let sx = 1.0 - STRENGTH * 0.72 * wave;
            for (i, item) in self.items.iter_mut().enumerate() {
                let size = item.size();
                let w = ((size as f64 * sx).round() as i32).max(1);
                let h = ((size as f64 * sy).round() as i32).max(1);
                let x = item.rect.left + item.rect.width() / 2 - self.bounds.left - w / 2;
                let y = item.rect.bottom - self.bounds.top - h;
                self.hits[i] = RectI {
                    left: x,
                    top: y,
                    right: x + w,
                    bottom: y + h + 24,
                };
                let resized =
                    image::imageops::resize(&item.icon, w as u32, h as u32, FilterType::Triangle);
                draw_rgba(self.pixels, self.width, self.height, &resized, x, y);
            }

            let size = SIZE { cx: self.width, cy: self.height };
            let origin = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: 0,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let ok = unsafe {
                UpdateLayeredWindow(
                    self.hwnd,
                    null_mut(),
                    null(),
                    &size,
                    self.mem_dc,
                    &origin,
                    0 as COLORREF,
                    &blend,
                    ULW_ALPHA,
                )
            };
            if ok == 0 {
                return Err(format!("提交桌面动画帧失败（错误码 {}）", unsafe {
                    GetLastError()
                }));
            }
            Ok(())
        }

        fn open_at(&self, x: i32, y: i32) {
            for (index, hit) in self.hits.iter().enumerate() {
                if !hit.contains(x, y) {
                    continue;
                }
                if let Some(path) = self.items.get(index).and_then(|item| item.path.as_deref()) {
                    open_path(path);
                }
                return;
            }
        }
    }

    impl Drop for Layer {
        fn drop(&mut self) {
            unsafe {
                if !self.hwnd.is_null() {
                    SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
                    DestroyWindow(self.hwnd);
                    self.hwnd = null_mut();
                }
                if !self.old_object.is_null() {
                    SelectObject(self.mem_dc, self.old_object);
                    self.old_object = null_mut();
                }
                if !self.dib.is_null() {
                    DeleteObject(self.dib);
                    self.dib = null_mut();
                }
                if !self.mem_dc.is_null() {
                    DeleteDC(self.mem_dc);
                    self.mem_dc = null_mut();
                }
            }
        }
    }

    unsafe extern "system" fn layer_wndproc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_ERASEBKGND => 1,
            WM_LBUTTONDBLCLK => {
                let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const Layer;
                if !pointer.is_null() {
                    let x = (lparam as u32 & 0xffff) as u16 as i16 as i32;
                    let y = ((lparam as u32 >> 16) & 0xffff) as u16 as i16 as i32;
                    unsafe { (*pointer).open_at(x, y) };
                }
                0
            }
            _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
        }
    }

    pub fn run(stop: Arc<AtomicBool>, ready: SyncSender<Result<(), String>>) -> Result<(), String> {
        tracing::info!("desktop jelly: locating desktop list");
        let list = find_desktop_list().ok_or_else(|| {
            "未找到 Windows 桌面图标窗口。请使用 Windows 10/11 资源管理器桌面，并先显示桌面图标。"
                .to_owned()
        })?;
        if unsafe { IsWindowVisible(list) } == 0 {
            return Err(
                "桌面图标当前处于隐藏状态，请先在桌面右键 → 查看 → 显示桌面图标。".to_owned()
            );
        }
        let host = unsafe { GetParent(list) };
        if host.is_null() {
            return Err("无法获取桌面图标宿主窗口".to_owned());
        }
        tracing::info!(?list, ?host, "desktop jelly: reading desktop items");
        let sources = read_desktop_items(list)?;
        tracing::info!(count = sources.len(), "desktop jelly: desktop items loaded");
        if sources.is_empty() {
            return Err("桌面没有可显示的图标".to_owned());
        }

        ensure_window_class()?;
        let font = load_font();
        tracing::info!(font = font.is_some(), "desktop jelly: window class ready");
        let groups = split_groups(sources);
        tracing::info!(groups = groups.len(), "desktop jelly: groups prepared");
        let mut layers = Vec::with_capacity(groups.len());
        for group in groups {
            let bounds = union_bounds(&group);
            layers.push(Layer::create(host, bounds, group, font.as_ref())?);
        }
        tracing::info!(layers = layers.len(), "desktop jelly: layers created");

        let watcher = spawn_restore_watcher(list);
        tracing::info!(watcher = watcher.is_some(), "desktop jelly: restore watcher ready");
        for layer in &mut layers {
            unsafe { layer.show()? };
        }
        tracing::info!("desktop jelly: layers shown");
        unsafe {
            ShowWindow(list, SW_HIDE);
        }
        tracing::info!("desktop jelly: original icon list hidden");
        let _ = ready.send(Ok(()));

        let started = Instant::now();
        let mut next_frame = Instant::now();
        let mut message = MSG::default();
        while !stop.load(Ordering::Acquire) {
            unsafe {
                while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            let now = Instant::now();
            if now >= next_frame {
                let phase = started.elapsed().as_secs_f64() * SPEED_CYCLES_PER_SECOND;
                for layer in &mut layers {
                    layer.render(phase)?;
                }
                next_frame += FRAME_INTERVAL;
                if next_frame < now {
                    next_frame = now + FRAME_INTERVAL;
                }
            }
            std::thread::sleep(Duration::from_millis(1));
        }

        drop(watcher);
        for layer in &mut layers {
            if !layer.hwnd.is_null() {
                unsafe { SetWindowLongPtrW(layer.hwnd, GWLP_USERDATA, 0) };
            }
        }
        layers.clear();
        unsafe {
            if IsWindow(list) != 0 {
                ShowWindow(list, SW_SHOW);
            }
        }
        Ok(())
    }

    fn ensure_window_class() -> Result<(), String> {
        use std::sync::OnceLock;
        static REGISTERED: OnceLock<Result<(), String>> = OnceLock::new();
        REGISTERED
            .get_or_init(|| unsafe {
                let class = wide(CLASS_NAME);
                let window_class = WNDCLASSW {
                    style: CS_DBLCLKS,
                    lpfnWndProc: Some(layer_wndproc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: GetModuleHandleW(null()),
                    hIcon: null_mut(),
                    hCursor: LoadCursorW(null_mut(), IDC_ARROW),
                    hbrBackground: null_mut(),
                    lpszMenuName: null(),
                    lpszClassName: class.as_ptr(),
                };
                if RegisterClassW(&window_class) == 0 {
                    let error = GetLastError();
                    // ERROR_CLASS_ALREADY_EXISTS = 1410
                    if error != 1410 {
                        return Err(format!("注册桌面动画窗口类失败（错误码 {error}）"));
                    }
                }
                Ok(())
            })
            .clone()
    }

    fn load_font() -> Option<Font> {
        for path in [r"C:\Windows\Fonts\simhei.ttf", r"C:\Windows\Fonts\segoeui.ttf"] {
            if let Ok(bytes) = std::fs::read(path)
                && let Ok(font) = Font::from_bytes(bytes, FontSettings::default())
            {
                return Some(font);
            }
        }
        None
    }

    fn render_label(font: &Font, text: &str, max_width: u32) -> Option<RgbaImage> {
        let size = 13.0;
        let mut selected = String::new();
        let mut width = 0.0;
        for ch in text.chars() {
            let advance = font.metrics(ch, size).advance_width;
            if width + advance > max_width as f32 - 8.0 {
                selected.push('…');
                break;
            }
            width += advance;
            selected.push(ch);
        }
        if selected.is_empty() {
            return None;
        }
        let total_width = selected
            .chars()
            .map(|ch| font.metrics(ch, size).advance_width)
            .sum::<f32>()
            .max(1.0);
        let width = max_width.max(total_width.ceil() as u32 + 8);
        let mut image = RgbaImage::new(width, 22);
        let baseline = 17.0f32;
        let mut pen_x = (width as f32 - total_width) / 2.0;
        for ch in selected.chars() {
            let (metrics, coverage) = font.rasterize(ch, size);
            let glyph_x = pen_x + metrics.xmin as f32;
            let glyph_y = baseline - metrics.height as f32 - metrics.ymin as f32;
            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let alpha = coverage[gy * metrics.width + gx];
                    if alpha == 0 {
                        continue;
                    }
                    blend_label_pixel(
                        &mut image,
                        glyph_x as i32 + gx as i32 + 1,
                        glyph_y as i32 + gy as i32 + 1,
                        [0, 0, 0],
                        (alpha as u16 * 150 / 255) as u8,
                    );
                    blend_label_pixel(
                        &mut image,
                        glyph_x as i32 + gx as i32,
                        glyph_y as i32 + gy as i32,
                        [255, 255, 255],
                        alpha,
                    );
                }
            }
            pen_x += metrics.advance_width;
        }
        Some(image)
    }

    fn blend_label_pixel(image: &mut RgbaImage, x: i32, y: i32, rgb: [u8; 3], alpha: u8) {
        if alpha == 0 || x < 0 || y < 0 || x >= image.width() as i32 || y >= image.height() as i32 {
            return;
        }
        let pixel = image.get_pixel_mut(x as u32, y as u32);
        let src_a = alpha as f32 / 255.0;
        let dst_a = pixel[3] as f32 / 255.0;
        let out_a = src_a + dst_a * (1.0 - src_a);
        if out_a <= 0.0 {
            return;
        }
        for channel in 0..3 {
            let src = rgb[channel] as f32 / 255.0;
            let dst = pixel[channel] as f32 / 255.0;
            pixel[channel] = (((src * src_a + dst * dst_a * (1.0 - src_a)) / out_a) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
        pixel[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    fn draw_rgba(
        canvas: *mut u32,
        canvas_width: i32,
        canvas_height: i32,
        image: &RgbaImage,
        x: i32,
        y: i32,
    ) {
        for (sx, sy, pixel) in image.enumerate_pixels() {
            let dx = x + sx as i32;
            let dy = y + sy as i32;
            if dx < 0 || dy < 0 || dx >= canvas_width || dy >= canvas_height {
                continue;
            }
            let sa = pixel[3] as u32;
            if sa == 0 {
                continue;
            }
            let index = (dy * canvas_width + dx) as usize;
            unsafe {
                let dst = *canvas.add(index);
                let dst_a = (dst >> 24) & 0xff;
                let dst_r = (dst >> 16) & 0xff;
                let dst_g = (dst >> 8) & 0xff;
                let dst_b = dst & 0xff;
                let out_a = sa + dst_a * (255 - sa) / 255;
                if out_a == 0 {
                    continue;
                }
                let src_r = pixel[0] as u32 * sa / 255;
                let src_g = pixel[1] as u32 * sa / 255;
                let src_b = pixel[2] as u32 * sa / 255;
                let out_r = src_r + dst_r * (255 - sa) / 255;
                let out_g = src_g + dst_g * (255 - sa) / 255;
                let out_b = src_b + dst_b * (255 - sa) / 255;
                *canvas.add(index) = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
            }
        }
    }

    fn sprite_bounds(item: &SourceItem) -> RectI {
        let size = item.rect.height().clamp(16, 256);
        let width = 160.max((size as f64 * 1.468).ceil() as i32 + 8);
        let high = (size as f64 * 1.65).ceil() as i32 + 4;
        let left = item.rect.left + item.rect.width() / 2 - width / 2;
        RectI {
            left,
            top: item.rect.bottom - high,
            right: left + width,
            bottom: item.rect.bottom - high + high + 56,
        }
    }

    fn union_bounds(items: &[SourceItem]) -> RectI {
        let mut result = sprite_bounds(&items[0]);
        for item in &items[1..] {
            result = result.union(sprite_bounds(item));
        }
        result
    }

    fn split_groups(items: Vec<SourceItem>) -> Vec<Vec<SourceItem>> {
        let mut result = Vec::new();
        split_group(items, &mut result);
        result
    }

    fn split_group(mut items: Vec<SourceItem>, output: &mut Vec<Vec<SourceItem>>) {
        if items.len() <= 1 {
            if !items.is_empty() {
                output.push(items);
            }
            return;
        }
        let all = union_bounds(&items);
        let occupied: i64 = items
            .iter()
            .map(|item| {
                let rect = sprite_bounds(item);
                rect.width() as i64 * rect.height() as i64
            })
            .sum();
        if all.width() as i64 * all.height() as i64 <= occupied * 5 / 4
            && all.width() as i64 * all.height() as i64 <= 2_500_000
        {
            output.push(items);
            return;
        }
        let horizontal = all.width() >= all.height();
        if horizontal {
            items.sort_by_key(|item| item.rect.left);
        } else {
            items.sort_by_key(|item| item.rect.top);
        }
        let right = items.split_off(items.len() / 2);
        split_group(items, output);
        split_group(right, output);
    }

    fn find_desktop_list() -> Option<HWND> {
        unsafe extern "system" fn callback(window: HWND, context: LPARAM) -> i32 {
            let def_view =
                FindWindowExW(window, null_mut(), wide("SHELLDLL_DefView").as_ptr(), null());
            if !def_view.is_null() {
                let list =
                    FindWindowExW(def_view, null_mut(), wide("SysListView32").as_ptr(), null());
                if !list.is_null() {
                    *(context as *mut HWND) = list;
                    return 0;
                }
            }
            1
        }
        let mut result: HWND = null_mut();
        unsafe {
            EnumWindows(Some(callback), &mut result as *mut HWND as LPARAM);
        }
        if result.is_null() { None } else { Some(result) }
    }

    fn read_desktop_items(list: HWND) -> Result<Vec<SourceItem>, String> {
        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(list, &mut pid) };
        if pid == 0 {
            return Err("无法读取桌面图标进程".to_owned());
        }
        let process = unsafe { OpenProcess(0x38, 0, pid) };
        if process.is_null() {
            return Err(
                "无法读取桌面图标位置，请让 Sea Lantern 与资源管理器使用相同权限运行".to_owned()
            );
        }
        let remote = unsafe {
            VirtualAllocEx(process, null(), 4096, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE)
        };
        if remote.is_null() {
            unsafe { CloseHandle(process) };
            return Err("无法在桌面进程中分配读取缓冲区".to_owned());
        }

        let result = (|| {
            tracing::info!("desktop jelly: requesting item count");
            let count = send_message(list, LVM_GETITEMCOUNT, 0, 0)? as i32;
            tracing::info!(count, "desktop jelly: item count received");
            if count <= 0 {
                return Err("桌面没有可显示的图标".to_owned());
            }
            if count > 2000 {
                return Err("桌面图标数量过多（超过 2000），请减少图标后重试".to_owned());
            }
            let image_list =
                send_message(list, LVM_GETIMAGELIST, LVSIL_NORMAL as usize, 0)? as isize;
            tracing::info!(image_list, "desktop jelly: image list received");
            if image_list == 0 {
                return Err("无法读取桌面图标图像列表".to_owned());
            }
            let paths = desktop_paths();
            tracing::info!(paths = paths.len(), "desktop jelly: desktop paths collected");
            let mut result = Vec::with_capacity(count as usize);
            for index in 0..count {
                let mut item = LVITEMW::default();
                item.mask = LVIF_IMAGE;
                item.iItem = index;
                item.iSubItem = 0;
                write_struct(process, remote, &item)?;
                send_message(list, LVM_GETITEMW, index as usize, remote as isize)?;
                let item_info: LVITEMW = read_struct(process, remote)?;
                let icon_index = item_info.iImage;

                zero_remote(process, remote, 2048)?;
                let mut text_item = LVITEMW::default();
                text_item.iItem = index;
                text_item.iSubItem = 0;
                text_item.pszText = unsafe { remote.add(256) as *mut u16 };
                text_item.cchTextMax = 1024;
                write_struct(process, remote, &text_item)?;
                send_message(list, LVM_GETITEMTEXTW, index as usize, remote as isize)?;
                let mut text_bytes = vec![0u8; 2048];
                read_buffer(process, unsafe { remote.add(256) }, &mut text_bytes)?;
                let name = decode_wide(&text_bytes);
                zero_remote(process, remote, 16)?;
                write_buffer(process, remote, &1u32.to_le_bytes())?; // LVIR_ICON
                send_message(list, LVM_GETITEMRECT, index as usize, remote as isize)?;
                let mut rect_bytes = [0u8; 16];
                read_buffer(process, remote, &mut rect_bytes)?;
                let raw = RectI {
                    left: i32::from_le_bytes(rect_bytes[0..4].try_into().unwrap()),
                    top: i32::from_le_bytes(rect_bytes[4..8].try_into().unwrap()),
                    right: i32::from_le_bytes(rect_bytes[8..12].try_into().unwrap()),
                    bottom: i32::from_le_bytes(rect_bytes[12..16].try_into().unwrap()),
                };
                let mut point = POINT { x: raw.left, y: raw.top };
                let host = unsafe { GetParent(list) };
                unsafe { MapWindowPoints(list, host, &mut point, 1) };
                let rect = RectI {
                    left: point.x,
                    top: point.y,
                    right: point.x + raw.width(),
                    bottom: point.y + raw.height(),
                };
                let icon = icon_from_image_list(image_list, icon_index, rect.height())
                    .unwrap_or_else(generic_icon);
                result.push(SourceItem {
                    path: paths
                        .get(&name.to_lowercase())
                        .cloned()
                        .or_else(|| special_path(&name)),
                    name,
                    rect,
                    icon,
                });
            }
            Ok(result)
        })();

        unsafe {
            VirtualFreeEx(process, remote, 0, MEM_RELEASE);
            CloseHandle(process);
        }
        result
    }

    fn send_message(
        window: HWND,
        message: u32,
        wparam: usize,
        lparam: isize,
    ) -> Result<usize, String> {
        let mut result: usize = 0;
        let ok =
            unsafe { SendMessageTimeoutW(window, message, wparam, lparam, 2, 1500, &mut result) };
        if ok == 0 {
            return Err("桌面图标窗口没有响应，请稍后重试".to_owned());
        }
        Ok(result)
    }

    fn zero_remote(process: HANDLE, address: *mut c_void, size: usize) -> Result<(), String> {
        let zeroes = vec![0u8; size];
        write_buffer(process, address, &zeroes)
    }

    fn write_struct<T>(process: HANDLE, address: *mut c_void, value: &T) -> Result<(), String> {
        let bytes = unsafe {
            std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>())
        };
        write_buffer(process, address, bytes)
    }

    fn read_struct<T: Default>(process: HANDLE, address: *const c_void) -> Result<T, String> {
        let mut value = T::default();
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                &mut value as *mut T as *mut u8,
                std::mem::size_of::<T>(),
            )
        };
        read_buffer(process, address, bytes)?;
        Ok(value)
    }

    fn write_buffer(process: HANDLE, address: *mut c_void, bytes: &[u8]) -> Result<(), String> {
        let mut written = 0usize;
        let ok = unsafe {
            WriteProcessMemory(
                process,
                address,
                bytes.as_ptr() as *const c_void,
                bytes.len(),
                &mut written,
            )
        };
        if ok == 0 || written != bytes.len() {
            return Err("写入桌面进程内存失败".to_owned());
        }
        Ok(())
    }

    fn read_buffer(
        process: HANDLE,
        address: *const c_void,
        bytes: &mut [u8],
    ) -> Result<(), String> {
        let mut read = 0usize;
        let ok = unsafe {
            ReadProcessMemory(
                process,
                address,
                bytes.as_mut_ptr() as *mut c_void,
                bytes.len(),
                &mut read,
            )
        };
        if ok == 0 || read != bytes.len() {
            return Err("读取桌面进程内存失败".to_owned());
        }
        Ok(())
    }

    fn decode_wide(bytes: &[u8]) -> String {
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|unit| *unit != 0)
            .collect();
        String::from_utf16_lossy(&units)
    }

    fn icon_from_image_list(image_list: isize, index: i32, size: i32) -> Option<RgbaImage> {
        if index < 0 {
            return None;
        }
        let size = size.clamp(16, 256);
        unsafe {
            let icon = ImageList_GetIcon(image_list, index, 1);
            if icon.is_null() {
                return None;
            }
            let result = icon_to_rgba(icon, size);
            windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon(icon);
            result
        }
    }

    fn icon_to_rgba(
        icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON,
        size: i32,
    ) -> Option<RgbaImage> {
        unsafe {
            let mut info = ICONINFO::default();
            if GetIconInfo(icon, &mut info) == 0 {
                return None;
            }
            let dc = GetDC(null_mut());
            if dc.is_null() {
                return None;
            }
            let mut color = vec![0u8; (size * size * 4) as usize];
            let mut mask = vec![0u8; (size * size * 4) as usize];
            let mut bitmap_info = BITMAPINFO::default();
            bitmap_info.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..Default::default()
            };
            let color_ok = if !info.hbmColor.is_null() {
                windows_sys::Win32::Graphics::Gdi::GetDIBits(
                    dc,
                    info.hbmColor,
                    0,
                    size as u32,
                    color.as_mut_ptr() as *mut c_void,
                    &mut bitmap_info,
                    DIB_RGB_COLORS,
                ) > 0
            } else {
                false
            };
            let mask_ok = if !info.hbmMask.is_null() {
                windows_sys::Win32::Graphics::Gdi::GetDIBits(
                    dc,
                    info.hbmMask,
                    0,
                    size as u32,
                    mask.as_mut_ptr() as *mut c_void,
                    &mut bitmap_info,
                    DIB_RGB_COLORS,
                ) > 0
            } else {
                false
            };
            ReleaseDC(null_mut(), dc);
            if !info.hbmColor.is_null() {
                DeleteObject(info.hbmColor);
            }
            if !info.hbmMask.is_null() {
                DeleteObject(info.hbmMask);
            }
            if !color_ok {
                return None;
            }
            let has_alpha = color.chunks_exact(4).any(|pixel| pixel[3] != 0);
            let mut rgba = vec![0u8; (size * size * 4) as usize];
            for i in 0..(size * size) as usize {
                let b = color[i * 4];
                let g = color[i * 4 + 1];
                let r = color[i * 4 + 2];
                let alpha = if has_alpha {
                    color[i * 4 + 3]
                } else if mask_ok
                    && (mask[i * 4] != 0 || mask[i * 4 + 1] != 0 || mask[i * 4 + 2] != 0)
                {
                    0
                } else {
                    255
                };
                rgba[i * 4] = r;
                rgba[i * 4 + 1] = g;
                rgba[i * 4 + 2] = b;
                rgba[i * 4 + 3] = alpha;
            }
            RgbaImage::from_raw(size as u32, size as u32, rgba)
        }
    }

    fn generic_icon() -> RgbaImage {
        let mut image = RgbaImage::new(48, 48);
        for y in 8..40 {
            for x in 8..40 {
                image.put_pixel(x, y, Rgba([70, 160, 190, 220]));
            }
        }
        image
    }

    fn desktop_paths() -> HashMap<String, String> {
        let mut result = HashMap::new();
        for folder in [FOLDERID_Desktop, FOLDERID_PublicDesktop] {
            if let Some(path) = known_folder(&folder) {
                collect_desktop_paths(&path, &mut result);
            }
        }
        result
    }

    fn known_folder(folder: &windows_sys::core::GUID) -> Option<PathBuf> {
        unsafe {
            let mut raw: *mut u16 = null_mut();
            if SHGetKnownFolderPath(folder, 0, null_mut(), &mut raw) < 0 || raw.is_null() {
                return None;
            }
            let mut len = 0usize;
            while *raw.add(len) != 0 {
                len += 1;
            }
            let path =
                PathBuf::from(String::from_utf16_lossy(std::slice::from_raw_parts(raw, len)));
            CoTaskMemFree(raw as *mut c_void);
            Some(path)
        }
    }

    fn collect_desktop_paths(folder: &Path, output: &mut HashMap<String, String>) {
        let Ok(entries) = std::fs::read_dir(folder) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let extension = path
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let display_name = if extension == "lnk" || extension == "url" {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(file_name)
                    .to_owned()
            } else {
                file_name.to_owned()
            };
            output.insert(display_name.to_lowercase(), path.to_string_lossy().into_owned());
            output.insert(file_name.to_lowercase(), path.to_string_lossy().into_owned());
        }
    }

    fn special_path(name: &str) -> Option<String> {
        let shell = match name.to_lowercase().as_str() {
            "此电脑" | "this pc" | "computer" => "shell:MyComputerFolder",
            "回收站" | "recycle bin" => "shell:RecycleBinFolder",
            "网络" | "network" => "shell:NetworkPlacesFolder",
            "控制面板" | "control panel" => "shell:ControlPanelFolder",
            "用户" | "users" | "用户文件夹" => "shell:UsersFilesFolder",
            _ => return None,
        };
        Some(shell.to_owned())
    }

    fn open_path(path: &str) {
        if let Some(shell) = path.strip_prefix("shell:") {
            let _ = std::process::Command::new("explorer.exe")
                .arg(format!("shell:{shell}"))
                .spawn();
            return;
        }
        let path = wide(path);
        let operation = wide("open");
        unsafe {
            ShellExecuteW(null_mut(), operation.as_ptr(), path.as_ptr(), null(), null(), SW_SHOW);
        }
    }

    fn spawn_restore_watcher(list: HWND) -> Option<std::process::Child> {
        let executable = std::env::current_exe().ok()?;
        std::process::Command::new(executable)
            .arg("--restore-desktop")
            .arg(std::process::id().to_string())
            .arg((list as isize).to_string())
            .creation_flags(0x08000000)
            .spawn()
            .ok()
    }

    pub fn run_restore_watcher_if_requested() -> bool {
        let args: Vec<OsString> = std::env::args_os().collect();
        if args.len() != 4 || args[1] != "--restore-desktop" {
            return false;
        }
        let Ok(parent_pid) = args[2].to_string_lossy().parse::<u32>() else {
            return true;
        };
        let Ok(handle) = args[3].to_string_lossy().parse::<isize>() else {
            return true;
        };
        let list = handle as HWND;
        unsafe {
            let process = OpenProcess(PROCESS_SYNCHRONIZE, 0, parent_pid);
            if !process.is_null() {
                WaitForSingleObject(process, INFINITE);
                CloseHandle(process);
            }
            if IsWindow(list) != 0 {
                ShowWindow(list, SW_SHOW);
            }
        }
        true
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}
