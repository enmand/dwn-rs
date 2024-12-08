mod derive;

use derive::descriptor::impl_descriptor_macro_attr;
use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn descriptor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr);
    let item = parse_macro_input!(item);

    impl_descriptor_macro_attr(attr, item).into()
}
