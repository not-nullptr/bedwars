use std::collections::HashMap;

use syn::DeriveInput;

use crate::repr::{NetRepr, Repr};

pub fn macro_impl(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ast: DeriveInput = syn::parse2(input)?;
    let repr = Repr::from_ast(&ast)?;

    match repr {
        Repr::Struct(s) => match &s.data.fields {
            syn::Fields::Named(fields) => {
                let name = &ast.ident;
                let field_reads = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let ty = &field.ty;
                    quote::quote! {
                        #field_name: <#ty as crate::Readable>::read_from(reader).await?,
                    }
                });

                Ok(quote::quote! {
                    impl crate::Readable for #name {
                        async fn read_from<R: ::tokio::io::AsyncRead + ::std::marker::Unpin>(
                            reader: &mut R,
                        ) -> Result<Self, crate::RwError> {
                            // Struct named fields
                            Ok(Self {
                                #(#field_reads)*
                            })
                        }
                    }
                })
            }
            syn::Fields::Unnamed(fields) => {
                let name = &ast.ident;
                // Check if it's a bitflags struct by seeing if it has exactly 1 field
                // with the `bitflags` inner pattern: `<Name as ...PublicFlags>::Internal`
                if fields.unnamed.len() == 1 {
                    let first = &fields.unnamed[0];
                    if let syn::Type::Path(p) = &first.ty {
                        if p.qself.is_some() {
                            if p.path.segments.last().map(|s| s.ident.to_string())
                                == Some("Internal".to_string())
                            {
                                // Bitflags
                                return Ok(quote::quote! {
                                    impl crate::Readable for #name {
                                        async fn read_from<R: ::tokio::io::AsyncRead + ::std::marker::Unpin>(
                                            reader: &mut R,
                                        ) -> Result<Self, crate::RwError> {
                                            let bits = <<Self as bitflags::Flags>::Bits as crate::Readable>::read_from(reader).await?;
                                            Ok(Self::from_bits_retain(bits))
                                        }
                                    }
                                });
                            }
                        }
                    }

                    // Normal tuple struct
                    let field_reads = fields.unnamed.iter().map(|field| {
                        let ty = &field.ty;
                        quote::quote! {
                            <#ty as crate::Readable>::read_from(reader).await?,
                        }
                    });

                    Ok(quote::quote! {
                        impl crate::Readable for #name {
                            async fn read_from<R: ::tokio::io::AsyncRead + ::std::marker::Unpin>(
                                reader: &mut R,
                            ) -> Result<Self, crate::RwError> {
                                Ok(Self(
                                    #(#field_reads)*
                                ))
                            }
                        }
                    })
                } else {
                    let field_reads = fields.unnamed.iter().map(|field| {
                        let ty = &field.ty;
                        quote::quote! {
                            <#ty as crate::Readable>::read_from(reader).await?,
                        }
                    });

                    Ok(quote::quote! {
                        impl crate::Readable for #name {
                            async fn read_from<R: ::tokio::io::AsyncRead + ::std::marker::Unpin>(
                                reader: &mut R,
                            ) -> Result<Self, crate::RwError> {
                                Ok(Self(
                                    #(#field_reads)*
                                ))
                            }
                        }
                    })
                }
            }
            syn::Fields::Unit => {
                let name = &ast.ident;
                Ok(quote::quote! {
                    impl crate::Readable for #name {
                        async fn read_from<R: ::tokio::io::AsyncRead + ::std::marker::Unpin>(
                            _reader: &mut R,
                        ) -> Result<Self, crate::RwError> {
                            Ok(Self)
                        }
                    }
                })
            }
        },

        Repr::Enum(e) => {
            match e.net_repr {
                NetRepr::Type(ty) => {
                    let mut num_map = HashMap::new();
                    for variant in &e.data.variants {
                        let ident = &variant.ident;
                        // look for `Name = 1`
                        let discriminant = variant.discriminant.as_ref().ok_or_else(|| syn::Error::new_spanned(
                            variant,
                            "Enum variants must have a discriminant value if the enum has a #[net_repr(type)] attribute",
                        ))?;

                        let value = match &discriminant.1 {
                            syn::Expr::Lit(expr) => {
                                if let syn::Lit::Int(lit) = &expr.lit {
                                    lit.base10_parse::<i32>().map_err(|_| {
                                        syn::Error::new_spanned(
                                            expr,
                                            "Discriminant values must be integers",
                                        )
                                    })?
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        expr,
                                        "Discriminant values must be integer literals",
                                    ));
                                }
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    discriminant.1.clone(),
                                    "Discriminant values must be integer literals",
                                ));
                            }
                        };

                        num_map.insert(value, ident);
                    }

                    let read_match_arms = num_map.iter().map(|(num, ident)| {
                        quote::quote! {
                            #num => Ok(Self::#ident),
                        }
                    });

                    let ty = &ty;
                    let name = &ast.ident;
                    Ok(quote::quote! {
                        impl crate::Readable for #name {
                            async fn read_from<R: ::tokio::io::AsyncRead + ::std::marker::Unpin>(
                                reader: &mut R,
                            ) -> Result<Self, crate::RwError> {
                                let num = #ty::read_from(reader).await?;
                                let res = match num.into() {
                                    #(#read_match_arms)*
                                    _ => Err(crate::RwError::InvalidEnumDiscriminant(i32::from(num))),
                                }?;

                                Ok(Self::from(res))
                            }
                        }
                    })
                }

                NetRepr::Union(u) => {
                    // union means we read:
                    // - length (VarInt)
                    // - discriminant (VarInt)
                    // - fields (based on discriminant)

                    // each field must be like Variant(Type)

                    let mut read_arms = Vec::new();

                    for variant in &e.data.variants {
                        if variant.fields.len() != 1 {
                            return Err(syn::Error::new_spanned(
                                variant,
                                "Enum variants must have exactly one field when using #[net_repr(union)]",
                            ));
                        }

                        // must be like Variant(Type)
                        let field = &variant.fields.iter().next().unwrap();
                        if field.ident.is_some() {
                            return Err(syn::Error::new_spanned(
                                field,
                                "Enum variant fields must be unnamed when using #[net_repr(union)]",
                            ));
                        }

                        let ty = &field.ty;
                        let ident = &variant.ident;
                        let discriminant = u.get(ident).ok_or_else(|| syn::Error::new_spanned(
                            variant,
                            "All enum variants must have a #[discriminant(value)] attribute when using #[net_repr(union)]",
                        ))?;

                        let read_arm = quote::quote! {
                            #discriminant => {
                                let value = <#ty as Readable>::read_from(&mut reader).await?;
                                Ok(Self::#ident(value))
                            }
                        };

                        read_arms.push(read_arm);
                    }

                    let name = &ast.ident;
                    Ok(quote::quote! {
                        impl crate::Readable for #name {
                            async fn read_from<R: ::tokio::io::AsyncRead + ::std::marker::Unpin>(
                                reader: &mut R,
                            ) -> Result<Self, crate::RwError> {
                                use crate::Readable;

                                let len = crate::varint::VarInt::read_from(reader).await?;
                                let mut reader = ::tokio::io::AsyncReadExt::take(reader, len.value() as u64);
                                let discriminant = crate::varint::VarInt::read_from(&mut reader).await?;

                                match discriminant.value() {
                                    #(#read_arms)*
                                    _ => Err(crate::RwError::InvalidEnumDiscriminant(discriminant.value())),
                                }
                            }
                        }
                    })
                }
            }
        }
    }
}
