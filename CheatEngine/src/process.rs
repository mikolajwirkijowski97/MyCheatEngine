use std::ptr::NonNull;
use std::mem::{MaybeUninit, size_of};
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE};
use winapi::um::winnt;
use std::io;

pub struct Process {
    pid: u32,
    handle: NonNull<c_void>,
}

impl Process {
    pub fn open(pid: u32) -> io::Result<Self> {
        
        let desired_access:DWORD = winnt::PROCESS_QUERY_INFORMATION|winnt::PROCESS_VM_READ;

        NonNull::new(unsafe {winapi::um::processthreadsapi::OpenProcess(desired_access, FALSE, pid) })
            .map(|handle| Self { pid, handle })
            .ok_or_else(io::Error::last_os_error)
    }
    
    pub fn name(&self) -> io::Result<String> {
        let mut module = MaybeUninit::<HMODULE>::uninit();
        let mut size = 0;
        // SAFETY: the pointer is valid and the size is correct.
        if unsafe {
            winapi::um::psapi::EnumProcessModules(
                self.handle.as_ptr(),
                module.as_mut_ptr(),
                size_of::<HMODULE>() as u32,
                &mut size,
            )
        } == FALSE
        {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: the call succeeded, so module is initialized.
        let module = unsafe { module.assume_init() };
        let mut buffer = Vec::<u8>::with_capacity(64);
        // SAFETY: the handle, module and buffer are all valid.
        let length = unsafe {
            winapi::um::psapi::GetModuleBaseNameA(
                self.handle.as_ptr(),
                module,
                buffer.as_mut_ptr().cast(),
                buffer.capacity() as u32,
            )
        };
        if length == 0 {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: the call succeeded and length represents bytes.
        unsafe { buffer.set_len(length as usize) };
        Ok(String::from_utf8(buffer).unwrap())
    }

    pub fn read_memory(&self, addr: usize, n: usize) -> io::Result<Vec<u8>>{
        let mut buffer = Vec::<u8>::with_capacity(n);
        let mut read = 0;

        // SAFETY: the buffer points to valid memory, and the buffer size is correctly set.
        if unsafe {
            winapi::um::memoryapi::ReadProcessMemory(
                self.handle.as_ptr(),
                addr as *const _,
                 buffer.as_mut_ptr().cast(),
                buffer.capacity(),
                &mut read,
            )
        } == FALSE // Value of return checked with if with cool syntax sprinkled on top.
        {
            Err(io::Error::last_os_error())

        } else {
            // SAFETY: the call succeeded and `read` contains the amount of bytes written.
            unsafe { buffer.set_len(read as usize) };
            Ok(buffer)
        }
    }

    pub fn memory_regions(&self) -> Vec<winapi::um::winnt::MEMORY_BASIC_INFORMATION> {
        let mut base = 0;
        let mut regions = Vec::new();
        let mut memory_info = MaybeUninit::uninit();
        
        loop {
            let written = unsafe {
                winapi::um::memoryapi::VirtualQueryEx(self.handle.as_ptr(), 
                base as *const _, 
                memory_info.as_mut_ptr(), 
                size_of::<winapi::um::winnt::MEMORY_BASIC_INFORMATION>())
            };
            let memory_info = unsafe {memory_info.assume_init()};
            base = memory_info.BaseAddress as usize + memory_info.RegionSize;
            regions.push(memory_info);
        }

    }

}

// Drop is called like a destructor when the Process life has ended
impl Drop for Process {
    fn drop(&mut self) {
        // SAFETY: the handle is valid and non-null.
        unsafe { winapi::um::handleapi::CloseHandle(self.handle.as_mut()) };
    }
}