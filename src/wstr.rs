use windows::core::{PCWSTR, PWSTR};

/// 适用于 Windows API 的宽字符串。
/// 
/// 使用 UTF-16 编码，零终止。
pub struct WStr {
    chars: Vec<u16>,
}

impl WStr {
    pub fn new(s: &str) -> Self {
        let mut v: Vec<_> = s.encode_utf16().collect();
        v.push(0);
        WStr { chars: v }
    }

    pub fn as_wstr(&mut self) -> PWSTR {
        PWSTR(self.chars.as_mut_ptr())
    }

    pub fn as_cwstr(&self) -> PCWSTR {
        PCWSTR(self.chars.as_ptr())
    }

    pub fn null_cwstr() -> PCWSTR {
        PCWSTR(std::ptr::null::<u16>())
    }
}

impl std::fmt::Display for WStr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let chars: Vec<_> = self
            .chars
            .iter()
            .take_while(|c| **c != 0)
            .copied()
            .collect();
        let s = String::from_utf16_lossy(&chars);
        f.write_str(s.as_str())
    }
}

impl<const N: usize> From<&[u16; N]> for WStr {
    fn from(s: &[u16; N]) -> Self {
        WStr { chars: s.to_vec() }
    }
}

impl From<&str> for WStr {
    fn from(s: &str) -> Self {
        WStr::new(s)
    }
}
