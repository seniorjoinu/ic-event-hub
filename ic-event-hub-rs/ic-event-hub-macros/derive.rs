use proc_macro::TokenStream;
use std::collections::BTreeSet;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse, Data, DeriveInput, Fields, Ident as SynIdent, Type};

pub fn event_macro_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse(input).unwrap();

    let name = &ast.ident;
    let name_str = name.to_string();

    let filter_name = SynIdent::new(&format!("{}Filter", name), Span::call_site());

    let mut topics: Vec<(Ident, Type, String)> = vec![];
    let mut values: Vec<(Ident, String)> = vec![];

    match ast.data {
        Data::Struct(ref data_struct) => {
            if let Fields::Named(ref fields_named) = data_struct.fields {
                for field in fields_named.named.iter() {
                    let field_attrs: BTreeSet<String> = field
                        .attrs
                        .iter()
                        .map(|attr| {
                            attr.parse_meta()
                                .unwrap()
                                .path()
                                .get_ident()
                                .unwrap()
                                .to_string()
                        })
                        .collect();
                    let item = field.ident.clone().unwrap();

                    if field_attrs.contains("topic") {
                        topics.push((item.clone(), field.ty.clone(), item.to_string()))
                    } else {
                        values.push((item.clone(), item.to_string()))
                    }
                }
            }
        }

        _ => panic!("Must be a struct"),
    }

    // Transform marked elements into new struct fields
    let topics_event_ser = topics.iter().fold(quote!(), |es, (field, _, field_name)| {
        quote! {
            #es res.insert(ic_event_hub::types::EventField {
                name: String::from(#field_name),
                value: ic_cdk::export::candid::encode_one(&self.#field).unwrap()
            });
        }
    });

    let topics_event_de = topics.iter().fold(quote!(), |es, (field, _, field_name)| {
        quote! {
            #es #field: ic_cdk::export::candid::decode_one(fields.get(#field_name).unwrap()).unwrap(),
        }
    });

    let topics_filter = topics.iter().fold(quote!(), |ts, (field, field_type, _)| {
        quote! {
            #ts pub #field: Option<#field_type>,
        }
    });

    let topics_filter_ser = topics.iter().fold(quote!(), |es, (field, _, field_name)| {
        quote! {
            #es
            if let Some(value) = &self.#field {
                res.insert(ic_event_hub::types::EventField {
                    name: String::from(#field_name),
                    value: ic_cdk::export::candid::encode_one(value).unwrap()
                });
            }
        }
    });

    let topics_filter_de = topics.iter().fold(quote!(), |es, (field, _, field_name)| {
        quote! {
            #es #field: Some(ic_cdk::export::candid::decode_one(fields.get(#field_name).unwrap()).unwrap()),
        }
    });

    let values_ser = values.iter().fold(quote!(), |es, (field, field_name)| {
        quote! {
            #es ic_event_hub::types::EventField {
                name: String::from(#field_name),
                value: ic_cdk::export::candid::encode_one(&self.#field).unwrap()
            },
        }
    });

    let values_de = values.iter().fold(quote!(), |es, (field, field_name)| {
        quote! {
            #es #field: ic_cdk::export::candid::decode_one(fields.get(#field_name).unwrap()).unwrap(),
        }
    });

    // Create the new structure
    let gen = quote! {
        impl ic_event_hub::types::IEvent for #name {
             fn to_event(&self) -> ic_event_hub::types::Event {
                let mut res = std::collections::BTreeSet::new();
                res.insert(ic_event_hub::types::EventField {
                    name: String::from(ic_event_hub::EVENT_NAME_FIELD),
                    value: ic_cdk::export::candid::encode_one(#name_str).unwrap()
                });
                #topics_event_ser

                ic_event_hub::types::Event {
                    topics: res,
                    values: vec![#values_ser],
                }
            }

            fn from_event(event: ic_event_hub::types::Event) -> Self {
                let fields: std::collections::HashMap<String, Vec<u8>> = event
                    .topics
                    .into_iter()
                    .filter(|topic| topic.name != *ic_event_hub::EVENT_NAME_FIELD)
                    .chain(event.values.into_iter())
                    .map(|field| (field.name, field.value))
                    .collect();

                Self {
                    #topics_event_de
                    #values_de
                }
            }
        }

        #[derive(Debug)]
        pub struct #filter_name {
            #topics_filter
        }

        impl ic_event_hub::types::IEventFilter for #filter_name {
             fn to_event_filter(&self) -> ic_event_hub::types::EventFilter {
                let mut res = std::collections::BTreeSet::new();
                res.insert(ic_event_hub::types::EventField {
                    name: String::from(ic_event_hub::EVENT_NAME_FIELD),
                    value: ic_cdk::export::candid::encode_one(#name_str).unwrap()
                });
                #topics_filter_ser

                ic_event_hub::types::EventFilter(res)
            }

            fn from_event_filter(filter: ic_event_hub::types::EventFilter) -> Self {
                let fields: std::collections::HashMap<String, Vec<u8>> = filter
                    .0
                    .into_iter()
                    .filter(|topic| topic.name != *ic_event_hub::EVENT_NAME_FIELD)
                    .map(|field| (field.name, field.value))
                    .collect();

                Self {
                    #topics_filter_de
                }
            }
        }
    };

    gen.into()
}
