#![warn(missing_docs)]
//! Derive macros for `enum-update`. 
//! 
//! See the repository README.md for more information.
use proc_macro::TokenStream;
mod parse;
use parse::{EnumPatch, convert_ident_to_case};
mod construction;
use construction::{EnumConstructionInfo, EnumConstructionVariant};
use quote::TokenStreamExt;

/// Generates an enum representing updates to a given struct
/// 
/// The provided struct must have named fields. See the README.md for
/// more examples.
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
