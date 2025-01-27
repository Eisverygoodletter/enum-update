use enum_update::{ EnumUpdate, EnumUpdateSetters };

#[derive(Debug, EnumUpdate, Clone, EnumUpdateSetters)]
#[enum_update(derive(Debug, Clone, PartialEq))]
pub struct SharedState {
    managed_by_first: bool,
}

#[test]
fn communicating_threads() {
    let initial_state = SharedState {
        managed_by_first: false,
    };
    let (one_to_two, two_recv) = std::sync::mpsc::sync_channel(1);
    let mut thread_one_state = initial_state.clone();
    let mut thread_two_state = initial_state.clone();
    let thread_one = std::thread::Builder::new()
        .spawn(move || {
            let change = thread_one_state.modify_managed_by_first(true);
            one_to_two.send(change).unwrap();
        })
        .unwrap();
    let thread_two = std::thread::Builder::new()
        .spawn(move || {
            assert!(!thread_two_state.managed_by_first);
            // now, we receive the change
            let change = two_recv.recv().unwrap();
            assert_eq!(change, SharedStateUpdate::ManagedByFirst(true));
            // applying the change
            thread_two_state.apply(change);
            // it becomes true
            assert!(thread_two_state.managed_by_first);
        })
        .unwrap();
    thread_one.join().unwrap();
    thread_two.join().unwrap();
}
