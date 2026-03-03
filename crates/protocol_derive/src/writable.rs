use crate::repr::{NetRepr, Repr};

pub fn macro_impl(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ast: syn::DeriveInput = syn::parse2(input)?;
    let repr = Repr::from_ast(&ast)?;

    match repr {
        Repr::Struct(s) => {
            let field_writes = s.data.fields.iter().map(|field| {
                let name = &field.ident;
                quote::quote! {
                    self.#name.write_into(writer).await?;
                }
            });

            let name = &ast.ident;
            Ok(quote::quote! {
                impl crate::Writable for #name {
                    async fn write_into<W: ::tokio::io::AsyncWrite + ::std::marker::Unpin>(
                        &self,
                        writer: &mut W,
                    ) -> Result<(), crate::RwError> {
                        #(#field_writes)*
                        Ok(())
                    }
                }
            })
        }

        Repr::Enum(e) => match e.net_repr {
            crate::repr::NetRepr::Type(ty) => {
                let write_match_arms = e.data.variants.iter().map(|variant| {
                    let ident = &variant.ident;
                    let discriminant = variant.discriminant.as_ref().unwrap().1.clone();
                    quote::quote! {
                        // Self::#ident => #discriminant.write_into(writer).await?,
                        Self::#ident => <#ty as crate::Writable>::write_into(&(#discriminant.into()), writer).await?,
                    }
                });

                let name = &ast.ident;
                Ok(quote::quote! {
                    impl crate::Writable for #name {
                        async fn write_into<W: ::tokio::io::AsyncWrite + ::std::marker::Unpin>(
                            &self,
                            writer: &mut W,
                        ) -> Result<(), crate::RwError> {
                            match self {
                                #(#write_match_arms)*
                            }
                            Ok(())
                        }
                    }
                })
            }

            NetRepr::Union(u) => {
                let write_match_arms = e.data.variants.iter().map(|variant| {
                    let ident = &variant.ident;
                    let discriminant = u.get(ident).unwrap();
                    quote::quote! {
                        Self::#ident(inner) => {
                            let mut buf = Vec::new();
                            inner.write_into(&mut buf).await?;
                            let packet_id = crate::varint::VarInt::from(#discriminant);
                            // len should be the number of bytes in packet_id + the number of bytes in the inner data
                            let len = crate::varint::VarInt::from((packet_id.len_bytes() + buf.len()) as u32);
                            len.write_into(writer).await?;
                            packet_id.write_into(writer).await?;
                            writer.write_all(&buf).await?;
                        }
                    }
                });

                let name = &ast.ident;
                Ok(quote::quote! {
                    impl crate::Writable for #name {
                        async fn write_into<W: ::tokio::io::AsyncWrite + ::std::marker::Unpin>(
                            &self,
                            writer: &mut W,
                        ) -> Result<(), crate::RwError> {
                            use ::tokio::io::AsyncWriteExt;
                            match self {
                                #(#write_match_arms)*
                            }
                            Ok(())
                        }
                    }
                })
            }
        },
    }
}
