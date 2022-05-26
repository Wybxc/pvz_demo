use pvz_demo::*;

fn main() {
    let pvz = attach_pvz!();
    println!("进程ID: {}", pvz.get_pid());
    if let Ok(mut level) = pvz.get_level() {
        level.set_sun(9990).unwrap();
    }
}
