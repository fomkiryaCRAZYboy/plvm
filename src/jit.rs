use memmap2::MmapOptions;
use std::mem;

pub type JitFn = extern "C" fn() -> i32;

/* returns the pointer to jit compiled function */
pub unsafe fn jit_compile(code: &[u8]) -> JitFn {
    let mut map = MmapOptions::new()
        .len(4096)
        .map_anon()
        .expect("mmap failed");

    map[..code.len()].copy_from_slice(code);

    let exec_map = map.make_exec().expect("make_exec failed");
    let ptr = exec_map.as_ptr();
    std::mem::forget(exec_map); /* do not drop map when exiting a function */

    unsafe { mem::transmute(ptr) }
}

/*
let code1: &[u8] = &[
            // lea rsi, [rip + 0x1a] — адрес строки (33 байта от конца lea до "Hello\n")
            0x48, 0x8d, 0x35, 0x1a, 0x00, 0x00, 0x00,  // lea rsi, [rip + 0x1a]
            0xba, 0x06, 0x00, 0x00, 0x00,              // mov edx, 6   ; длина
            0xbf, 0x01, 0x00, 0x00, 0x00,              // mov edi, 1   ; stdout
            0xb8, 0x01, 0x00, 0x00, 0x00,              // mov eax, 1   ; SYS_write
            0x0f, 0x05,                                 // syscall
            0x31, 0xff,                                 // xor edi, edi ; status = 0
            0xb8, 0x3c, 0x00, 0x00, 0x00,              // mov eax, 60  ; SYS_exit
            0x0f, 0x05,                                 // syscall
            b'H', b'e', b'l', b'l', b'o', b'\n'         // "Hello\n"
        ];

let code: &[u8] = &[0xb8, 0x2b, 0x00, 0x00, 0x00, 0xc3]; // mov eax, 43; ret

let f = unsafe { jit_compile(code1) };
println!("{}", f());
*/