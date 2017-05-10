use term_painter::ToStyle;
use term_painter::Color::*;


pub fn info(message: &str) {
    println!("{}", NotSet.bold().paint(message));
}

pub fn warn(message: &str) {
    println!("{}", Yellow.bold().paint(message));
}

pub fn success(message: &str) {
    println!("{}", Green.bold().paint(message));
}

pub fn error(message: &str) {
    println!("{}", Red.bold().paint(message));
}
