#[derive(Clone)]
pub struct HDCWrapper(pub windows::Win32::Graphics::Gdi::HDC);

impl std::ops::Deref for HDCWrapper {
    type Target = windows::Win32::Graphics::Gdi::HDC;

    fn deref(&self) -> &windows::Win32::Graphics::Gdi::HDC {
        &self.0
    }
}

unsafe impl Send for HDCWrapper {}
unsafe impl Sync for HDCWrapper {}
