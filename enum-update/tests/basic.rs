use enum_update_derive::{EnumUpdate, EnumUpdateSetters};

#[derive(EnumUpdate, EnumUpdateSetters)]
pub struct TestStruct<'a, T: Clone> {
    #[variant_group(UpdateBoth)]
    test: String,
    #[rename_default(ADifferentName)]
    #[variant_group(UpdateBoth)]
    test2: i32,
    generics_included: Box<i32>,
    a_reference: &'a str,
    custom_value: T,
    ref_and_custom: &'a T,
    a_mutable: &'a mut i32,
}
pub struct UnitStruct;

#[test]
fn tes() {
    let mut referenced = 123;
    let mut state = TestStruct {
        test: "hello".to_string(),
        test2: 123,
        generics_included: Box::new(456),
        a_reference: "world",
        custom_value: (),
        ref_and_custom: &(),
        a_mutable: &mut referenced,
    };
    state.modify_custom_value(());

    //hello
}
