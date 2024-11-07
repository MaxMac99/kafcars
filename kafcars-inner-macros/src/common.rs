use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::Type;

#[derive(FromAttributes)]
#[darling(attributes(kafka))]
pub struct FieldOptions {
    #[darling(default)]
    pub min_version: Option<i16>,
    #[darling(default)]
    pub max_version: Option<i16>,
    #[darling(default)]
    pub tag: Option<i16>,
    #[darling(default)]
    pub default: Option<syn::ExprPath>,
    #[darling(default)]
    pub serialize_with: Option<syn::ExprPath>,
    #[darling(default)]
    pub deserialize_with: Option<syn::ExprPath>,
}

pub fn darling_to_syn(e: darling::Error) -> Vec<syn::Error> {
    let msg = format!("{e}");
    let token_errors = e.write_errors();
    vec![syn::Error::new(token_errors.span(), msg)]
}

pub fn to_compile_errors(errors: Vec<syn::Error>) -> TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}



pub fn option_type(ty: &Type) -> Option<&Type> {
    let Type::Path(ty) = ty else { return None };
    if ty.qself.is_some() {
        return None;
    }

    let ty = &ty.path;

    if ty.segments.is_empty() || ty.segments.last().unwrap().ident.to_string() != "Option" {
        return None;
    }

    if !(ty.segments.len() == 1
        || (ty.segments.len() == 3
        && ["core", "std"].contains(&ty.segments[0].ident.to_string().as_str())
        && ty.segments[1].ident.to_string() == "option"))
    {
        return None;
    }

    let last_segment = ty.segments.last().unwrap();
    let syn::PathArguments::AngleBracketed(generics) = &last_segment.arguments else {
        return None;
    };
    if generics.args.len() != 1 {
        return None;
    }
    let syn::GenericArgument::Type(inner_type) = &generics.args[0] else {
        return None;
    };

    Some(inner_type)
}
