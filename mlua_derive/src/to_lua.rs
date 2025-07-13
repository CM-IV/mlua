use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

pub fn to_lua(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let fields = if let Data::Struct(data_struct) = input.data {
        match data_struct.fields {
            Fields::Named(fields) => fields,
            _ => panic!("ToLua can only be derived for structs with named fields"),
        }
    } else {
        panic!("ToLua can only be derived for structs");
    };

    let add_field_methods = fields.named.iter().map(|field| {
        let name = &field.ident;
        let name_str = name.as_ref().unwrap().to_string();
        let ty = &field.ty;

        let get_method = quote! {
            fields.add_field_method_get(#name_str, |_, this| {
                use facet_reflect::Peek;

                let peek = Peek::new(&this.#name);
                let vtable = peek.vtable();
                if vtable.is_copy() {
                    Ok(this.#name)
                } else {
                    Ok(this.#name.clone())
                }
            });
        };

        let set_method = quote! {
            fields.add_field_method_set(#name_str, |_, this, val| {
                this.#name = val;
                Ok(())
            });
        };

        quote! {
            #get_method
            #set_method
        }
    });

    let gen = quote! {
        impl ::mlua::UserData for #ident {
            fn add_fields<F: ::mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
                #(#add_field_methods)*
            }
        }
    };

    gen.into()
}
