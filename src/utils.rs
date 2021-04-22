/// Return whether two vecs contain the same set of elements.
pub fn vec_eq<T: Eq + PartialEq + Ord + PartialOrd + Clone>(a: &Vec<T>, b: &Vec<T>) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_by(|x, y| x.partial_cmp(y).unwrap());
    b.sort_by(|x, y| x.partial_cmp(y).unwrap());
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
