mod log_monitor;
use std::{thread, time};

use crate::log_monitor::LogReader;

fn open_log(path: &str) -> std::io::Result<LogReader> {
    LogReader::new(path)
}

fn read_log(log_monitor: &mut LogReader) -> std::io::Result<()> {
    for line in log_monitor {
        println!("{}", line?.trim());
    }
    Ok(())
}

#[allow(dead_code)]
fn read_using_iterator(mut log_mon: &mut LogReader) -> std::io::Result<()> {
    let one_secs = time::Duration::from_secs(1);
    read_log(&mut log_mon)?;
    for _ in 1..20 {
        println!("sleeping...");
        thread::sleep(one_secs);
        read_log(&mut log_mon)?;
    }
    Ok(())
}

#[allow(dead_code)]
fn read_block(log_mon: &mut LogReader) -> std::io::Result<()> {
    let one_secs = time::Duration::from_secs(1);
    let s = log_mon.read()?;
    print!("{}", s);
    for _ in 1..20 {
        // println!("sleeping...");
        thread::sleep(one_secs);
        let s = log_mon.read()?;
        print!("{}", s);
    }
    Ok(())
}

#[allow(dead_code)]
fn read_range(log_mon: &mut LogReader, from: isize, to: isize) -> std::io::Result<()> {
    let s = log_mon.read_range(from, to, true)?;
    print!("{}", s);
    Ok(())
}

fn main()  -> std::io::Result<()> {
    let mut log_mon = open_log("foo.txt")?;
    // read_using_iterator(&mut log_mon)?;
    // read_block(&mut log_mon)?;

    // read_range(&mut log_mon, 1, 4)?;
    // read_range(&mut log_mon, 1, 4)?;

    println!("=== 2:5");
    read_range(&mut log_mon, 2, 5)?;
    println!("=== 1:4");
    read_range(&mut log_mon, 1, 4)?;
    println!("=== 10:14");
    read_range(&mut log_mon, 10, 14)?;
    println!("=== 9:10");
    read_range(&mut log_mon, 9, 10)?;
    println!("=== 10:10");
    read_range(&mut log_mon, 10, 10)?;
    println!("=== 11:11");
    read_range(&mut log_mon, 11, 11)?;
    println!("=== 8:-1");
    read_range(&mut log_mon, 8, -1)?;
    println!("=== 6:-2");
    read_range(&mut log_mon, 6, -2)?;
    println!("=== 8:-2");
    read_range(&mut log_mon, 8, -2)?;
    println!("=== 9:-2");
    read_range(&mut log_mon, 9, -2)?;
    println!("=== 10:-2");
    read_range(&mut log_mon, 10, -2)?;

    Ok(())
}
