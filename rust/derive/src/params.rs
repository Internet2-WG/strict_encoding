// Derivation macro library for strict encoding.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 UBIDECO Institute
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

use amplify_syn::{
    ArgValueReq, AttrReq, DataType, ListReq, ParametrizedAttr, TypeClass, ValueClass,
};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{DeriveInput, Error, Expr, LitInt, LitStr, Path, Result};

const ATTR: &str = "strict_type";
const ATTR_CRATE: &str = "crate";
const ATTR_LIB: &str = "lib";
const ATTR_RENAME: &str = "rename";
const ATTR_WITH: &str = "with";
const ATTR_ENCODE_WITH: &str = "encode_with";
const ATTR_DECODE_WITH: &str = "decode_with";
const ATTR_DUMB: &str = "dumb";
const ATTR_TAGS: &str = "tags";
const ATTR_TAGS_ORDER: &str = "order";
const ATTR_TAGS_REPR: &str = "repr";
const ATTR_TAGS_CUSTOM: &str = "custom";
const ATTR_TAG: &str = "tag";

pub struct ContainerAttr {
    pub strict_crate: Path,
    pub lib: Expr,
    pub rename: Option<LitStr>,
    pub dumb: Option<Expr>,
    pub encode_with: Option<Path>,
    pub decode_with: Option<Path>,
}

pub struct EnumAttr {
    pub tags: VariantTags,
}

pub struct FieldAttr {
    pub dumb: Option<Expr>,
    pub rename: Option<LitStr>,
}

pub struct VariantAttr {
    pub dumb: bool,
    pub rename: Option<LitStr>,
    pub value: Option<LitInt>,
}

pub enum VariantTags {
    Repr,
    Order,
    Custom,
}

impl TryFrom<ParametrizedAttr> for ContainerAttr {
    type Error = Error;

    fn try_from(mut params: ParametrizedAttr) -> Result<Self> {
        let map = HashMap::from_iter(vec![
            (ATTR_CRATE, ArgValueReq::optional(TypeClass::Path)),
            (ATTR_LIB, ArgValueReq::required(ValueClass::Expr)),
            (ATTR_RENAME, ArgValueReq::optional(ValueClass::str())),
            (ATTR_DUMB, ArgValueReq::optional(ValueClass::Expr)),
            (ATTR_ENCODE_WITH, ArgValueReq::optional(TypeClass::Path)),
            (ATTR_DECODE_WITH, ArgValueReq::optional(TypeClass::Path)),
        ]);

        params.check(AttrReq::with(map))?;

        Ok(ContainerAttr {
            strict_crate: params.arg_value(ATTR_CRATE).unwrap_or_else(|_| path!(strict_encoding)),
            lib: params.unwrap_arg_value(ATTR_LIB),
            rename: params.arg_value(ATTR_RENAME).ok(),
            dumb: params.arg_value(ATTR_DUMB).ok(),
            encode_with: params
                .arg_value(ATTR_ENCODE_WITH)
                .or_else(|_| params.arg_value(ATTR_WITH))
                .ok(),
            decode_with: params
                .arg_value(ATTR_DECODE_WITH)
                .or_else(|_| params.arg_value(ATTR_WITH))
                .ok(),
        })
    }
}

impl TryFrom<ParametrizedAttr> for EnumAttr {
    type Error = Error;

    fn try_from(mut params: ParametrizedAttr) -> Result<Self> {
        let map = HashMap::from_iter(vec![(ATTR_TAGS, ArgValueReq::required(TypeClass::Path))]);
        params.check(AttrReq::with(map))?;

        let tags = match params
            .arg_value(ATTR_TAGS)
            .unwrap_or_else(|_| path!(custom))
            .to_token_stream()
            .to_string()
            .as_str()
        {
            ATTR_TAGS_REPR => VariantTags::Repr,
            ATTR_TAGS_ORDER => VariantTags::Order,
            ATTR_TAGS_CUSTOM => VariantTags::Custom,
            unknown => {
                return Err(Error::new(
                    Span::call_site(),
                    format!(
                        "invalid enum strict encoding value for `tags` attribute `{unknown}`; \
                         only `repr`, `order` or `custom` are allowed"
                    ),
                ))
            }
        };

        Ok(EnumAttr { tags })
    }
}

impl TryFrom<ParametrizedAttr> for FieldAttr {
    type Error = Error;

    fn try_from(mut params: ParametrizedAttr) -> Result<Self> {
        let map = HashMap::from_iter(vec![
            (ATTR_RENAME, ArgValueReq::optional(ValueClass::str())),
            (ATTR_DUMB, ArgValueReq::optional(ValueClass::Expr)),
        ]);

        params.check(AttrReq::with(map))?;

        Ok(FieldAttr {
            rename: params.arg_value(ATTR_RENAME).ok(),
            dumb: params.arg_value(ATTR_DUMB).ok(),
        })
    }
}

impl TryFrom<ParametrizedAttr> for VariantAttr {
    type Error = Error;

    fn try_from(mut params: ParametrizedAttr) -> Result<Self> {
        let map = HashMap::from_iter(vec![
            (ATTR_RENAME, ArgValueReq::optional(ValueClass::str())),
            (ATTR_TAG, ArgValueReq::optional(ValueClass::int())),
        ]);

        let mut req = AttrReq::with(map);
        req.path_req = ListReq::maybe_one(path!(dumb));
        params.check(req)?;

        Ok(VariantAttr {
            rename: params.arg_value(ATTR_RENAME).ok(),
            value: params.arg_value(ATTR_TAG).ok(),
            dumb: params.has_verbatim("dumb"),
        })
    }
}

pub struct StrictDerive {
    pub data: DataType,
    pub conf: ContainerAttr,
}

impl TryFrom<DeriveInput> for StrictDerive {
    type Error = Error;

    fn try_from(input: DeriveInput) -> Result<Self> {
        let params = ParametrizedAttr::with(ATTR, &input.attrs)?;
        let conf = ContainerAttr::try_from(params)?;
        let data = DataType::with(input, ident!(strict_type))?;
        Ok(Self { data, conf })
    }
}