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

/// Return whether two vecs contain the same set of elements.
pub fn vec_eq<T: PartialEq + PartialOrd + Clone>(a: &[T], b: &[T]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_by(|x, y| x.partial_cmp(y).unwrap());
    b.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

/// Return whether two vecs contain the same set of elements with the same positions.
pub fn vec_eq_exact<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_eq_works() {
        assert!(vec_eq::<u32>(&vec![], &vec![]));
        assert!(!vec_eq(&vec![], &vec![2]));
        assert!(vec_eq(&vec![2, 3], &vec![2, 3]));
        assert!(vec_eq(&vec![2, 3], &vec![3, 2]));
        assert!(vec_eq(&vec!["a", "b", "c"], &vec!["a", "b", "c"]));
    }
}
