use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::borrow::Cow;
use syn::{ConstParam, LifetimeParam, TypeParam};

use crate::{convert_ident_to_case, EnumPatch};
pub(crate) struct EnumConstructionInfo<'s> {
    pub(crate) struct_name: &'s syn::Ident,
    pub(crate) enum_visibility: &'s syn::Visibility,
    pub(crate) enum_name: syn::Ident,
    pub(crate) variants: Vec<EnumConstructionVariant<'s>>,
    pub(crate) passed_attributes: &'s Vec<syn::Attribute>,
    pub(crate) _source: &'s EnumPatch<'s>,
}
impl<'s> EnumConstructionVariant<'s> {
    fn generate_assignments(&self) -> proc_macro2::TokenStream {
        let idents = &self.ident_mappings.iter().map(|v| &v.0).collect::<Vec<_>>();
        quote! {
            #(self.#idents = #idents;)*
        }
    }
    fn generate_maybe_clone_assignments(&self) -> Option<proc_macro2::TokenStream> {
        let mut complete_stream = proc_macro2::TokenStream::new();
        for (ident, ty) in &self.ident_mappings {
            let tokens = if let syn::Type::Reference(r) = ty {
                if r.mutability.is_some() {
                    // mutable references can't be handled properly
                    None
                } else {
                    Some(quote! {self.#ident = #ident;})
                }
            } else {
                Some(quote! {self.#ident = #ident.clone();})
            };
            complete_stream.append_all(tokens?);
        }
        Some(complete_stream)
    }
    fn generate_pattern_matcher_with_idents(&self, enum_name: &syn::Ident) -> proc_macro2::TokenStream {
        self.generate_constructor_with_idents(enum_name)
    }
    fn generate_constructor_with_idents(&self, enum_name: &syn::Ident) -> proc_macro2::TokenStream {
        let idents: Vec<_> = self
            .ident_mappings
            .iter()
            .map(|(ident, _)| ident)
            .collect();
        let variant_name = &self.variant_name;
        quote! {
            #enum_name::#variant_name{
                #(#idents),*
            }
        }
    }
    fn generate_match_arm(&self, enum_name: &syn::Ident) -> proc_macro2::TokenStream {
        let constructor = self.generate_pattern_matcher_with_idents(enum_name);
        let assignments = self.generate_assignments();
        quote! {
            #constructor => {
                #assignments
            }
        }
    }
    fn generate_setter(
        &self,
        enum_name: &syn::Ident,
        // enum_generics: &syn::Generics,
    ) -> Option<proc_macro2::TokenStream> {
        let variant_name = &self.variant_name;
        let lowercase = convert_ident_to_case(variant_name, convert_case::Case::Snake);
        let method_name = format_ident!("modify_{}", lowercase);
        let name_type_pairs: Vec<proc_macro2::TokenStream> = self
            .ident_mappings
            .iter()
            .map(|(ident, ty)| {
                quote! {
                    ,#ident: #ty
                }
            })
            .collect::<Vec<_>>();
        let assignments = self.generate_maybe_clone_assignments()?;
        let constructor = self.generate_constructor_with_idents(enum_name);
        Some(quote! {
            pub fn #method_name(&mut self #(#name_type_pairs)*) -> #enum_name {
                #assignments
                #constructor
            }
        })
    }
}
impl ToTokens for EnumConstructionVariant<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant_name = &self.variant_name;
        let field_iter = &self
            .ident_mappings
            .iter()
            .map(|(ident, ty)| {
                quote! {#ident: #ty}
            })
            .collect::<Vec<_>>();
        let section = quote! {
            #variant_name {
                #(#field_iter),*
            }
        };
        tokens.append_all(section);
    }
}
pub(crate) struct EnumConstructionVariant<'s> {
    pub(crate) variant_name: Cow<'s, syn::Ident>,
    // a mapping from the enum value index to the ident of the member field
    pub(crate) ident_mappings: Vec<(Cow<'s, syn::Ident>, &'s syn::Type)>,
}

impl EnumConstructionInfo<'_> {
    pub(crate) fn generate_enum(&self) -> proc_macro2::TokenStream {
        let enum_name = &self.enum_name;
        let mappings = &self.variants;
        let attrs = self.passed_attributes;
        let visibility = &self.enum_visibility;
        let enum_tokens = quote! {
            #(#attrs)*
            #visibility enum #enum_name {
                #(#mappings),*
            }
        };
        enum_tokens
    }
    pub(crate) fn generate_enum_patch_impl(&self) -> proc_macro2::TokenStream {
        let struct_name = self.struct_name;
        let enum_name = &self.enum_name;
        let field_token_streams = self
            .variants
            .iter()
            .map(|v| v.generate_match_arm(&self.enum_name))
            .collect::<Vec<_>>();
        // let (impl_generics, ty_generics, _where_clause) = self.generics.split_for_impl();
        quote! {
            //TODO
            impl enum_update::EnumUpdate<#enum_name> for #struct_name {
                fn apply(&mut self, patch: #enum_name) {
                    match patch {
                        #(#field_token_streams),*
                    }
                }
            }
        }
    }
    pub(crate) fn generate_setters(&self) -> proc_macro2::TokenStream {
        let struct_name = self.struct_name;
        // let generics = self.generics;
        let methods = self
            .variants
            .iter()
            .map(|v| v.generate_setter(&self.enum_name))
            .collect::<Vec<_>>();
        quote! {
            impl #struct_name {
                #(#methods)*
            }
        }
    }
}
