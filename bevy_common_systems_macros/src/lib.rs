use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(EventKind, attributes(event_name, event_info))]
pub fn derive_event_kind(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string().to_lowercase();

    let mut event_name = quote! { #name_str };
    // NOTE: `()` is the default payload for an event with no `#[event_info(...)]`:
    // it satisfies the `EventKind::Info` bounds (Serialize + Default + Clone + Debug)
    // and needs no import at the derive site. Do not name a concrete type here -- the
    // original default named one that neither resolved nor implemented Serialize, so
    // the attribute-less derive never compiled (guarded by
    // `attribute_less_derive_defaults_to_no_payload` in `modding::events`).
    let mut event_info = quote! { () };

    for attr in &input.attrs {
        if attr.path().is_ident("event_name") {
            if let Ok(syn::Lit::Str(s)) = &attr.parse_args() {
                let s = s.value();
                event_name = quote! { #s };
            }
        } else if attr.path().is_ident("event_info") {
            if let Ok(syn::TypePath { path, .. }) = &attr.parse_args() {
                event_info = quote! { #path };
            }
        }
    }

    let expanded = quote! {
        impl EventKind for #name {
            type Info = #event_info;

            fn name() -> &'static str {
                #event_name
            }
        }
    };

    TokenStream::from(expanded)
}
