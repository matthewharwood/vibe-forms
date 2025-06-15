use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, Attribute};

// Helper to check if a field has #[mark] attribute
fn has_mark_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().segments.last().map_or(false, |s| s.ident == "mark")
    })
}

// Helper to get the simple type name (e.g., "Field" from "Field")
fn get_type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) if type_path.qself.is_none() => {
            type_path.path.segments.last().map(|seg| seg.ident.to_string())
        }
        _ => None,
    }
}

// Check if a type is a primitive that should be used directly
fn is_primitive(type_name: &str) -> bool {
    matches!(type_name, "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | 
                       "f32" | "f64" | "bool" | "String" | "str" | "char" | "usize" | "isize")
}

#[proc_macro_derive(FormGen, attributes(mark))]
pub fn form_gen(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let form_name = format_ident!("{}Form", struct_name);

    let Data::Struct(ds) = &input.data else {
        return syn::Error::new_spanned(struct_name, "Only structs supported")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &ds.fields else {
        return syn::Error::new_spanned(struct_name, "Named fields required")
            .to_compile_error()
            .into();
    };

    let mut form_fields = vec![];
    let mut form_to_domain = vec![];
    let mut domain_to_form = vec![];

    // Walk through each field in the struct
    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        if has_mark_attr(&field.attrs) {
            // This field is marked - include it directly in the form
            let type_name = get_type_name(field_type).unwrap_or_default();

            if is_primitive(&type_name) {
                // Primitive marked field - use directly
                form_fields.push(quote!(pub #field_name: #field_type));
                form_to_domain.push(quote!(#field_name: form.#field_name));
                domain_to_form.push(quote!(#field_name: domain.#field_name));
            } else {
                return syn::Error::new_spanned(field_name,
                                               "Marked non-primitive fields not yet supported")
                    .to_compile_error()
                    .into();
            }
        } else {
            // This field is not marked - check if it's a known type with marked fields
            let type_name = get_type_name(field_type).unwrap_or_default();

            if !is_primitive(&type_name) {
                // For custom types, assume they follow the Field pattern
                // In a real implementation, you'd parse the type definition
                if type_name == "Field" {
                    let flat_name = format_ident!("{}_value", field_name);
                    form_fields.push(quote!(pub #flat_name: String));

                    // Generate conversion: Field { value: form.name_value, ..Default::default() }
                    form_to_domain.push(quote!(
                        #field_name: Field {
                            value: form.#flat_name.clone(),
                            ..Default::default()
                        }
                    ));

                    // Generate reverse conversion: name_value: domain.name.value
                    domain_to_form.push(quote!(#flat_name: domain.#field_name.value.clone()));
                }
            }
        }
    }

    // For any fields not handled above, provide defaults
    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let type_name = get_type_name(field_type).unwrap_or_default();

        // If it's not marked and not a Field, provide default
        if !has_mark_attr(&field.attrs) && type_name != "Field" && is_primitive(&type_name) {
            // This is an unmarked primitive - provide default in domain conversion
            if !form_to_domain.iter().any(|token| token.to_string().contains(&field_name.to_string())) {
                form_to_domain.push(quote!(#field_name: Default::default()));
            }
        }
    }

    let expanded = quote! {
        #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
        pub struct #form_name {
            #(#form_fields,)*
        }

        impl From<#form_name> for #struct_name {
            fn from(form: #form_name) -> Self {
                #struct_name {
                    #(#form_to_domain,)*
                    ..Default::default()
                }
            }
        }

        impl From<#struct_name> for #form_name {
            fn from(domain: #struct_name) -> Self {
                #form_name {
                    #(#domain_to_form,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}