use std::ffi::CString;
use std::io::{self, BufRead, Write};
use std::os::raw::c_char;
use std::process::ExitCode;

mod ast;
mod ffi;

//use ffi::{atexit_registration, get_ast, convert_program};

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
    let res = unsafe { atexit_registration() };
    if res == -1 {
        eprintln!("atexit_registration error");
        return ExitCode::FAILURE;
    }

    let code = read_interactive();

    let ast_ptr = unsafe { get_ast(code.as_ptr() as *mut c_char) };
    if ast_ptr.is_null() {
        eprintln!("err: get_ast returned NULL");
        return ExitCode::FAILURE;
    }

    let program = unsafe { convert_program(ast_ptr) };
    println!("{:#?}", program);

    ExitCode::SUCCESS
}
