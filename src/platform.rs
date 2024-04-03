// Compile time detection of platform we're running on.
pub const PLATFORM: Platform = if cfg!(target_os = "windows") {
    if cfg!(target_arch = "x86") {
        Platform::Windows32
    } else if cfg!(target_arch = "x86_64") {
        Platform::Windows64
    } else {
        Platform::Unsupported
    }
} else if cfg!(target_os = "macos") {
    Platform::MacOS
} else if cfg!(target_os = "linux") {
    if cfg!(target_arch = "x86") {
        Platform::Linux32
    } else if cfg!(target_arch = "x86_64") {
        Platform::Linux64
    } else {
        Platform::Unsupported
    }
} else {
    Platform::Unsupported
};

#[derive(Clone, Copy, Debug)]
pub enum Platform {
    Windows32,
    Windows64,
    MacOS,
    Linux32,
    Linux64,
    Unsupported,
}

impl Platform {
    pub fn to_package(&self) -> &'static str {
        match self {
            Platform::Windows32 => "win32.exe",
            Platform::Windows64 => "win64.exe",
            Platform::MacOS => "osx.universal",
            Platform::Linux32 => "x11.32",
            Platform::Linux64 => "x11.64",
            Platform::Unsupported => "unsupported",
        }
    }
}
