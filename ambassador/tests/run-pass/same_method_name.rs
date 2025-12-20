extern crate ambassador;

use ambassador::{delegatable_trait, Delegate};

#[delegatable_trait]
pub trait Shout1 {
    fn shout(&self, input: &str) -> String;
}

#[delegatable_trait]
pub trait Shout2 {
    fn shout(&self);
}

pub struct Cat;

impl Shout1 for Cat {
    fn shout(&self, input: &str) -> String {
        format!("{} - meow!", input)
    }
}

impl Shout2 for Cat {
    fn shout(&self) {
        println!("meow!")
    }
}

#[derive(Delegate)]
#[delegate(Shout1)]
#[delegate(Shout2)]
struct WrappedCat(Cat);

fn main() {
    let cat = WrappedCat(Cat);
    println!("{}", Shout1::shout(&cat, ""));
    Shout2::shout(&cat);
}
