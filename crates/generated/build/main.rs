mod blocks;
mod casing;
use crate::blocks::BlockMap;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::{fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var("OUT_DIR")?;
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let manifest_dir = manifest_dir.parent().unwrap().parent().unwrap();

    let blocks_path = manifest_dir.join("generated/reports/blocks.json");
    let blocks = serde_json::from_str::<BlockMap>(&std::fs::read_to_string(blocks_path)?)?;

    let mut mods = Vec::new();
    let mut block_variants = Vec::new();
    let mut to_id_arms = Vec::new();
    let mut from_id_arms = Vec::new();

    for (name, info) in blocks.iter() {
        // let name = "minecraft:acacia_button";
        // let info = blocks.get(name).unwrap();

        let name_pascal = Ident::new(
            &casing::to_pascal_case(name.split(':').last().unwrap()),
            Span::call_site(),
        );

        let name_mod = Ident::new(&name.split(':').last().unwrap(), Span::call_site());

        let prop_enums = info.properties.as_ref().map(|p| {
            p.iter()
                .map(|(k, v)| {
                    let enum_name = Ident::new(&casing::to_pascal_case(k), Span::call_site());
                    let variants = v.iter().map(|v| {
                        let variant_name =
                            Ident::new(&casing::to_pascal_case(v), Span::call_site());
                        quote! {
                            #variant_name,
                        }
                    });
                    quote! {
                        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                        pub enum #enum_name {
                            #(#variants)*
                        }
                    }
                })
                .collect::<Vec<_>>()
        });

        let properties_output = match prop_enums {
            Some(enums) => quote! {
                #(#enums)*
            },
            None => quote! {},
        };

        let block_fields = info.properties.as_ref().map(|p| {
            p.iter().map(|(k, _)| {
                let field_name = process_field(k);
                let enum_name = Ident::new(&casing::to_pascal_case(k), Span::call_site());
                quote! {
                    pub #field_name: #enum_name,
                }
            })
        });

        let block_output = if let Some(fields) = block_fields {
            quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub struct #name_pascal {
                    #(#fields)*
                }
            }
        } else {
            quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub struct #name_pascal;
            }
        };

        let output = quote! {
            pub mod #name_mod {
                #properties_output
                #block_output
            }
        };

        block_variants.push(quote! {
            #name_pascal(#name_mod::#name_pascal),
        });

        // 2. Iterate over the states to generate match arms
        for state in &info.states {
            let id = state.id; // Assuming `id` is a u32 in your deserialized struct

            let prop_assignments = state.properties.as_ref().map(|v| {
                v.iter().map(|(k, v)| {
                    let field_name = process_field(k);
                    let enum_name = Ident::new(&casing::to_pascal_case(k), Span::call_site());
                    let variant_name =
                        Ident::new(&casing::to_pascal_case(v.as_str()), Span::call_site());

                    quote! {
                        #field_name: #name_mod::#enum_name::#variant_name
                    }
                })
            });

            let state_pattern = if let Some(assignments) = prop_assignments {
                quote! {
                    Block::#name_pascal(#name_mod::#name_pascal {
                        #(#assignments),*
                    })
                }
            } else {
                quote! {
                    Block::#name_pascal(#name_mod::#name_pascal)
                }
            };

            to_id_arms.push(quote! { #state_pattern => #id, });
            from_id_arms.push(quote! { #id => Some(#state_pattern), });
        }

        mods.push(output);
    }

    let final_output = quote! {
        #(#mods)*

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Block {
            #(#block_variants)*
        }

        impl Block {
            pub const fn id(&self) -> u32 {
                match self {
                    #(#to_id_arms)*
                }
            }

            pub const fn from_id(id: u32) -> Option<Self> {
                match id {
                    #(#from_id_arms)*
                    _ => None,
                }
            }
        }
    };

    fs::write(
        PathBuf::from(out_dir).join("out.rs"),
        final_output.to_string(),
    )?;

    Ok(())
}

fn process_field(mut field_name: &str) -> Ident {
    if field_name == "type" {
        field_name = "kind";
    }

    Ident::new(field_name, Span::call_site())
}
