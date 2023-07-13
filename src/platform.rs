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
