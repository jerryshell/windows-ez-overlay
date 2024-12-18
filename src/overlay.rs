use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};
use windows::{
    core::s,
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            CreatePen, CreateSolidBrush, FillRect, GetBkColor, GetDC, LineTo, MoveToEx, Rectangle,
            SelectObject, HBRUSH, PS_SOLID,
        },
        UI::WindowsAndMessaging::{
            CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, RegisterClassA,
            SetLayeredWindowAttributes, TranslateMessage, CS_HREDRAW, CS_VREDRAW, LWA_COLORKEY,
            MSG, WNDCLASSA, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
        },
    },
};

use crate::{error, hdc_wrapper};

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

    pub fn window_loop(&mut self) -> Result<(), error::OverlayError> {
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
                return Err(error::OverlayError::RegisterClassA);
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
            .map_err(|_| error::OverlayError::CreateWindowExA)?;
            let hdc = GetDC(window);
            let bkcolor = GetBkColor(hdc);
            SetLayeredWindowAttributes(window, bkcolor, 0, LWA_COLORKEY)
                .map_err(|_| error::OverlayError::SetLayeredWindowAttributes)?;

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
            let hdc = hdc_wrapper::HDCWrapper(hdc);
            let tick_rate = Duration::from_millis(1000 / self.frame_rate);
            thread::spawn(move || {
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
                    thread::sleep(timeout);
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
