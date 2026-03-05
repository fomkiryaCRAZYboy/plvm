use std::ffi::CString;
use std::io::{self, BufRead, Write};
use std::os::raw::{c_char};

use::std::process::ExitCode;

unsafe extern "C"   /* main.c:125 bool interprete(char * program);*/
{
    fn interprete(program: *mut c_char) -> bool;
}

fn read_interactive() -> CString {
    let stdin = io::stdin();
    let mut program = String::new();
    let mut lnum = 1u32;

    print!("Enter your PLI program (empty line to finish):\n{lnum} > ");
    io::stdout().flush().unwrap();
    lnum += 1;

    for line in stdin.lock().lines() {
        let line = line.expect("ошибка чтения stdin");

        if line.trim().is_empty() && !program.is_empty() {
            break;
        }

        program.push_str(&line);
        program.push('\n');

        print!("{lnum} > ");
        io::stdout().flush().unwrap();
        lnum += 1;
    }

    CString::new(program).expect("код программы содержит нулевой байт")
}

fn main() -> ExitCode {
    let code = read_interactive();

    match unsafe { interprete(code.as_ptr() as *mut c_char) } 
    {
        true  =>  { ExitCode::SUCCESS }
        false =>  { ExitCode::FAILURE }
    }
}
