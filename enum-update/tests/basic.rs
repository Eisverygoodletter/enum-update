use enum_update_derive::{EnumUpdate, EnumUpdateSetters};

#[derive(Debug, EnumUpdate, EnumUpdateSetters, PartialEq)]
#[enum_update(derive(Debug, PartialEq))]
pub struct TestStruct {
    #[variant_group(UpdateBoth)]
    test: String,
    #[variant_group(UpdateBoth)]
    test2: i32,
}
pub struct UnitStruct;

#[test]
fn test() {
    let mut state = TestStruct {
        test: "hello".to_string(),
        test2: 123,
    };
    assert_eq!(
        state.modify_update_both("".to_string(), 0),
        TestStructUpdate::UpdateBoth {
            test: "".to_string(),
            test2: 0
        }
    );
    assert_eq!(state.test, "".to_string());
    assert_eq!(state.test2, 0);
}
