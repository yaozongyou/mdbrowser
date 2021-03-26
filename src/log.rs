#[macro_export]
macro_rules! message {
    ($($arg:tt)*) => {
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut s = String::from(format!("[{} {} {}:{}] ", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), gettid::gettid(),  file!(), line!()));
        s += &String::from(format!($($arg)*));

        if let Ok(mut f) = OpenOptions::new().write(true).append(true).create(true).open("/tmp/debug.txt") {
            writeln!(f, "{}", s).unwrap();
        }
    }
}

#[macro_export]
macro_rules! backtrace {
    () => {};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        message!("{} {}", "aaaa", 123);
        assert_eq!(2 + 2, 4);
    }
}
