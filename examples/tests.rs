#![feature(never_type)]

use anonymous_enums::*;

fn main() {
    let _: u32 = OneOf::<(u32, )>::new(0).take().infallible();
    assert!(OneOf::<(u32, i32)>::new(0u32).take::<i32>().is_err());
    let e = OneOf::<(u32, u64, i32, i64)>::new(123u64);
    let r = match_type! { e in
        u32 as u => {
            println!("u32 = {}", u);
            u as f64
        }
        u64 as u => {
            println!("u64 = {}", u);
            u as f64
        }
        i32 as i => {
            println!("i32 = {}", i);
            i as f64
        }
        i64 as i => {
            println!("i64 = {}", i);
            i as f64
        }
    };
    println!("f64 = {}", r);

    let t = OneOf::<(u32, u64)>::new(0u32);
    let t: OneOf<(u32, u64, i32, i64)> = t.into();
    let t: OneOf<(u32, u64)> = t.into();
}