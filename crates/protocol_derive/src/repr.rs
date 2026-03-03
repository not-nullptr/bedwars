use std::collections::HashMap;

pub enum Repr {
    Struct(StructRepr),
    Enum(EnumRepr),
}

pub struct StructRepr {
    pub data: syn::DataStruct,
}

pub struct EnumRepr {
    pub data: syn::DataEnum,
    pub net_repr: NetRepr,
}

pub enum NetRepr {
    Union(FieldMap),
    Type(syn::Type),
}

pub type FieldMap = HashMap<syn::Ident, syn::Expr>; // map of field name to discriminant value

impl Repr {
    pub fn from_ast(ast: &syn::DeriveInput) -> Result<Self, syn::Error> {
        let net_repr = ast.attrs.iter().find_map(|attr| {
            if attr.path().is_ident("net_repr") {
                Some(attr.parse_args::<syn::Type>().ok()?)
            } else {
                None
            }
        });

        match &ast.data {
            syn::Data::Struct(data) => Ok(Self::Struct(StructRepr { data: data.clone() })),
            syn::Data::Enum(data) => {
                let net_repr = match net_repr {
                    Some(ty) => NetRepr::Type(ty),
                    None => {
                        let mut field_map = HashMap::new();
                        // get the #[discriminant(value)] attribute for each variant
                        for variant in &data.variants {
                            let ident = &variant.ident;
                            let discriminant = variant.attrs.iter().find_map(|attr| {
                                if attr.path().is_ident("discriminant") {
                                    Some(attr.parse_args::<syn::Expr>().ok()?)
                                } else {
                                    None
                                }
                            }).ok_or_else(|| syn::Error::new_spanned(
                                variant,
                                "Enum variants must have a #[discriminant(value)] attribute if no #[net_repr(type)] is specified",
                            ))?;

                            field_map.insert(ident.clone(), discriminant);
                        }
                        NetRepr::Union(field_map)
                    }
                };

                Ok(Self::Enum(EnumRepr {
                    data: data.clone(),
                    net_repr,
                }))
            }
            _ => Err(syn::Error::new_spanned(
                ast,
                "Readable/Writable can only be derived for structs and enums",
            )),
        }
    }
}
