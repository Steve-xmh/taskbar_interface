use std::mem;

use raw_window_handle::{RawWindowHandle, RawDisplayHandle};
use windows::Win32::{
    Foundation::HWND,
    System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
    UI::{
        Shell::{
            ITaskbarList3, TaskbarList, TBPF_ERROR, TBPF_INDETERMINATE, TBPF_NOPROGRESS,
            TBPF_NORMAL, TBPF_PAUSED,
        },
        WindowsAndMessaging::{FlashWindowEx, FLASHWINFO, FLASHW_STOP, FLASHW_TIMERNOFG, FLASHW_TRAY},
    },
};

use crate::ProgressIndicatorState;

const MAX_PROGRESS: u64 = 100_000;

pub struct TaskbarIndicator {
    hwnd: HWND,
    taskbar: ITaskbarList3,
    progress: u64,
}

impl TaskbarIndicator {
    pub fn new(window: RawWindowHandle, _display: RawDisplayHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let hwnd = match window {
            RawWindowHandle::Win32(handle) => HWND(handle.hwnd as isize),
            h => unimplemented!("{:?}", h),
        };
        unsafe {
            // Intialize COM library if it is not already done
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let taskbar: ITaskbarList3 = unsafe { CoCreateInstance(&TaskbarList, None, CLSCTX_ALL)? };
        Ok(Self {
            hwnd,
            taskbar,
            progress: 0,
        })
    }

    fn update_progress(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.taskbar
                .SetProgressValue(self.hwnd, self.progress, MAX_PROGRESS)?;
        }
        Ok(())
    }

    pub fn set_progress(&mut self, progress: f64) -> Result<(), Box<dyn std::error::Error>> {
        let progress = (progress.clamp(0.0, 1.0) * MAX_PROGRESS as f64) as u64;
        if self.progress != progress {
            self.progress = progress;
            self.update_progress()?;
        }
        Ok(())
    }

    pub fn set_progress_state(
        &mut self,
        state: ProgressIndicatorState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let flag = match state {
            ProgressIndicatorState::NoProgress => TBPF_NOPROGRESS,
            ProgressIndicatorState::Indeterminate => TBPF_INDETERMINATE,
            ProgressIndicatorState::Normal => TBPF_NORMAL,
            ProgressIndicatorState::Paused => TBPF_PAUSED,
            ProgressIndicatorState::Error => TBPF_ERROR,
        };
        unsafe {
            self.taskbar.SetProgressState(self.hwnd, flag)?;
        }
        if matches!(
            state,
            ProgressIndicatorState::Normal
                | ProgressIndicatorState::Paused
                | ProgressIndicatorState::Error
        ) {
            self.update_progress()?;
        }
        Ok(())
    }

    pub fn needs_attention(
        &mut self,
        needs_attention: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let flags = match needs_attention {
            true => FLASHW_TIMERNOFG | FLASHW_TRAY,
            false => FLASHW_STOP,
        };
        let params = FLASHWINFO {
            cbSize: mem::size_of::<FLASHWINFO>() as u32,
            hwnd: self.hwnd,
            dwFlags: flags,
            uCount: 0,
            dwTimeout: 0,
        };
        unsafe {
            FlashWindowEx(&params);
        }
        Ok(())
    }
}
