use anyhow::Result;
use std::sync::{Arc, RwLock};
use windows::{
    core::*,
    Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*},
};

#[derive(Debug)]
pub struct Overlay {
    pub window_rect: RECT,
    pub draw_rect_list: Arc<RwLock<Vec<RECT>>>,
    pub pen_width: i32,
    pub refresh_interval_ms: u64,
    pub draw_bottom_line_flag: bool,
}

impl Overlay {
    pub fn new(
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        draw_rect_list: Arc<RwLock<Vec<RECT>>>,
        refresh_interval_ms: u64,
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
            refresh_interval_ms,
            draw_bottom_line_flag,
        }
    }

    pub fn window_loop(&mut self) -> Result<()> {
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
                return Err(anyhow::anyhow!("err RegisterClassA"));
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
            );
            let hdc = GetDC(window);
            let bkcolor = GetBkColor(hdc);
            SetLayeredWindowAttributes(window, bkcolor, 0, LWA_COLORKEY)?;

            let pen = CreatePen(PS_SOLID, self.pen_width, COLORREF(0xFF));
            SelectObject(hdc, pen);

            let draw_rect_list = self.draw_rect_list.clone();
            let refresh_rect = RECT {
                left: 0,
                top: 0,
                right: window_width,
                bottom: window_height,
            };
            let refresh_interval_ms = self.refresh_interval_ms;
            let draw_bottom_line_flag = self.draw_bottom_line_flag;
            std::thread::spawn(move || loop {
                FillRect(hdc, &refresh_rect, HBRUSH(0x0));

                let draw_rect_list = {
                    let draw_rect_list_lock = draw_rect_list.read().unwrap();
                    draw_rect_list_lock.clone()
                };
                draw_rect_list.iter().for_each(|rect| {
                    Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);

                    if draw_bottom_line_flag {
                        MoveToEx(hdc, refresh_rect.right / 2, refresh_rect.bottom, None);
                        let rect_width = rect.right - rect.left;
                        LineTo(hdc, rect.left + rect_width / 2, rect.bottom);
                    }
                });

                std::thread::sleep(std::time::Duration::from_millis(refresh_interval_ms));
            });

            let mut message = MSG::default();
            while GetMessageA(&mut message, None, 0, 0).into() {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
        }
        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
