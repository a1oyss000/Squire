use crate::Point;
use squire_error::Result;

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE,
    MOUSEINPUT, VIRTUAL_KEY,
};

pub struct WinApiBackend;

impl WinApiBackend {
    pub fn new() -> Self {
        Self
    }
}

impl super::InputBackend for WinApiBackend {
    fn click(&self, point: Point) -> Result<()> {
        move_cursor(point)?;
        send_click()?;
        Ok(())
    }

    fn double_click(&self, point: Point) -> Result<()> {
        move_cursor(point)?;
        send_click()?;
        send_click()?;
        Ok(())
    }

    fn drag(&self, from: Point, to: Point) -> Result<()> {
        move_cursor(from)?;
        #[cfg(windows)]
        {
            let down = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dwFlags: MOUSEEVENTF_LEFTDOWN,
                        ..Default::default()
                    },
                },
            };
            let sent = unsafe { SendInput(&[down], size_of::<INPUT>() as i32) };
            if sent == 0 {
                return Err(squire_error::SquireError::Input("SendInput drag down failed".to_string()));
            }
        }
        move_cursor(to)?;
        #[cfg(windows)]
        {
            let up = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dwFlags: MOUSEEVENTF_LEFTUP,
                        ..Default::default()
                    },
                },
            };
            let sent = unsafe { SendInput(&[up], size_of::<INPUT>() as i32) };
            if sent == 0 {
                return Err(squire_error::SquireError::Input("SendInput drag up failed".to_string()));
            }
        }
        Ok(())
    }

    fn key_press(&self, key: u16) -> Result<()> {
        #[cfg(windows)]
        {
            let down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(key),
                        dwFlags: Default::default(),
                        ..Default::default()
                    },
                },
            };
            let up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(key),
                        dwFlags: KEYEVENTF_KEYUP,
                        ..Default::default()
                    },
                },
            };
            let sent = unsafe { SendInput(&[down, up], size_of::<INPUT>() as i32) };
            if sent == 0 {
                return Err(squire_error::SquireError::Input("SendInput key press failed".to_string()));
            }
        }
        Ok(())
    }
}

fn move_cursor(point: Point) -> Result<()> {
    #[cfg(windows)]
    {
        let screen_w = unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
            )
        };
        let screen_h = unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
            )
        };
        let x = point.x * 65535 / screen_w;
        let y = point.y * 65535 / screen_h;

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: x,
                    dy: y,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                    ..Default::default()
                },
            },
        };
        let sent = unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
        if sent == 0 {
            return Err(squire_error::SquireError::Input("SendInput move failed".to_string()));
        }
    }
    Ok(())
}

fn send_click() -> Result<()> {
    #[cfg(windows)]
    {
        let down = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: MOUSEEVENTF_LEFTDOWN,
                    ..Default::default()
                },
            },
        };
        let up = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: MOUSEEVENTF_LEFTUP,
                    ..Default::default()
                },
            },
        };
        let sent = unsafe { SendInput(&[down, up], size_of::<INPUT>() as i32) };
        if sent == 0 {
            return Err(squire_error::SquireError::Input("SendInput click failed".to_string()));
        }
    }
    Ok(())
}

use std::mem::size_of;
