
#![cfg(test)]

use super::*;

#[test]
fn test_vec_list() {
    let list = VecList::<u8>::new();
    assert_eq!(list.len(), 0, "`VecList::new` initialised non empty",);
    let capacity = 10;
    let mut list = VecList::with_capacity(capacity);

    assert_eq!(list.capacity(), capacity, "`VecList::with_capacity` initialised with wrong capacity",);
    assert_eq!(list.front(), None, "`VecList::front` initialised with front value",);
    assert_eq!(list.back(), None, "`VecList::back` initialised with front value",);

    list.push_front(1);
    assert_eq!(list.front(), Some(&1), "`VecList::push_front` did not push value",);

    list.push_back(2);
    assert_eq!(list.back(), Some(&2), "`VecList::push_back` did not push value",);
    
    list.push_front(0);
    assert_eq!(list.front(), Some(&0), "`VecList::push_front` did not push value to correct end",);
    assert_eq!(list.iter().collect::<Vec<_>>(), vec![&0, &1, &2,],
        "`VecList::iter` did not iterate correctly.",
    );
    assert_eq!(list.iter().rev().collect::<Vec<_>>(), vec![&2, &1, &0,],
        "`VecList::iter` did not iterate correctly.",
    );

    assert_eq!(list.len(), 3, "`VecList::len` length was not tracked across pushes properly",);

    assert_eq!(list.pop_front(), Some(0), "`VecList::pop_front` did not pop the correct value",);
    assert_eq!(list.pop_back(), Some(2), "`VecList::pop_back` did not pop the correct value",);
    assert_eq!(list.len(), 1, "`VecList::len` length was not tracked across pops properly",);

    list.shrink_to_fit();
    assert_eq!(list.len(), 1,
        "`VecList::shink_to_fit` altered length",
    );
    assert_eq!(list.len(), list.capacity(),
        "`VecList::shink_to_fit` did not shink to minimum size",
    );

    let mut list2 = VecList::<i32>::from_iter(0..=3);
    
    list2.retain(|&i: &i32| i % 2 == 0);
    assert_eq!(list2.iter().collect::<Vec<_>>(), vec![&0, &2,],
        "`VecList::retain` did not retain proper values"
    );

    list2.clear();
    assert!(list2.is_empty(), "`VecList::clear` did not clear all values");
    
    ::std::mem::forget(list2);
    ::std::mem::forget(list);
}

#[test]
fn test_vec_list_iter() {
    let list = VecList::<u8>::from_iter(0..=3);
    
    assert_eq!(list.iter().collect::<Vec<_>>(), vec![&0, &1, &2, &3,],
        "`VecList::iter` did not iterate correctly.",
    );
    assert_eq!(list.iter().rev().collect::<Vec<_>>(), vec![&3, &2, &1, &0,],
        "`VecList::iter` did not iterate correctly.",
    );
}

#[test]
fn test_vec_list_iter_mut() {
    let mut list = VecList::<u8>::from_iter(0..=3);
    
    for i in list.iter_mut() { *i += 1 }
    assert_eq!(list.iter().collect::<Vec<_>>(), vec![&1, &2, &3, &4,],
        "`VecList::iter_mut` did not iterate correctly.",
    );
}

#[test]
fn test_vec_list_drain() {
    let mut list = VecList::<u8>::from_iter(0..=3);
    
    assert_eq!(list.clone().drain(1..=2).rev().collect::<Vec<_>>(), vec![2, 1,],
        "`VecList::drain` did not iterate backwards correctly.",
    );
    assert_eq!(list.drain(1..=2).collect::<Vec<_>>(), vec![1, 2,],
        "`VecList::drain` did not iterate correctly.",
    );
    assert_eq!(list.iter().collect::<Vec<_>>(), vec![&0, &3,],
        "`VecList::drain` did not retain the correct value.",
    );
}

#[test]
fn test_vec_list_drain_filter() {
    let mut list = VecList::<u8>::from_iter(0..=3);
    let filter = |i: &mut u8| { *i += 1; *i % 2 == 0 };

    assert_eq!(list.clone().drain_filter(filter).rev().collect::<Vec<_>>(), vec![4, 2,],
        "`VecList::drain_filter` did not iterate backward correctly.",
    );
    assert_eq!(list.drain_filter(filter).collect::<Vec<_>>(), vec![2, 4,],
        "`VecList::drain_filter` did not iterate correctly.",
    );
    assert_eq!(list.iter().collect::<Vec<_>>(), vec![&1, &3,],
        "`VecList::drain_filter` did not retain the correct values.",
    );
    assert_eq!(list.drain_filter(filter).collect::<Vec<_>>(), vec![2, 4,],
        "`VecList::drain_filter` did not iterate correctly the second time.",
    );
}

#[test]
fn test_view() {
    let list = {
        let mut list = VecList::<u8>::new();

        list.extend(0..=10);
        list
    };
    
    assert_eq!(list.view(10), &10, "`VecList::view` got wrong value 1",);
    assert_eq!(list.view(0), &0, "`VecList::view` got wrong value 2",);

    let view = list.view(3);
    assert_eq!(view, &3, "`VecList::view` got wrong value 3",);
    assert_eq!(view.next().expect("Failed to get next value"), &4,
        "`View::next` got wrong value",
    );
    assert_eq!(view.prev().expect("Failed to get prev value"), &2,
        "`View::prev` got wrong value",
    );
}

#[test]
fn test_view_mut() {
    let mut list = VecList::<u8>::new();

    list.extend(0..=10);
    
    assert_eq!(list.view_mut(10), &10, "`VecList::view` got wrong value 1",);
    assert_eq!(list.view_mut(0), &0, "`VecList::view` got wrong value 2",);

    let mut view = list.view_mut(3);
    assert_eq!(view, &3, "`VecList::view` got wrong value 3",);
    assert_eq!(*view.next().expect("Failed to get next value"), &4,
        "`View::next` got wrong value",
    );
    assert_eq!(*view.prev().expect("Failed to get prev value"), &3,
        "`View::prev` got wrong value",
    );
    
    let value = view.pop_before()
        .expect("`VecList::pop_before` did not return a value");
    assert_eq!(value, 2, "`VecList::pop_before` did not return the expected value",);
    view.insert_before(value);
    assert_eq!(*view.prev().expect("Failed to get prev value"), &2,
        "`View::insert_before` inserted value wrong",
    );
    
    let value = view.pop_after()
        .expect("`VecList::pop_after` did not return a value");
    assert_eq!(value, 3, "`VecList::pop_after` did not return the expected value",);
    view.insert_after(value);
    assert_eq!(*view.next().expect("Failed to get ext value"), &3,
        "`View::insert_after` inserted value wrong",
    );
}
