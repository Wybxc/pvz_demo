# PVZ Demo

使用 Rust 注入并操作 PVZ 游戏的示例性工具。

内部实现参考了 [pvzclass](https://github.com/Lazuplis-Mei/pvzclass)。

```rust
use pvz_demo::*;

fn main() {
    let pvz = attach_pvz!();
    println!("进程ID: {}", pvz.get_pid());
    if let Ok(mut level) = pvz.get_level() {
        level.set_sun(9990).unwrap();
    }
}
```