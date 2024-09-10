use manyhow::{ensure, manyhow, Result};
use quote::format_ident;
use quote_use::quote_use as quote;
use syn::{self, Data, DeriveInput, Fields};

static STATE_IGNORE: &str = "state_ignore";

#[manyhow]
#[proc_macro_derive(State, attributes(state_ignore, DefaultState))]
pub fn state_derive(input: DeriveInput) -> Result {
    let name = &input.ident;

    ensure!(let Data::Struct(strct) = &input.data, input, "only structs are supported");

    ensure!(
        let Fields::Named(struct_fields) = &strct.fields,
        strct.fields,
        "only named fields"
    );

    let (field_idents, field_names): (Vec<_>, Vec<_>) = struct_fields
        .named
        .iter()
        .filter(|f| {
            // Ignore all `STATE_IGNORE` attributes
            !f.attrs.iter().any(|attr| attr.path().is_ident(STATE_IGNORE))
        })
        .filter_map(|f| f.ident.as_ref())
        .map(|f| (f, f.to_string()))
        .unzip();

    let component_struct = format_ident!("{}Component", name);

    Ok(quote! {
        # use ::anathema::state::{self, Value, ValueRef, PendingValue, Path, state, Subscriber, CommonVal};
        # use ::std::any::Any;
        impl state::State for #name {
            fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
                let Path::Key(key) = path else { return None };
                match key {
                    #(
                        #field_names => {
                            Some(self.#field_idents.value_ref(sub))
                        }
                    )*
                    _ => None,
                }
            }

            fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
                let Path::Key(key) = path else { return None };
                match key {
                    #(
                        #field_names => {
                            Some(self.#field_idents.to_pending())
                        }
                    )*
                    _ => None,
                }
            }

            fn to_common(&self) -> Option<CommonVal<'_>> {
                None
            }
        }

        struct #component_struct {}

        impl Component for #component_struct {
            type State = #name;
            type Message = ();
        }
    })
}

// #[manyhow]
// #[proc_macro_derive(DefaultState)]
// pub fn default_state_derive(input: DeriveInput) -> Result {
//     let name = &input.ident;
//
//     ensure!(let Data::Struct(strct) = &input.data, input, "only structs are supported");
//
//     ensure!(
//         let Fields::Named(struct_fields) = &strct.fields,
//         strct.fields,
//         "only named fields"
//     );
//
//     let message_struct = format_ident!("{}Message", name);
//     let component_struct = format_ident!("{}Component", name);
//
//     Ok(quote! {
//         # use ::anathema::state::{self, Value, ValueRef, PendingValue, Path, state, Subscriber, CommonVal};
//         # use ::std::any::Any;
//         struct #name {}
//
//         struct #message_struct{}
//
//         impl Component for #component_struct {
//             type State = MyComponentState;
//             type Message = #message_struct;
//         }
//     })
// }
