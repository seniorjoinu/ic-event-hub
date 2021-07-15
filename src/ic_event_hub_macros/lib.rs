use proc_macro::TokenStream;
use std::collections::BTreeSet;

use quote::quote;
use syn::{Data, DeriveInput, Fields, parse};

#[proc_macro_derive(Event, attributes(topic))]
pub fn event_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse(input).unwrap();

    let name = &ast.ident;
    let name_str = name.to_string();

    let mut topics: Vec<(proc_macro2::Ident, String)> = vec![];
    let mut values: Vec<(proc_macro2::Ident, String)> = vec![];

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
                        topics.push((item.clone(), item.to_string()))
                    } else {
                        values.push((item.clone(), item.to_string()))
                    }
                }
            }
        }

        _ => panic!("Must be a struct"),
    }

    // Transform the marked elements into new struct fields
    let topics_ser = topics.iter().fold(quote!(), |es, (field, field_name)| {
        quote! {
            #es res.insert(ic_event_hub::EventField {
                name: String::from(#field_name),
                value: ic_cdk::export::candid::encode_one(&self.#field).unwrap()
            });
        }
    });

    let topics_de = topics.iter().fold(quote!(), |es, (field, field_name)| {
        quote! {
            #es #field: ic_cdk::export::candid::decode_one(fields.get(#field_name).unwrap()).unwrap(),
        }
    });

    let values_ser = values.iter().fold(quote!(), |es, (field, field_name)| {
        quote! {
            #es ic_event_hub::EventField {
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
        impl ic_event_hub::IEvent for #name {
             fn to_event(&self) -> ic_event_hub::Event {
                let mut res = std::collections::BTreeSet::new();
                res.insert(ic_event_hub::EventField {
                    name: String::from(ic_event_hub::EVENT_NAME_FIELD),
                    value: ic_cdk::export::candid::encode_one(#name_str).unwrap()
                });
                #topics_ser

                ic_event_hub::Event {
                    topics: res,
                    values: vec![#values_ser],
                }
            }

            fn from_event(event: ic_event_hub::Event) -> Self {
                let fields: std::collections::HashMap<String, Vec<u8>> = event
                    .topics
                    .into_iter()
                    .filter(|topic| topic.name != *ic_event_hub::EVENT_NAME_FIELD)
                    .chain(event.values.into_iter())
                    .map(|field| (field.name, field.value))
                    .collect();

                Self {
                    #topics_de
                    #values_de
                }
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(EventFilter, attributes(emitter_id, EventName))]
pub fn event_filter_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse(input).unwrap();

    let name = &ast.ident;

    let event_name_attr = ast
        .attrs
        .iter()
        .find(|item| item.path.is_ident("EventName"))
        .expect("Missing struct attribute #[EventName = \"<name of the event to filter by>\"]");

    let meta = event_name_attr.parse_meta().unwrap();

    let event_name = match meta {
        syn::Meta::NameValue(v) => match v.lit {
            syn::Lit::Str(l) => l.value(),
            _ => panic!("EventName should be of type String"),
        },
        _ => panic!("Missing struct attribute #[EventName = \"<name of the event to filter by>\"]"),
    };

    let mut topics: Vec<(proc_macro2::Ident, String)> = vec![];

    match ast.data {
        Data::Struct(ref data_struct) => {
            if let Fields::Named(ref fields_named) = data_struct.fields {
                for field in fields_named.named.iter() {
                    let item = field.ident.clone().unwrap();

                    topics.push((item.clone(), item.to_string()))
                }
            }
        }

        _ => panic!("Must be a struct"),
    }

    // Transform the marked elements into new struct fields
    let topics_ser = topics.iter().fold(quote!(), |es, (field, field_name)| {
        quote! {
            #es res.insert(ic_event_hub::EventField {
                name: String::from(#field_name),
                value: ic_cdk::export::candid::encode_one(&self.#field).unwrap()
            });
        }
    });

    let topics_de = topics.iter().fold(quote!(), |es, (field, field_name)| {
        quote! {
            #es #field: ic_cdk::export::candid::decode_one(fields.get(#field_name).unwrap()).unwrap(),
        }
    });

    let gen = quote! {
        impl ic_event_hub::IEventFilter for #name {
             fn to_event_filter(&self) -> ic_event_hub::EventFilter {
                let mut res = std::collections::BTreeSet::new();
                res.insert(ic_event_hub::EventField {
                    name: String::from(ic_event_hub::EVENT_NAME_FIELD),
                    value: ic_cdk::export::candid::encode_one(#event_name).unwrap()
                });
                #topics_ser

                ic_event_hub::EventFilter(res)
            }

            fn from_event_filter(filter: ic_event_hub::EventFilter) -> Self {
                let fields: std::collections::HashMap<String, Vec<u8>> = filter
                    .0
                    .into_iter()
                    .filter(|topic| topic.name != *ic_event_hub::EVENT_NAME_FIELD)
                    .map(|field| (field.name, field.value))
                    .collect();

                Self {
                    #topics_de
                }
            }
        }
    };

    gen.into()
}
