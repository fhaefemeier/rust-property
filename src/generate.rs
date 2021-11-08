// Copyright (C) 2019-2021 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use syn::{punctuated::Punctuated, token::Comma, GenericArgument};

pub(crate) enum GetType {
    Ref,
    Copy_,
    Clone_,
    String_,
    Slice(syn::TypeSlice),
    Option_(Punctuated<GenericArgument, Comma>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClrMethod {
    SetZero,
    SetNone,
    SetDefault,
    CallClear,
    FillWithDefault,
    None_,
}

pub(crate) enum FieldType {
    Number,
    Boolean,
    Character,
    String_,
    Array(syn::TypeArray),
    Vector(syn::Type),
    Option_(Punctuated<GenericArgument, Comma>),
    Unhandled(Option<String>),
}

impl GetType {
    pub(crate) fn from_field_type(ty: &FieldType) -> Self {
        match ty {
            FieldType::Number | FieldType::Boolean | FieldType::Character => GetType::Copy_,
            FieldType::String_ => GetType::String_,
            FieldType::Array(type_array) => {
                let syn::TypeArray {
                    bracket_token,
                    elem,
                    ..
                } = type_array.clone();
                GetType::Slice(syn::TypeSlice {
                    bracket_token,
                    elem,
                })
            }
            FieldType::Vector(inner_type) => GetType::Slice(syn::TypeSlice {
                bracket_token: syn::token::Bracket::default(),
                elem: Box::new(inner_type.clone()),
            }),
            FieldType::Option_(inner_type) => {
                if inner_type.len() == 1 {
                    if let Some(syn::GenericArgument::Type(inner_type)) = inner_type.first() {
                        if let GetType::Copy_ =
                            GetType::from_field_type(&FieldType::from_type(inner_type))
                        {
                            return GetType::Copy_;
                        }
                    }
                }
                GetType::Option_(inner_type.clone())
            }
            FieldType::Unhandled(_) => GetType::Ref,
        }
    }
}

impl ClrMethod {
    pub(crate) fn from_field_type(ty: &FieldType) -> Self {
        match ty {
            FieldType::Number => ClrMethod::SetZero,
            FieldType::Option_(_) => ClrMethod::SetNone,
            FieldType::Boolean | FieldType::Character => ClrMethod::SetDefault,
            FieldType::String_ | FieldType::Vector(_) => ClrMethod::CallClear,
            FieldType::Array(_) => ClrMethod::FillWithDefault,
            FieldType::Unhandled(Some(ref type_name)) => match type_name.as_str() {
                "String" | "PathBuf" | "Vec" | "VecDeque" | "LinkedList" | "HashMap"
                | "BTreeMap" | "HashSet" | "BTreeSet" | "BinaryHeap" => ClrMethod::CallClear,
                _ => ClrMethod::None_,
            },
            _ => ClrMethod::None_,
        }
    }
}

impl FieldType {
    pub(crate) fn from_type(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Path(type_path) => {
                let segs = &type_path.path.segments;
                if !segs.is_empty() {
                    match segs[0].ident.to_string().as_ref() {
                        "f32" | "f64" => FieldType::Number,
                        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => FieldType::Number,
                        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => FieldType::Number,
                        "bool" => FieldType::Boolean,
                        "char" => FieldType::Character,
                        "String" => FieldType::String_,
                        "Vec" => {
                            if let syn::PathArguments::AngleBracketed(inner) =
                                &type_path.path.segments[0].arguments
                            {
                                if let syn::GenericArgument::Type(ref inner_type) = inner.args[0] {
                                    FieldType::Vector(inner_type.clone())
                                } else {
                                    unreachable!()
                                }
                            } else {
                                unreachable!()
                            }
                        }
                        "Option" => {
                            if let syn::PathArguments::AngleBracketed(inner) =
                                &type_path.path.segments[0].arguments
                            {
                                FieldType::Option_(inner.args.clone())
                            } else {
                                unreachable!()
                            }
                        }
                        _ => {
                            let type_name = segs.last().cloned().unwrap().ident.to_string();
                            FieldType::Unhandled(Some(type_name))
                        }
                    }
                } else {
                    FieldType::Unhandled(None)
                }
            }
            syn::Type::Array(type_array) => FieldType::Array(type_array.clone()),
            _ => FieldType::Unhandled(None),
        }
    }
}
