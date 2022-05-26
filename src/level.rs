use crate::{
    consts::*,
    memory::{Address, Memory},
};

/// 游戏关卡。
/// 
/// 使用 [`super::PVZ::get_level`] 获取。
/// 
/// # Examples
/// ```rust,no_run
/// let pvz = attach_pvz!();
/// if let Ok(mut level) = pvz.get_level() {
///    level.set_sun(9990).unwrap();
/// }
/// ```
pub struct Level<'a> {
    memory: &'a Memory,
    address: Address,
}

impl<'a> Level<'a> {    
    pub fn new(memory: &'a Memory, address: Address) -> Self {
        Level { memory, address }
    }

    /// 从偏移地址获取关卡信息。
    unsafe fn read_offset<T>(&self, offset: isize) -> Result<T, String> {
        self.memory.read(&self.address.offset(offset))
    }

    /// 从偏移地址写入关卡信息。
    unsafe fn write_offset<T>(&mut self, offset: isize, value: &T) -> Result<(), String> {
        self.memory.write(&mut self.address.offset(offset), value)
    }

    /// 获取当前阳光数。
    pub fn get_sun(&self) -> Result<i32, String> {
        unsafe { self.read_offset::<i32>(PVZ_LEVEL_SUN_OFFSET) }
    }

    /// 设置当前阳光数。
    pub fn set_sun(&mut self, value: i32) -> Result<(), String> {
        unsafe { self.write_offset::<i32>(PVZ_LEVEL_SUN_OFFSET, &value) }
    }
}
