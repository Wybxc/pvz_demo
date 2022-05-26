//! # PVZ Demo
//!
//! 使用 Rust 注入并操作 PVZ 游戏的示例性工具。
//!
//! 具体使用方式请参考 [`PVZ`] 类。

mod consts;
pub mod level;
mod memory;
mod wstr;

use std::path::Path;

use windows::Win32::{
    Foundation::CloseHandle,
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        Threading::CreateProcessW,
    },
    UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId},
};

use consts::*;
use level::*;
use memory::*;
use wstr::*;

/// 游戏主体附加。
///
/// 使用 [`open_pvz`] 或 [`attach_pvz`] 宏创建并获取：
/// ```rust,no_run
/// let pvz = open!();
/// let pvz = attach!();
/// ```
///
/// # Examples
/// ```rust,no_run
/// use pvz_demo::*;
///
/// let pvz = attach_pvz!();
/// println!("进程ID: {}", pvz.get_pid());
///
/// if let Ok(mut level) = pvz.get_level() {
///    level.set_sun(9990).unwrap();
/// }
/// ```
pub struct PVZ {
    memory: Memory,
}

impl PVZ {
    /// 从已知的 PID 创建一个 PVZ 对象。    
    ///
    /// # Examples
    /// ```rust,no_run
    /// let pid = unsafe {
    ///     let mut pid = 0;
    ///     GetWindowThreadProcessId(FindWindowEx(...), &mut pid)
    /// };
    /// let pvz = PVZ::new(pid);
    /// ```
    pub fn new(pid: u32) -> Result<Self, String> {
        Ok(PVZ {
            memory: Memory::new(pid)?,
        })
    }

    /// 从进程名获取运行中的游戏进程，并以此创建一个 PVZ 对象。
    ///
    /// 进程名一般是 exe 文件的名称，如 `"PlantsVsZombies.exe"`。
    ///
    /// 此方法较复杂，一般情况下请使用 [`attach_pvz`] 宏。
    ///
    /// # Examples
    /// ```rust,no_run
    /// let pvz = PVZ::new_with_process_name("PlantsVsZombies.exe").unwrap();
    /// ```
    pub fn new_with_process_name(process_name: &str) -> Result<Self, String> {
        unsafe {
            let mut pe32: PROCESSENTRY32W = std::mem::zeroed();
            pe32.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            let hsnapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .map_err(|err| err.message().to_string_lossy())?;

            let mut ret = Process32FirstW(hsnapshot, &mut pe32).into();
            while ret {
                let name = WStr::from(&pe32.szExeFile).to_string();
                if process_name == name {
                    CloseHandle(hsnapshot);
                    return PVZ::new(pe32.th32ProcessID);
                }
                ret = Process32NextW(hsnapshot, &mut pe32).into();
            }
            CloseHandle(hsnapshot);
        }
        Err("未找到符合名称的进程".to_string())
    }

    /// 从窗口标题获取运行中的游戏进程，并以此创建一个 PVZ 对象。
    ///
    /// 此方法较复杂，一般情况下请使用 [`attach_pvz`] 宏。
    ///
    /// # Examples
    /// ```rust,no_run
    /// let pvz = PVZ::new_with_window_title("植物大战僵尸中文版").unwrap();
    /// ```
    pub fn new_with_window_title(window_title: &str) -> Result<Self, String> {
        let window_title = WStr::new(window_title);
        unsafe {
            let handle = FindWindowW(WStr::null_cwstr(), window_title.as_cwstr());
            let mut pid = 0;
            GetWindowThreadProcessId(handle, &mut pid);
            if pid != 0 {
                PVZ::new(pid)
            } else {
                Err("未找到符合名称的窗口".to_string())
            }
        }
    }

    /// 运行指定路径的游戏，并以此创建一个 PVZ 对象。
    ///
    /// 此方法较复杂，一般情况下请使用 [`open_pvz`] 宏。
    ///
    /// # Examples
    /// ```rust,no_run
    /// let pvz = PVZ::new_with_executable_path("C:/PlantsVsZombies/PlantsVsZombies.exe").unwrap();
    /// ```
    pub fn new_with_executable_path(executable_path: &str) -> Result<Self, String> {
        let executable_path = Path::new(executable_path)
            .canonicalize()
            .map_err(|err| err.to_string())?;
        let mut command: WStr = executable_path
            .to_str()
            .ok_or(format!("无效路径：{}", executable_path.display()))?
            .into();
        let directory: WStr = executable_path
            .parent()
            .ok_or("获取目录失败")?
            .to_str()
            .ok_or("获取目录失败")?
            .into();

        unsafe {
            let mut process_info = std::mem::zeroed();
            let startup_info = std::mem::zeroed();
            let success = CreateProcessW(
                WStr::null_cwstr(),
                command.as_wstr(),
                std::ptr::null(),
                std::ptr::null(),
                false,
                Default::default(),
                std::ptr::null(),
                directory.as_cwstr(),
                &startup_info,
                &mut process_info,
            )
            .ok();
            match success {
                Ok(()) => {
                    CloseHandle(process_info.hProcess);
                    CloseHandle(process_info.hThread);
                    PVZ::new(process_info.dwProcessId)
                }
                Err(err) => Err(err.message().to_string_lossy()),
            }
        }
    }

    /// 获取进程 ID。
    pub fn get_pid(&self) -> u32 {
        self.memory.get_pid()
    }

    /// 获取基础内存地址。
    unsafe fn get_base_address(&self) -> Result<Address, String> {
        Address::new(PVZ_BASE_ADDRESS).read_address_from(&self.memory)
    }

    /// 获取关卡地址。
    unsafe fn get_level_address(&self) -> Result<Address, String> {
        self.get_base_address()?
            .offset(PVZ_LEVEL_OFFSET)
            .read_address_from(&self.memory)
    }

    /// 获取关卡对象。
    ///
    /// 如果关卡未加载，则返回 `Err(String)`。
    ///
    /// # Examples
    /// ```rust,no_run
    /// let pvz = attach_pvz!();
    /// match pvz.get_level() {
    ///    Ok(level) => {
    ///       println!("当前阳光数：{}", level.get_sun());
    ///    },
    ///    Err(err) => {
    ///        println!("获取关卡失败，原因为：{}", err);
    ///    }
    /// }
    /// ```
    pub fn get_level(&self) -> Result<Level, String> {
        let level_address = unsafe { self.get_level_address()? };
        if level_address.is_null() {
            Err("关卡未启动".to_string())
        } else {
            Ok(Level::new(&self.memory, level_address))
        }
    }
}

/// 打开游戏进程，并返回一个 PVZ 对象。
///
/// # Examples
///
/// 当不指定路径时，默认打开在当前路径下的 `PlantsVsZombies.exe` 文件。
/// ```rust,no_run
/// let pvz = open_pvz!();
/// ```
///
/// 也可以指定路径，如下所示：
/// ```rust,no_run
/// let pvz = open_pvz!("C:/PlantsVsZombies/PlantsVsZombies.exe");
/// ```
#[macro_export]
macro_rules! open_pvz {
    () => {
        PVZ::new_with_executable_path("PlantsVsZombies.exe").unwrap()
    };
    ($executable_path:expr) => {
        PVZ::new_with_executable_path($executable_path).unwrap()
    };
}

/// 附加到一个已打开的游戏进程，并返回一个 PVZ 对象。
///
/// # Examples
///
/// 无参数时，会先查找进程名为 `PlantsVsZombies.exe` 的进程，如果找不到，则改为查找标题为
/// Plants vs. Zombies 的窗口。若两种都找不到，则报错退出。
/// ```rust,no_run
/// let pvz = attach_pvz!();
/// ```
///
/// 可以用如下的语法（类似 Python 的可选参数）指定进程名和窗口名：
/// ```rust,no_run
/// let pvz = attach_pvz!(process_name = "PlantsVsZombies.exe");
/// let pvz = attach_pvz!(window_title = "植物大战僵尸中文版");
/// ```
///
/// 当两者同时指定时，将按照指定的顺序查找。
/// ```rust,no_run
/// let pvz = attach_pvz!(
///     window_title = "植物大战僵尸中文版",     // 先按照窗口名查找
///     process_name = "PlantsVsZombies.exe", // 找不到则再按照进程名查找
/// );
/// ```
#[macro_export]
macro_rules! attach_pvz {
    () => {
        PVZ::new_with_process_name("PlantsVsZombies.exe")
            .or_else(|_| PVZ::new_with_window_title("Plants vs. Zombies"))
            .unwrap()
    };
    (process_name = $process_name:expr) => {
        PVZ::new_with_process_name($process_name).unwrap()
    };
    (process_name = $process_name:expr,) => {
        PVZ::new_with_process_name($process_name).unwrap()
    };
    (window_title = $window_title:expr) => {
        PVZ::new_with_window_title($window_title).unwrap()
    };
    (window_title = $window_title:expr,) => {
        PVZ::new_with_window_title($window_title).unwrap()
    };
    (process_name = $process_name:expr, window_title = $window_title:expr) => {
        PVZ::new_with_process_name($process_name)
            .or_else(|_| PVZ::new_with_window_title($window_title))
            .unwrap()
    };
    (process_name = $process_name:expr, window_title = $window_title:expr,) => {
        PVZ::new_with_process_name($process_name)
            .or_else(|_| PVZ::new_with_window_title($window_title))
            .unwrap()
    };
    (window_title = $window_title:expr, process_name = $process_name:expr) => {
        PVZ::new_with_window_title($window_title)
            .or_else(|_| PVZ::new_with_process_name($process_name))
            .unwrap()
    };
    (window_title = $window_title:expr, process_name = $process_name:expr,) => {
        PVZ::new_with_window_title($window_title)
            .or_else(|_| PVZ::new_with_process_name($process_name))
            .unwrap()
    };
}
