use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winnt::{LPCSTR, IMAGE_NT_HEADERS32, IMAGE_DOS_HEADER};

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum SampVersion {
    NotLoaded = 0,
    Unknown = 1,
    V037R1 = 2,
    V037R3 = 3
}

pub fn samp_get_base() -> usize {
    static mut BASE: usize = SampVersion::NotLoaded as usize;
    unsafe {
        if BASE == SampVersion::NotLoaded as usize {
            BASE = GetModuleHandleA("samp.dll\0".as_ptr() as LPCSTR) as usize;
        }
        BASE
    }
}

pub fn samp_get_version() -> SampVersion {
    static mut VERSION: SampVersion = SampVersion::NotLoaded;
    unsafe {
        if VERSION == SampVersion::NotLoaded {
            let base = samp_get_base();
            if base == 0 {
                return SampVersion::NotLoaded;
            }

            let nt_header = *std::mem::transmute::<usize, *const IMAGE_NT_HEADERS32>(base + ((*std::mem::transmute::<usize, *const IMAGE_DOS_HEADER>(base)).e_lfanew as usize));

            VERSION = match nt_header.OptionalHeader.AddressOfEntryPoint {
                0x31DF13 => SampVersion::V037R1,
                0xCC4D0  => SampVersion::V037R3,
                _ => panic!("Unknown SAMP version")
            };
        }

        VERSION
    }
}