use std::ffi::CString;
use std::io::{self, BufRead, Write};
use std::os::raw::c_char;
use std::process::ExitCode;

mod jit;
use jit::{ jit_compile, JitFn };

mod bytecode_gen;
use bytecode_gen::{ Generator };

mod ast; /* ast types converted from from pli/include/parser.h */
use ast::Program;

mod ffi; /* 
          * declarations of necessary c-functions
          * conversion a ast_ptr (c-pointer) to Program type (ast.rs)
          */

use ffi::{ atexit_registration, get_ast, convert_program, emergency_cleanup };

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

    /* getting raw pointer leading to c-ast */
    let ast_ptr = unsafe { get_ast(code.as_ptr() as *mut c_char) };
    if ast_ptr.is_null() {
        eprintln!("err: get_ast returned NULL");
        return ExitCode::FAILURE;
    }

    /* converting ast from raw c-pointer to Program type */
    let program: Program = unsafe { convert_program(ast_ptr) };

    /* clearing all c-memory allocated for c-token-stream and c-ast */
    unsafe { emergency_cleanup() } ;

    //println!("{:#?}", program);

    let mut gener = Generator::new();
    gener.generate_bytecode(program);

    let bytecode = gener.finish();
    println!("{:#?}", bytecode);

    ExitCode::SUCCESS
}
