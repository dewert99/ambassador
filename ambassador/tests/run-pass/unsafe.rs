#![deny(unsafe_op_in_unsafe_fn)]
extern crate ambassador;
use ambassador::{delegatable_trait, Delegate};

#[delegatable_trait]
trait Sleep {
    unsafe fn sleep(&mut self);
}

struct Cat;

impl Sleep for Cat {
    unsafe fn sleep(&mut self) {}
}

#[derive(Delegate)]
#[delegate(Sleep)]
pub struct WrappedCat(Cat);

fn main() {
    unsafe { WrappedCat(Cat).sleep() }
}
