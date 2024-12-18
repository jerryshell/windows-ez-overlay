use std::{
    ops::Deref,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use windows::{
    core::s,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            CreatePen, CreateSolidBrush, FillRect, GetBkColor, GetDC, LineTo, MoveToEx, Rectangle,
            SelectObject, HBRUSH, HDC, PS_SOLID,
        },
        UI::WindowsAndMessaging::{
            CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, RegisterClassA,
            SetLayeredWindowAttributes, TranslateMessage, CS_HREDRAW, CS_VREDRAW, LWA_COLORKEY,
            MSG, WNDCLASSA, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
        },
    },
};

#[derive(Debug)]
pub enum OverlayError {
    RegisterClassA,
    CreateWindowExA,
    SetLayeredWindowAttributes,
}

impl std::fmt::Display for OverlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for OverlayError {}

#[derive(Clone)]
struct HDCWrapper(HDC);

impl Deref for HDCWrapper {
    type Target = HDC;

    fn deref(&self) -> &HDC {
        &self.0
    }
}

unsafe impl Send for HDCWrapper {}
unsafe impl Sync for HDCWrapper {}

#[derive(Debug)]
pub struct Overlay {
    pub window_rect: RECT,
    pub draw_rect_list: Arc<RwLock<Vec<RECT>>>,
    pub pen_width: i32,
    pub frame_rate: u64,
    pub draw_bottom_line_flag: bool,
}

impl Overlay {
    pub fn new(
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        draw_rect_list: Arc<RwLock<Vec<RECT>>>,
        frame_rate: u64,
        draw_bottom_line_flag: bool,
    ) -> Self {
        Overlay {
            window_rect: RECT {
                left,
                top,
                right,
                bottom,
            },
            draw_rect_list,
            pen_width: 1,
            frame_rate,
            draw_bottom_line_flag,
        }
    }

    pub fn window_loop(&mut self) -> Result<(), OverlayError> {
        unsafe {
            let window_class_name = s!("ezOverlay");

            let window_class = WNDCLASSA {
                lpszClassName: window_class_name,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                ..Default::default()
            };
            let atom = RegisterClassA(&window_class);
            if atom == 0 {
                return Err(OverlayError::RegisterClassA);
            }

            let window_width = self.window_rect.right - self.window_rect.left;
            let window_height = self.window_rect.bottom - self.window_rect.top;

            let window = CreateWindowExA(
                WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_LAYERED,
                window_class_name,
                s!("EZ Overlay"),
                WS_POPUP | WS_VISIBLE,
                self.window_rect.left,
                self.window_rect.top,
                window_width,
                window_height,
                None,
                None,
                None,
                None,
            )
            .map_err(|_| OverlayError::CreateWindowExA)?;
            let hdc = GetDC(window);
            let bkcolor = GetBkColor(hdc);
            SetLayeredWindowAttributes(window, bkcolor, 0, LWA_COLORKEY)
                .map_err(|_| OverlayError::SetLayeredWindowAttributes)?;

            let pen = CreatePen(PS_SOLID, self.pen_width, COLORREF(0xFF));
            SelectObject(hdc, pen);

            let draw_rect_list = self.draw_rect_list.clone();
            let refresh_rect = RECT {
                left: 0,
                top: 0,
                right: window_width,
                bottom: window_height,
            };
            let draw_bottom_line_flag = self.draw_bottom_line_flag;
            let hdc = HDCWrapper(hdc);
            let tick_rate = Duration::from_millis(1000 / self.frame_rate);
            std::thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let hbr: HBRUSH = CreateSolidBrush(bkcolor);
                    FillRect(*hdc, &refresh_rect, hbr);

                    let draw_rect_list = {
                        let draw_rect_list_lock = draw_rect_list.read().unwrap();
                        draw_rect_list_lock.clone()
                    };
                    draw_rect_list.iter().for_each(|rect| {
                        let _ = Rectangle(*hdc, rect.left, rect.top, rect.right, rect.bottom);

                        if draw_bottom_line_flag {
                            let _ =
                                MoveToEx(*hdc, refresh_rect.right / 2, refresh_rect.bottom, None);
                            let rect_width = rect.right - rect.left;
                            let _ = LineTo(*hdc, rect.left + rect_width / 2, rect.bottom);
                        }
                    });

                    let timeout = tick_rate.saturating_sub(last_tick.elapsed());
                    std::thread::sleep(timeout);
                    last_tick = Instant::now();
                }
            });

            let mut message = MSG::default();
            while GetMessageA(&mut message, None, 0, 0).into() {
                let _ = TranslateMessage(&message);
                DispatchMessageA(&message);
            }
        }
        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefWindowProcA(window, message, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::RwLock;

    #[test]
    fn it_works() {
        const FRAME_RATE: u64 = 60;

        let rect_list = Arc::new(RwLock::new(vec![
            RECT {
                left: 0,
                top: 0,
                right: 100,
                bottom: 100,
            },
            RECT {
                left: 123,
                top: 456,
                right: 789,
                bottom: 666,
            },
        ]));

        {
            let rect_list = rect_list.clone();
            let mut overlay = Overlay::new(0, 0, 1920, 1080, rect_list, FRAME_RATE, true);
            std::thread::spawn(move || {
                overlay.window_loop().unwrap();
            });
        }

        let mut frame_count = 0;
        let tick_rate = Duration::from_millis(1000 / FRAME_RATE);
        let mut last_tick = Instant::now();
        loop {
            {
                let mut rect_list = rect_list.write().unwrap();
                rect_list.iter_mut().for_each(|rect| {
                    rect.left += 1;
                    rect.top += 1;
                    rect.right += 1;
                    rect.bottom += 1;
                });
            }

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            std::thread::sleep(timeout);
            last_tick = Instant::now();

            frame_count += 1;
            if frame_count >= 500 {
                break;
            }
        }
    }
}
