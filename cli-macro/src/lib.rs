extern crate proc_macro;

#[proc_macro_attribute]
pub fn crud_gen(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    cli_macros_impl::do_gen(attr.into(), item.into()).unwrap().into()
}
