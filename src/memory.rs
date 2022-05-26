use std::ffi::c_void;

use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
        Threading::{OpenProcess, PROCESS_ALL_ACCESS},
    },
};

/// 远程进程地址。
pub struct Address(isize);

impl Address {
    pub fn new(address: isize) -> Self {
        Address(address)
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.0 as *const c_void
    }

    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self.0 as *mut c_void
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn as_bool(&self) -> bool {
        !self.is_null()
    }

    pub fn offset(&self, offset: isize) -> Self {
        Self(self.0 + offset)
    }

    pub unsafe fn read_from<T>(&self, memory: &Memory) -> Result<T, String> {
        memory.read(self)
    }

    pub unsafe fn write_to<T>(&mut self, memory: &Memory, value: &T) -> Result<(), String> {
        memory.write(self, value)
    }

    pub unsafe fn read_address_from(&self, memory: &Memory) -> Result<Self, String> {
        memory.read_address(self)
    }
}

impl From<i32> for Address {
    fn from(address: i32) -> Self {
        Address(address.try_into().unwrap())
    }
}

impl From<isize> for Address {
    fn from(address: isize) -> Self {
        Address(address)
    }
}


/// 远程进程内存管理器。
pub struct Memory {
    pid: u32,
    process_handle: HANDLE,
}

impl Memory {
    pub fn new(pid: u32) -> Result<Self, String> {
        unsafe {
            let process_handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)
                .map_err(|err| err.message().to_string_lossy())?;
            Ok(Memory {
                pid,
                process_handle,
            })
        }
    }

    pub fn get_pid(&self) -> u32 {
        self.pid
    }

    pub unsafe fn read<T>(&self, address: &Address) -> Result<T, String> {
        let mut buffer: T = std::mem::zeroed();
        let result = ReadProcessMemory(
            self.process_handle,
            address.as_ptr(),
            &mut buffer as *mut T as *mut c_void,
            std::mem::size_of::<T>(),
            std::ptr::null_mut(),
        )
        .ok();
        match result {
            Ok(()) => Ok(buffer),
            Err(err) => Err(err.message().to_string_lossy()),
        }
    }

    pub unsafe fn write<T>(&self, address: &mut Address, value: &T) -> Result<(), String> {
        WriteProcessMemory(
            self.process_handle,
            address.as_ptr(),
            value as *const T as *const c_void,
            std::mem::size_of::<T>(),
            std::ptr::null_mut(),
        )
        .ok()
        .map_err(|err| err.message().to_string_lossy())
    }

    pub unsafe fn read_address(&self, address: &Address) -> Result<Address, String> {
        Ok(self.read::<i32>(address)?.into())
    }
}
