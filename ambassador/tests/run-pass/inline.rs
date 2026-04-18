extern crate ambassador;

use ambassador::{delegatable_trait, Delegate};

#[delegatable_trait]
pub trait Shout {
    fn shout(&self) -> String;
}

#[delegatable_trait(inline = "no")]
pub trait Whisper {
    fn whisper(&self) -> String;
}

#[delegatable_trait(inline = "always")]
pub trait Sing {
    fn sing(&self) -> String;
}

#[delegatable_trait(inline = "never")]
pub trait Dance {
    fn dance(&self) -> String;
}

pub struct Cat;

impl Shout for Cat {
    fn shout(&self) -> String {
        "meow!".to_owned()
    }
}
impl Whisper for Cat {
    fn whisper(&self) -> String {
        "mew".to_owned()
    }
}
impl Sing for Cat {
    fn sing(&self) -> String {
        "la la".to_owned()
    }
}
impl Dance for Cat {
    fn dance(&self) -> String {
        "tap".to_owned()
    }
}

pub struct Dog;

impl Shout for Dog {
    fn shout(&self) -> String {
        "woof!".to_owned()
    }
}
impl Whisper for Dog {
    fn whisper(&self) -> String {
        "wrf".to_owned()
    }
}
impl Sing for Dog {
    fn sing(&self) -> String {
        "howl".to_owned()
    }
}
impl Dance for Dog {
    fn dance(&self) -> String {
        "spin".to_owned()
    }
}

// Default inline (yes) from trait, no override
#[derive(Delegate)]
#[delegate(Shout)]
pub enum Animals {
    Cat(Cat),
    Dog(Dog),
}

// Trait default is "no", but override to "always" at delegation site
#[derive(Delegate)]
#[delegate(Whisper, inline = "always")]
pub enum AnimalsWhisperOverride {
    Cat(Cat),
    Dog(Dog),
}

// Trait default is "always", use it
#[derive(Delegate)]
#[delegate(Sing)]
pub enum AnimalsSing {
    Cat(Cat),
    Dog(Dog),
}

// Trait default is "never", but override to "yes" at delegation site
#[derive(Delegate)]
#[delegate(Dance, inline = "yes")]
pub enum AnimalsDanceOverride {
    Cat(Cat),
    Dog(Dog),
}

// All four inline modes on delegate, with default trait (inline=yes)
#[derive(Delegate)]
#[delegate(Shout, inline = "yes")]
pub struct WrappedCatYes(Cat);

#[derive(Delegate)]
#[delegate(Shout, inline = "no")]
pub struct WrappedCatNo(Cat);

#[derive(Delegate)]
#[delegate(Shout, inline = "always")]
pub struct WrappedCatAlways(Cat);

#[derive(Delegate)]
#[delegate(Shout, inline = "never")]
pub struct WrappedCatNever(Cat);

// Use trait defaults without override (structs)
#[derive(Delegate)]
#[delegate(Whisper)]
pub struct WrappedCatWhisper(Cat);

#[derive(Delegate)]
#[delegate(Sing)]
pub struct WrappedCatSing(Cat);

#[derive(Delegate)]
#[delegate(Dance)]
pub struct WrappedCatDance(Cat);

pub fn main() {
    let a = Animals::Cat(Cat);
    assert_eq!(a.shout(), "meow!");

    let a = AnimalsWhisperOverride::Dog(Dog);
    assert_eq!(a.whisper(), "wrf");

    let a = AnimalsSing::Cat(Cat);
    assert_eq!(a.sing(), "la la");

    let a = AnimalsDanceOverride::Dog(Dog);
    assert_eq!(a.dance(), "spin");

    assert_eq!(WrappedCatYes(Cat).shout(), "meow!");
    assert_eq!(WrappedCatNo(Cat).shout(), "meow!");
    assert_eq!(WrappedCatAlways(Cat).shout(), "meow!");
    assert_eq!(WrappedCatNever(Cat).shout(), "meow!");

    assert_eq!(WrappedCatWhisper(Cat).whisper(), "mew");
    assert_eq!(WrappedCatSing(Cat).sing(), "la la");
    assert_eq!(WrappedCatDance(Cat).dance(), "tap");
}
