use convert_case::{Case, Casing};
use quote::{quote, ToTokens};
use std::{borrow::Cow, collections::HashMap};
use syn::{parse::Parser, ItemStruct};

use crate::{EnumConstructionInfo, EnumConstructionVariant};

fn parse_ident_list(input: syn::parse::ParseStream<'_>) -> syn::Result<Vec<syn::Ident>> {
    let mut ret = vec![input.parse::<syn::Ident>()?];
    while input.parse::<syn::Token![,]>().is_ok() {
        ret.push(input.parse::<syn::Ident>()?);
    }
    Ok(ret)
}
#[derive(Debug)]
pub(crate) struct EnumPatchField<'s> {
    ident: &'s syn::Ident,
    ty: &'s syn::Type,
    variant_groups: Vec<Cow<'s, syn::Ident>>,
}
impl<'s> EnumPatchField<'s> {
    pub(crate) fn from_field(field: &'s syn::Field) -> syn::Result<Self> {
        let mut all_variant_groups: Vec<std::borrow::Cow<'s, syn::Ident>> = vec![];
        let mut default_variant_group: Option<Cow<'s, syn::Ident>> =
            Some(Cow::Borrowed(field.ident.as_ref().unwrap()));
        for attr in &field.attrs {
            if let Some(attr_name) = attr.path().get_ident() {
                match attr_name.to_string().as_str() {
                    "variant_group" => {
                        let extra_group_idents = attr.parse_args_with(parse_ident_list)?;
                        all_variant_groups
                            .extend(extra_group_idents.into_iter().map(std::borrow::Cow::Owned));
                    }
                    "skip_default" => {
                        let _ = default_variant_group.take();
                    }
                    "rename_default" => {
                        let new_ident =
                            attr.parse_args_with(|input: syn::parse::ParseStream<'_>| {
                                input.parse::<syn::Ident>()
                            })?;
                        default_variant_group = Some(Cow::Owned(new_ident));
                    }
                    _ => {}
                }
            }
        }
        if let Some(default_variant) = default_variant_group {
            all_variant_groups.push(default_variant);
        }
        Ok(Self {
            ident: field.ident.as_ref().unwrap(),
            ty: &field.ty,
            variant_groups: all_variant_groups,
        })
    }
}
#[derive(Debug)]
pub(crate) struct EnumPatch<'s> {
    ident: &'s syn::Ident,
    fields: Vec<EnumPatchField<'s>>,
    generics: &'s syn::Generics,
    passed_attributes: Vec<syn::Attribute>,
}

pub(crate) fn convert_ident_to_case(ident: &syn::Ident, case: Case) -> syn::Ident {
    let s = ident.to_string();
    syn::Ident::new(&s.to_case(case), ident.span())
}

impl<'s> EnumPatch<'s> {
    pub(crate) fn from_item_struct(item: &'s ItemStruct) -> syn::Result<Self> {
        let fields: Vec<EnumPatchField> = item
            .fields
            .iter()
            .map(EnumPatchField::from_field)
            .collect::<syn::Result<Vec<EnumPatchField>>>()?;
        let attributes = item
            .attrs
            .iter()
            .map(|attribute| {
                if attribute
                    .path()
                    .get_ident()
                    .is_some_and(|v| v == "enum_update")
                {
                    let mut ts = attribute
                        .meta
                        .require_list()
                        .unwrap()
                        .to_token_stream()
                        .into_iter();
                    // discard enum_update
                    // refer to https://docs.rs/strum_macros/0.26.4/src/strum_macros/macros/enum_discriminants.rs.html
                    let _ = ts.next();
                    let passthrough_group = ts.next().unwrap();
                    let passthrough_attribute = match passthrough_group {
                        proc_macro2::TokenTree::Group(ref group) => group.stream(),
                        _ => {
                            unimplemented!()
                        }
                    };
                    if passthrough_attribute.is_empty() {
                        unimplemented!()
                    }
                    quote! {#[#passthrough_attribute]}
                } else {
                    quote! {}
                }
            })
            .flat_map(|tokenstream| syn::Attribute::parse_outer.parse2(tokenstream).unwrap())
            .collect::<Vec<_>>();
        Ok(Self {
            ident: &item.ident,
            fields,
            generics: &item.generics,
            passed_attributes: attributes,
        })
    }
    pub(crate) fn get_variants(&'s self) -> Vec<EnumConstructionVariant<'s>> {
        // mapping from groups to
        let mut mapping: HashMap<&'s Cow<'s, syn::Ident>, Vec<(&'s syn::Ident, &'s syn::Type)>> =
            HashMap::new();
        for field in &self.fields {
            for grouping in &field.variant_groups {
                if let Some(group_idents) = mapping.get_mut(grouping) {
                    group_idents.push((field.ident, field.ty));
                } else {
                    mapping.insert(grouping, vec![(field.ident, field.ty)]);
                }
            }
        }
        mapping
            .into_iter()
            .map(|v| EnumConstructionVariant {
                variant_name: std::borrow::Cow::Owned(convert_ident_to_case(v.0, Case::Pascal)),
                ident_mappings: v.1.into_iter().map(|v| (Cow::Borrowed(v.0), v.1)).collect(),
            })
            .collect()
    }
    pub(crate) fn to_construction(&'s self) -> EnumConstructionInfo<'s> {
        let variants = self.get_variants();
        let enum_name_string = self.ident.to_string() + "Update";
        let enum_name = syn::Ident::new(&enum_name_string, self.ident.span());
        EnumConstructionInfo {
            struct_name: self.ident,
            enum_name,
            variants,
            generics: self.generics,
            _source: self,
            passed_attributes: &self.passed_attributes,
        }
    }
}

// impl Parse for EnumPatchField {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         Err(syn::Error::new(Span::mixed_site(), "unimplemented"))
//     }
// }
// impl Parse for EnumPatch {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         Err(syn::Error::new(Span::mixed_site(), "unimplemented"))
//     }
// }

// impl EnumPatch {
//     pub(crate) fn to_construction(self) -> EnumConstructionInfo {
//         let pairings: Vec<(&String, syn::Ident)> = self
//             .data
//             .take_struct()
//             .unwrap()
//             .fields
//             .iter()
//             .flat_map(|f| {
//                 f.variant_group
//                     .as_ref()
//                     .map(|v| v.iter().map(|f| f.value()).collect())
//                     .unwrap_or(vec![f.ident.unwrap().to_string()])
//                     .iter()
//                     .map(|group| (group, f.ident.clone().unwrap()))
//             })
//             .collect();
//         let mut mappings: HashMap<&String, Vec<syn::Ident>> = HashMap::new();
//         for pair in pairings {
//             if let Some(inner) = mappings.get_mut(&pair.0) {
//                 inner.push(pair.1);
//             } else {
//                 mappings.insert(pair.0, vec![pair.1]);
//             }
//         }
//         let variants: Vec<EnumConstructionVariant> = mappings
//             .into_iter()
//             .map(|(key, value)| EnumConstructionVariant {
//                 name: syn::Ident::new(&key, Span::call_site()),
//                 ident_mappings: value,
//             })
//             .collect();
//         let mut enum_name = self.ident.to_string();
//         enum_name.push_str("Update");
//         let construction = EnumConstructionInfo {
//             name: enum_name,
//             variants,
//         };
//         construction
//     }
// }
