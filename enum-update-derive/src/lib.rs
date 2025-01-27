use proc_macro::TokenStream;
mod parse;
use parse::*;
mod construction;
use construction::*;
use quote::{ToTokens, TokenStreamExt};

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
