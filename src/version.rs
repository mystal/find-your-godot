use std::fmt::{self, Display, Formatter};

use crate::platform::{PLATFORM, Platform};

pub struct GodotVersion {
    pub version: String,
    pub pre_release: Option<String>,
    pub mono: bool,
    // TODO: Cache full_version_with_flavor and full_version?
}

impl GodotVersion {
    pub fn new(full_version: &str, mono: bool) -> Self {
        // TODO: Use a more thorough heuristic to parse.
        let (version, pre_release) = full_version.split_once('-')
            .unwrap_or((full_version, ""));
        let pre_release = if pre_release.is_empty() || pre_release == "stable" {
            None
        } else {
            Some(pre_release.to_string())
        };
        Self {
            version: version.to_string(),
            pre_release,
            mono,
        }
    }

    pub fn get_full_version(&self) -> String {
        let release = match &self.pre_release {
            Some(p) => p,
            None => "stable",
        };
        format!("{}-{}", &self.version, release)
    }

    pub fn get_full_version_with_flavor(&self) -> String {
        let mut full_version = self.get_full_version();
        if self.mono {
            full_version.push_str("_mono");
        }
        full_version
    }

    pub fn get_binary_name(&self) -> String {
        // TODO: The naming convention for binary/zip names seems to change a lot. To support all
        // versions, might be best to use a static list that we generate.
        let platform_suffix = if self.version.starts_with('4') {
            match PLATFORM {
                Platform::Windows32 => "win32.exe",
                Platform::Windows64 => "win64.exe",
                Platform::MacOS => "macos.universal",
                Platform::Linux32 => "linux.x86_32",
                Platform::Linux64 => "linux.x86_64",
                Platform::Unsupported => "unsupported",
            }
        } else {
            match PLATFORM {
                Platform::Windows32 => "win32.exe",
                Platform::Windows64 => "win64.exe",
                Platform::MacOS => "osx.universal",
                Platform::Linux32 => "x11.32",
                Platform::Linux64 => "x11.64",
                Platform::Unsupported => "unsupported",
            }
        };
        format!("Godot_v{}_{}", self.get_full_version_with_flavor(), platform_suffix)
    }

    pub fn get_zip_name(&self) -> String {
        // TODO: The naming convention for binary/zip names seems to change a lot. To support all
        // versions, might be best to use a static list that we generate.
        let platform_suffix = if self.version.starts_with('4') {
            match PLATFORM {
                Platform::Windows32 => "win32",
                Platform::Windows64 => "win64",
                Platform::MacOS => "macos.universal",
                Platform::Linux32 => "linux_x86_32",
                Platform::Linux64 => "linux_x86_64",
                Platform::Unsupported => "unsupported",
            }
        } else {
            match PLATFORM {
                Platform::Windows32 => "win32",
                Platform::Windows64 => "win64",
                Platform::MacOS => "osx.universal",
                Platform::Linux32 => "x11_32",
                Platform::Linux64 => "x11_64",
                Platform::Unsupported => "unsupported",
            }
        };
        format!("Godot_v{}_{}.zip", self.get_full_version_with_flavor(), platform_suffix)
    }
}

impl Display for GodotVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.version)?;
        if let Some(pre_release) = &self.pre_release {
            write!(f, "-{}", pre_release)?;
        }
        if self.mono {
            write!(f, " (Mono)")?;
        }
        Ok(())
    }
}
