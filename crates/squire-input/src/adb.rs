use crate::Point;
use squire_error::{Result, SquireError};
use std::process::Command;

pub struct AdbBackend {
    device: Option<String>,
}

impl AdbBackend {
    pub fn new(device: Option<String>) -> Self {
        Self { device }
    }

    fn adb_command(&self) -> Command {
        let mut cmd = Command::new("adb");
        if let Some(ref device) = self.device {
            cmd.args(["-s", device]);
        }
        cmd
    }
}

impl super::InputBackend for AdbBackend {
    fn click(&self, point: Point) -> Result<()> {
        let output = self
            .adb_command()
            .args(["shell", "input", "tap", &point.x.to_string(), &point.y.to_string()])
            .output()
            .map_err(|e| SquireError::Adb(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SquireError::Adb(format!("tap failed: {}", stderr)));
        }
        Ok(())
    }

    fn double_click(&self, point: Point) -> Result<()> {
        self.click(point)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.click(point)
    }

    fn drag(&self, from: Point, to: Point) -> Result<()> {
        let output = self
            .adb_command()
            .args([
                "shell", "input", "swipe",
                &from.x.to_string(), &from.y.to_string(),
                &to.x.to_string(), &to.y.to_string(),
                "300",
            ])
            .output()
            .map_err(|e| SquireError::Adb(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SquireError::Adb(format!("swipe failed: {}", stderr)));
        }
        Ok(())
    }

    fn key_press(&self, key: u16) -> Result<()> {
        let output = self
            .adb_command()
            .args(["shell", "input", "keyevent", &key.to_string()])
            .output()
            .map_err(|e| SquireError::Adb(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SquireError::Adb(format!("keyevent failed: {}", stderr)));
        }
        Ok(())
    }
}

pub fn list_devices() -> Result<Vec<String>> {
    let output = Command::new("adb")
        .args(["devices"])
        .output()
        .map_err(|e| SquireError::Adb(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<String> = stdout
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 2 && parts[1] == "device" {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(devices)
}
