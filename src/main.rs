use std::env;
use std::ffi::CString;
use std::io::{self, BufRead, Read, Write};
use std::os::raw::c_char;
use std::path::Path;
use std::process::ExitCode;

mod vm;

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
        let line = line.expect("failed to read stdin");

        if line.trim().is_empty() && !program.is_empty() {
            break;
        }

        program.push_str(&line);
        program.push('\n');

        print!("{lnum} > ");
        io::stdout().flush().unwrap();
        lnum += 1;
    }

    CString::new(program).expect("program contains null byte")
}

fn read_from_file(path: impl AsRef<Path>) -> io::Result<CString> {
    let program = std::fs::read_to_string(path)?;
    CString::new(program).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "file contains null byte"))
}

/** Read full stdin until EOF (for piped input, e.g. echo "print(1)" | vm -). */
fn read_from_stdin() -> io::Result<CString> {
    let mut program = String::new();
    io::stdin().read_to_string(&mut program)?;
    CString::new(program).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "program contains null byte"))
}

fn main() -> ExitCode {
    let res = unsafe { atexit_registration() };
    if res == -1 {
        eprintln!("atexit_registration error");
        return ExitCode::FAILURE;
    }

    let code = match env::args().nth(1).as_deref() {
        Some("-") | Some("--stdin") => read_from_stdin().expect("failed to read stdin"),
        Some(path) => match read_from_file(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("failed to read {}: {}", path, e);
                return ExitCode::FAILURE;
            }
        },
        None => read_interactive(),
    };

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
