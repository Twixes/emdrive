/// Equivalent to `println!`, just with a local time marker prepended.
#[macro_export]
macro_rules! timeprintln {
    () => (println!("{}", chrono::Local::now().format("[%T%.3f]")));
    ($($arg:tt)*) => ({
        print!("{} ", chrono::Local::now().format("[%T%.3f]"));
        println!($($arg)*);
    })
}

/// Equivalent to `eprintln!`, just with a local time marker prepended.
#[macro_export]
macro_rules! etimeprintln {
    () => (eprintln!("{}", chrono::Local::now().format("[%T%.3f]")));
    ($($arg:tt)*) => ({
        eprint!("{} ", chrono::Local::now().format("[%T%.3f]"));
        eprintln!($($arg)*);
    })
}
