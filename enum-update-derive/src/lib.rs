#![warn(missing_docs)]
//! Derive macros for `enum-update`.
//!
//! See the repository README.md for more information.
use proc_macro::TokenStream;
mod parse;
use parse::{convert_ident_to_case, EnumPatch};
mod construction;
use construction::{EnumConstructionInfo, EnumConstructionVariant};
use quote::TokenStreamExt;

/// Generates an enum representing updates to a given struct.
/// See the README.md for an overview.
///
/// The [`EnumUpdate`] macro works by creating a list of "variant groups"
/// with each group representing a modification to the state struct. By
/// default, a variant group is constructed for each struct member.
/// An enum variant is generated for each variant group containing all
/// changes for fields in that group.
///
/// ## Available attributes:
///
/// ### `variant_group`
/// Specifies that a field should belong to an extra group.
/// ```ignore
/// #[derive(EnumUpdate)]
/// pub struct TimeSeriesData {
///     #[variant_group(RecordOne)]
///     time: u32,
///     #[variant_group(RecordOne)]
///     value_one: u32,
///     value_two: u32
/// }
/// ```
/// will generate
/// ```
/// pub enum TimeSeriesDataUpdate {
///     Time(u32),
///     ValueOne(u32),
///     ValueTwo(u32),
///     RecordOne(u32, u32), // <- represents multiple changes
/// }
/// ```
///
/// ### `skip_default`
/// Specifies that a default variant group should not be generated for a field.
/// ```ignore
/// # use enum_update_derive::EnumUpdate;
/// #[derive(EnumUpdate)]
/// pub struct TimeSeriesData {
///     time: u32,
///     value_one: u32,
///     #[skip_default]
///     cached_data
/// }
/// ```
/// will generate
/// ```
/// pub enum TimeSeriesDataUpdate {
///     Time(u32),
///     ValueOne(u32)
/// }
/// ```
///
/// ### `rename_default`
/// Renames the default variant group generated for a field.
/// ```ignore
/// #[derive(EnumUpdate)]
/// pub struct TimeSeriesData {
///     time: u32,
///     value_one: u32,
///     #[rename_default(new_name)]
///     original_name: u32
/// }
/// ```
/// will generate
/// ```
/// pub enum TimeSeriesDataUpdate {
///     Time(u32),
///     ValueOne(u32),
///     NewName(u32), // <-- corresponds to original_name
/// }
/// ```
///
/// ### `enum_update`
/// Passes on any provided attributes to the generated enum.
/// ```ignore
/// #[derive(EnumUpdate)]
/// #[enum_update(derive(Debug))]
/// pub struct State {
///     value: u32
/// }
/// ```
/// will generate
/// ```
/// #[derive(Debug)]
/// pub enum StateUpdate {
///     Value(u32)
/// }
/// ```
#[proc_macro_derive(
    EnumUpdate,
    attributes(variant_group, skip_default, rename_default, enum_update)
)]
pub fn enum_update_derive(inputs: TokenStream) -> TokenStream {
    let parsed = syn::parse(inputs).unwrap();
    let receiver = EnumPatch::from_item_struct(&parsed).unwrap();
    let construction = receiver.to_construction();
    let mut output = construction.generate_enum();
    output.append_all(construction.generate_enum_patch_impl());
    output.into()
}

/// Generates setter methods that also return enum updates.
///
/// Must be used together with [`EnumUpdate`].
/// See the README.md for more examples.
#[proc_macro_derive(
    EnumUpdateSetters,
    attributes(variant_group, skip_default, rename_default, enum_update)
)]
pub fn enum_update_setters_derive(inputs: TokenStream) -> TokenStream {
    let parsed = syn::parse(inputs).unwrap();
    let receiver = EnumPatch::from_item_struct(&parsed).unwrap();
    let construction = receiver.to_construction();
    construction.generate_setters().into()
}
