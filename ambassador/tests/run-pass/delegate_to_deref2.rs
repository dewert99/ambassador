extern crate ambassador;

use ambassador::{delegatable_trait, delegate_to_remote_methods, Delegate};

#[delegatable_trait]
pub trait Shout {
    fn shout(&self, input: &str) -> String;

    fn shout_mut(&mut self) {}
}

#[delegatable_trait]
pub trait Deref {
    fn deref2(&self) {}
}

pub struct Cat;

impl Shout for Cat {
    fn shout(&self, input: &str) -> String {
        format!("{} - meow!", input)
    }
}

impl Deref for Cat {}

pub struct Dog;

impl Shout for Dog {
    fn shout(&self, input: &str) -> String {
        format!("{} - wuff!", input)
    }
}

impl Deref for Dog {}

#[delegate_to_remote_methods]
#[delegate(Deref, target_ref = "deref")]
#[delegate(Shout, target_ref = "deref", target_mut = "deref_mut")]
impl<T: ?Sized> Box<T> {}

#[derive(Delegate)]
#[delegate(Shout)]
pub struct WrappedAnimals(pub Box<dyn Shout>);

fn use_it<T: Shout>(shouter: T) {
    println!("{}", shouter.shout("BAR"));
}

pub fn main() {
    let foo_animal = WrappedAnimals(Box::new(Cat));
    use_it(foo_animal);
}
