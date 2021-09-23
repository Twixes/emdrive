/// Equivalent to `println!`, just with a local time marker prepended.
#[macro_export]
macro_rules! timeprintln {
    () => (println!("{}", chrono::Local::now().format("[%H:%M:%S]")));
    ($($arg:tt)*) => ({
        print!("{}", chrono::Local::now().format("[%H:%M:%S] "));
        println!($($arg)*);
    })
}

/// Equivalent to `eprintln!`, just with a local time marker prepended.
#[macro_export]
macro_rules! etimeprintln {
    () => (eprintln!("{}", chrono::Local::now().format("[%H:%M:%S]")));
    ($($arg:tt)*) => ({
        eprint!("{}", chrono::Local::now().format("[%H:%M:%S] "));
        eprintln!($($arg)*);
    })
}
