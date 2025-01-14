pub fn print_dbg_address(addr: isize, friendly_name: &str) {
    let q = region::query(addr as *const ()).unwrap();

    if q.is_executable() {
        println!("{friendly_name} is executable")
    } else {
        println!("{friendly_name} is NOT executable")
    }
}

pub fn print_vec(v: &Vec<u8>) {
    for (i, byte) in v.iter().enumerate() {
        print!("{byte:02X} ");

        if (i + 1) % 16 == 0 {
            println!();
        }
    }
}
