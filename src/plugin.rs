use libc::{memcmp, c_void, aligned_malloc};
use crate::samp::*;

static mut HOOK: Option<Box<rtdhook_rs::callhook::CallHook>> = None;
static mut MAINLOOP_HOOK: Option<Box<rtdhook_rs::callhook::CallHook>> = None;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct D3DCOLOR {
    _r: u8,
    _g: u8,
    _b: u8,
    a: u8,
}

unsafe extern "cdecl" fn mainloop() {
    static mut LOADED: bool = false;
    if !LOADED && samp_get_version() > SampVersion::Unknown {
        let hook_addr = match samp_get_version() {
            SampVersion::V037R3 => 0x22A4,
            SampVersion::V037R1 => 0x22B4,
            _ => unreachable!()
        };
        
        let naked_detour = aligned_malloc(16, 4);
        if naked_detour.is_null() {
            panic!("Couldn't alloc for naked function");
        }
        let naked_detour = naked_detour as usize;
        *((naked_detour + 0) as *mut u8) = 0x58; // pop eax
        *((naked_detour + 1) as *mut u8) = 0x56; // push esi
        *((naked_detour + 2) as *mut u8) = 0x50; // push eax
        *((naked_detour + 3) as *mut u8) = 0xE9; // jmp
        *((naked_detour + 4) as *mut usize) = CGame__DrawGangZone as usize - (naked_detour + 3) - 5; // relative address

        HOOK = Some(Box::new(rtdhook_rs::callhook::CallHook::new(samp_get_base() + hook_addr, naked_detour)));
        HOOK.as_mut().unwrap().install();

        LOADED = true;
    }
    std::mem::transmute::<usize, extern "cdecl" fn()>(MAINLOOP_HOOK.as_mut().unwrap().function_ptr())();
}

#[allow(non_snake_case)]
unsafe extern "stdcall" fn CGame__DrawGangZone(gang_zone_pool: *const usize, a1: *const f32, mut a2: D3DCOLOR) -> i32 {
    let color = (*gang_zone_pool + 0x10) as *mut D3DCOLOR;
    let alt_color = (*gang_zone_pool + 0x14) as *const D3DCOLOR;
    if memcmp(color as *const c_void, alt_color as *const c_void, 4) != 0 {
        let mut color = *color;
        let alt_color = *alt_color;
        a2 = alt_color;
        a2.a = ((((*((
            if *((0xBA6748 + 0x59) as *const bool) { 0xB7CB7C } else { 0xB7CB84 }
        ) as *const u32) & 1023) as f32 * 0.0061359233f32).sin() + 1.0f32)
            * 0.5f32
            * a2.a as f32) as u8;
        
        color.a = alt_color.a - a2.a;
        std::mem::transmute::<usize, extern "stdcall" fn(*const f32, a2: D3DCOLOR) -> i32>(HOOK.as_mut().unwrap().function_ptr())(a1, color);
    }
    std::mem::transmute::<usize, extern "stdcall" fn(*const f32, a2: D3DCOLOR) -> i32>(HOOK.as_mut().unwrap().function_ptr())(a1, a2)
}

pub fn init() {
    unsafe {
        match MAINLOOP_HOOK.as_ref() {
            None => {
                MAINLOOP_HOOK = Some(Box::new(rtdhook_rs::callhook::CallHook::new(0x53E968, mainloop as usize)));
                MAINLOOP_HOOK.as_mut().unwrap().install();
            },
            _ => {}
        }
    }
}
