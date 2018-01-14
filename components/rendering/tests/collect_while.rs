extern crate rendering;

use rendering::collect_while::collect_while;

#[test]
fn test_normal_order() {
    let nums = vec![1, 2, 3, 4, 5, 6];
    let mut iter = nums.into_iter().peekable();
    let first_three: Vec<_> = collect_while(&mut iter, |&i| i < 4 );
    assert_eq!(vec![1, 2, 3],
               first_three);
    assert_eq!(vec![4, 5, 6],
               iter.collect::<Vec<u8>>());
}

#[test]
fn test_reverse_order() {
    let nums = vec![1, 2, 3, 4, 5, 6];
    let mut iter = nums.into_iter().peekable();
    let first_three: Vec<_> = collect_while(&mut iter, |&i| i < 4 );
    assert_eq!(vec![4, 5, 6],
               iter.collect::<Vec<u8>>());
    assert_eq!(vec![1, 2, 3],
               first_three);
}
