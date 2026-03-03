mod readable;
mod repr;
mod writable;

#[proc_macro_derive(Readable, attributes(net_repr))]
pub fn readable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    readable::macro_impl(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Writable, attributes(net_repr))]
pub fn writable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    writable::macro_impl(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
