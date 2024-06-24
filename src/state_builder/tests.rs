use essential_types::intent::Directive;

use super::*;

#[test]
fn test_name() {
    let state = StateBuilder::new()
        .with_contract(
            "a",
            &[Intent {
                state_read: Default::default(),
                constraints: Default::default(),
                directive: Directive::Satisfy,
            }],
        )
        .with_contract(
            "b",
            &[Intent {
                state_read: Default::default(),
                constraints: vec![vec![1]],
                directive: Directive::Satisfy,
            }],
        )
        .push("a")
        .with_index(10)
        .with_b256([0; 4])
        .with_index(1)
        .with_map_int(2)
        .with_tuple(2)
        .with_map_int(3)
        .with_bool(true)
        .with_index(3)
        .with_tuple(6)
        .with_int(12)
        .pop()
        .push("b")
        .with_index(5)
        .with_bool(true);
    let a = state.intents.get("a").unwrap().clone();
    let b = state.intents.get("b").unwrap().clone();

    let state = state.build();

    assert_eq!(state.len(), 2);
    let state_a = state.get(&a).unwrap();
    let state_b = state.get(&b).unwrap();
    assert_eq!(state_a.len(), 3);
    assert_eq!(state_b.len(), 1);

    assert_eq!(*state_a.get(&vec![10]).unwrap(), vec![0; 4]);
    assert_eq!(*state_a.get(&vec![1, 2, 2, 3]).unwrap(), vec![1]);
    assert_eq!(*state_a.get(&vec![3, 6]).unwrap(), vec![12]);

    assert_eq!(*state_b.get(&vec![5]).unwrap(), vec![1]);
}
