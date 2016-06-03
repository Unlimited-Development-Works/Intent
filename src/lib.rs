pub mod intent_parser;
pub mod ast;

#[macro_use]
extern crate clap;
use clap::{ Arg, App, ArgGroup };

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
}
