use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub work_interval_mins: u32,
    pub break_duration_secs: u32,
    pub sound_enabled: bool,
    pub strict_mode: bool,
    pub launch_on_startup: bool,
    pub theme: String, // "system", "dark", or "light"
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_interval_mins: 20,
            break_duration_secs: 20,
            sound_enabled: true,
            strict_mode: false,
            launch_on_startup: false,
            theme: "system".to_string(),
        }
    }
}

impl Settings {
    pub fn config_path() -> PathBuf {
        let app_data = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let dir = app_data.join("EyeRest");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir.join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
                return settings;
            }
        }
        let default_settings = Self::default();
        let _ = default_settings.save();
        default_settings
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;

        // Apply autostart registry toggle if Windows
        self.apply_autostart();
        Ok(())
    }

    pub fn apply_autostart(&self) {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use windows::core::PCWSTR;
            use windows::Win32::System::Registry::{
                RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY,
                HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
            };

            let subkey: Vec<u16> = OsStr::new(r"Software\Microsoft\Windows\CurrentVersion\Run")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let value_name: Vec<u16> = OsStr::new("EyeRest")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            unsafe {
                let mut hkey = HKEY::default();
                if RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(subkey.as_ptr()),
                    0,
                    KEY_SET_VALUE,
                    &mut hkey,
                )
                .is_ok()
                {
                    if self.launch_on_startup {
                        if let Ok(exe_path) = std::env::current_exe() {
                            let path_str = format!("\"{}\"", exe_path.to_string_lossy());
                            let val_bytes: Vec<u16> = OsStr::new(&path_str)
                                .encode_wide()
                                .chain(std::iter::once(0))
                                .collect();
                            let byte_slice = std::slice::from_raw_parts(
                                val_bytes.as_ptr() as *const u8,
                                val_bytes.len() * 2,
                            );
                            let _ = RegSetValueExW(
                                hkey,
                                PCWSTR(value_name.as_ptr()),
                                0,
                                REG_SZ,
                                Some(byte_slice),
                            );
                        }
                    } else {
                        let _ = RegDeleteValueW(hkey, PCWSTR(value_name.as_ptr()));
                    }
                    let _ = RegCloseKey(hkey);
                }
            }
        }
    }
}
