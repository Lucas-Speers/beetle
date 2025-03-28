use std::env;

pub fn get_arg() -> Option<String> {
    let mut args = env::args();
    args.next(); // beetle
    args.next()
}