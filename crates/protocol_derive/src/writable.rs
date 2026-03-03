use crate::repr::Repr;

pub fn macro_impl(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ast: syn::DeriveInput = syn::parse2(input)?;
    let repr = Repr::from_ast(&ast)?;
    todo!()
}
