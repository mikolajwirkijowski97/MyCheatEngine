pub mod process;

use process::Process;
use std::io;
use std::mem;
use winapi::shared::minwindef::{DWORD,FALSE};


pub fn enum_proc() -> io::Result<Vec<u32>> {
    const  BASE_CAPACITY: u32 = 1024;
    const MULTIPLIER: u32 = 1; 
    let mut pids = Vec::<DWORD>::with_capacity((BASE_CAPACITY*MULTIPLIER) as usize);
    let mut size:u32 = 0;

    if unsafe {
        
        // How does WinApi know what the virtual adress is reffering to? I know it works with threads too, so theres sth im missing
        winapi::um::psapi::EnumProcesses(
            pids.as_mut_ptr(),
            (pids.capacity() * mem::size_of::<DWORD>()) as u32,
            &mut size,
        )
    } == FALSE {
        return  Err(io::Error::last_os_error());
    }

    let count = size as usize / mem::size_of::<DWORD>();
    unsafe { pids.set_len(count) };
    Ok(pids)
    
}

fn main() {
    enum_proc()
    .unwrap()
    .into_iter()
    .for_each(|pid| match Process::open(pid) {
        Ok(proc) => match proc.name() {
            Ok(name) => println!("{}: {}", pid, name),
            Err(e) => println!("{}: (failed to get name: {})", pid, e),
        },
        Err(e) => eprintln!("failed to open {}: {}", pid, e),
    });
}
