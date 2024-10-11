use windows::Win32::Foundation::{CloseHandle, HANDLE};


pub struct AutoCloseHandle(pub HANDLE);

impl Drop for AutoCloseHandle {
    fn drop(&mut self) {
        unsafe { _= CloseHandle(self.0); }
    }
}
