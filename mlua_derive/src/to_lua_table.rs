use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn to_lua_table(input: TokenStream) -> TokenStream {
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

    let set_fields = fields.named.iter().map(|field| {
        let name = &field.ident;
        let name_str = name.as_ref().unwrap().to_string();
        quote! {
            table.set(#name_str, self.#name)?;
        }
    });

    let gen = quote! {
        impl ::mlua::IntoLua for #ident {
            fn into_lua(self, lua: &::mlua::Lua) -> ::mlua::Result<::mlua::Value> {
                let table = lua.create_table()?;
                #(#set_fields)*
                Ok(::mlua::Value::Table(table))
            }
        }
    };

    gen.into()
}
