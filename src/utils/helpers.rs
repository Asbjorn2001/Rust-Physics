pub fn get_pair_mut<T>(vec: &mut Vec<T>, i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j);
    if i < j {
        let (head, tail) = vec.split_at_mut(j);
        (&mut head[i], &mut tail[0])
    } else {
        let (head, tail) = vec.split_at_mut(i);
        (&mut tail[0], &mut head[j])
    }
}