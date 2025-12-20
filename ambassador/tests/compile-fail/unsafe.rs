extern crate ambassador;
use ambassador::{delegatable_trait, Delegate};

#[delegatable_trait] //~ ERROR call to unsafe function `Cat::sleep` is unsafe
trait Sleep {
    fn sleep(&mut self);
}

#[derive(Delegate)]
#[delegate(Sleep, target = "self")]
struct Cat;

impl Cat {
    unsafe fn sleep(&mut self) {}
}

fn main() {
    Sleep::sleep(&mut Cat)
}
