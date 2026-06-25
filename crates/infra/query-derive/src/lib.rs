//! `#[derive(DomainQuery)]` proc macro.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Data, DeriveInput, Field, Fields, Ident, LitStr, Token,
};

// ---------- Attribute grammar ----------

#[derive(Debug, Default)]
struct FieldAttrs {
    filterable: bool,
    sortable: bool,
    ignore: bool,
    relation: Option<String>,
    builder: Option<String>,
}

impl Parse for FieldAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = Self::default();
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "filterable" => attrs.filterable = true,
                "sortable" => attrs.sortable = true,
                "ignore" => attrs.ignore = true,
                "relation" => {
                    let _eq: Token![=] = input.parse()?;
                    let lit: LitStr = input.parse()?;
                    attrs.relation = Some(lit.value());
                }
                "builder" => {
                    let _eq: Token![=] = input.parse()?;
                    let lit: LitStr = input.parse()?;
                    attrs.builder = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        format!("unknown #[query({other})] attribute; expected one of: filterable, sortable, ignore, relation = \"...\", builder = \"...\""),
                    ));
                }
            }
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            } else {
                break;
            }
        }
        Ok(attrs)
    }
}

fn parse_field_attrs(field: &Field) -> syn::Result<FieldAttrs> {
    let mut merged = FieldAttrs::default();
    for attr in &field.attrs {
        if attr.path().is_ident("query") {
            let list = attr.meta.require_list()?;
            let parsed: FieldAttrs = syn::parse2(list.tokens.clone())?;
            if parsed.relation.is_some() && merged.relation.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate `relation = \"...\"` attribute on field",
                ));
            }
            if parsed.builder.is_some() && merged.builder.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate `builder = \"...\"` attribute on field",
                ));
            }
            merged.filterable |= parsed.filterable;
            merged.sortable |= parsed.sortable;
            merged.ignore |= parsed.ignore;
            merged.relation = merged.relation.or(parsed.relation);
            merged.builder = merged.builder.or(parsed.builder);
        }
    }
    Ok(merged)
}

// ---------- Helpers ----------

fn pascal_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_word_start = true;
    for ch in s.chars() {
        if ch == '_' {
            at_word_start = true;
        } else if at_word_start {
            out.extend(ch.to_uppercase());
            at_word_start = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Convert a PascalCase or camelCase identifier to snake_case.
/// Used for table-name derivation in the EntityDescriptor
/// emission.
fn snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                let prev = s.chars().nth(i - 1);
                let next = s.chars().nth(i + 1);
                let prev_is_lower = prev.is_some_and(|c| c.is_ascii_lowercase());
                let next_is_lower = next.is_some_and(|c| c.is_ascii_lowercase());
                if prev_is_lower || next_is_lower {
                    out.push('_');
                }
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

// ---------- Macro expansion ---------

/// Procedural derive: emits the `*Field` enum, `*Relation` enum,
/// `*QueryBuilder` struct, and the `Field` / `HasRelations` impls
/// for a domain struct.
///
/// Usage:
///
/// ```rust,ignore
/// use educore_query_derive::DomainQuery;
/// use educore_core::prelude::*;
///
/// #[derive(DomainQuery)]
/// pub struct Student {
///     pub id: Uuid,
///
///     #[query(sortable)]
///     pub last_name: String,
///
///     #[query(filterable)]
///     pub status: StudentStatus,
///
///     #[query(filterable, relation = "Parent", builder = "ParentQueryBuilder")]
///     pub parent_id: Uuid,
/// }
/// ```
///
/// See `docs/query_layer.md` for the full spec.
#[proc_macro_derive(DomainQuery, attributes(query))]
pub fn derive_domain_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

struct FieldInfo {
    name: Ident,
    field: Field,
    attrs: FieldAttrs,
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    #[allow(
        clippy::too_many_lines,
        clippy::similar_names,
        clippy::needless_pass_by_value,
        clippy::needless_borrow
    )]
    fn expand_inner(input: DeriveInput) -> syn::Result<TokenStream2> {
        let struct_name = input.ident.clone();
        let struct_vis = input.vis.clone();

        let fields = match &input.data {
            Data::Struct(ds) => match &ds.fields {
                Fields::Named(named) => &named.named,
                _ => {
                    return Err(syn::Error::new_spanned(
                        &input,
                        "DomainQuery can only be derived for structs with named fields",
                    ));
                }
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "DomainQuery can only be derived for structs",
                ));
            }
        };

        if fields.is_empty() {
            return Err(syn::Error::new_spanned(
                &input,
                "DomainQuery cannot be derived for a struct with no fields",
            ));
        }

        let mut field_infos: Vec<FieldInfo> = Vec::with_capacity(fields.len());
        for field in fields {
            let attrs = parse_field_attrs(field)?;
            let name = field
                .ident
                .clone()
                .ok_or_else(|| syn::Error::new_spanned(field, "expected a named field"))?;
            field_infos.push(FieldInfo {
                name,
                field: field.clone(),
                attrs,
            });
        }

        let field_enum_name = format_ident!("{}Field", struct_name);
        let relation_enum_name = format_ident!("{}Relation", struct_name);
        let builder_name = format_ident!("{}QueryBuilder", struct_name);

        let queryable: Vec<&FieldInfo> = field_infos
            .iter()
            .filter(|f| {
                // Per the spec: "Fields are excluded from query
                // generation by default. A field is queryable only
                // when decorated." So a field needs `filterable` or
                // `sortable` to be included; a field with no
                // `#[query(...)]` attribute (e.g. `id`) is excluded
                // even if not `ignore`.
                (f.attrs.filterable || f.attrs.sortable)
                    && !f.attrs.ignore
                    && f.attrs.relation.is_none()
            })
            .collect();
        // `relations` carries the relation name and the builder type
        // alongside the field info. By filtering with `filter_map` and
        // binding the `relation` and `builder` values up-front, the
        // downstream code generation never needs to unwrap `Option`
        // values (the engine forbids `unwrap` and `expect` in
        // production paths).
        let relations: Vec<(&FieldInfo, &str, &str)> = field_infos
            .iter()
            .filter_map(|f| {
                let relation = f.attrs.relation.as_deref()?;
                let builder = f.attrs.builder.as_deref()?;
                Some((f, relation, builder))
            })
            .collect();

        if queryable.is_empty() && relations.is_empty() {
            return Err(syn::Error::new_spanned(
            &input,
            "DomainQuery requires at least one `#[query(...)]` decorated field (filterable, sortable, or relation)",
        ));
        }

        // Sanity-check every `relation` has a `builder`. This is
        // implied by `filter_map` above but a fresh compile error is
        // friendlier than letting the macro emit a type that names an
        // undeclared builder.
        for f in field_infos.iter().filter(|f| f.attrs.relation.is_some()) {
            if f.attrs.builder.is_none() {
                return Err(syn::Error::new_spanned(
                    &f.field,
                    format!(
                        "field `{}` has `relation = \"...\"` but no `builder = \"...\"`",
                        f.name
                    ),
                ));
            }
        }

        let mut seen_queryable: Vec<&Ident> = Vec::new();
        for f in &queryable {
            if seen_queryable.iter().any(|i| **i == f.name) {
                return Err(syn::Error::new_spanned(
                    &f.field,
                    format!("duplicate queryable field `{}`", f.name),
                ));
            }
            seen_queryable.push(&f.name);
        }

        let field_variants = queryable.iter().map(|f| {
            let variant = format_ident!("{}", pascal_case(&f.name.to_string()));
            quote! { #variant }
        });
        let field_match_arms = queryable.iter().map(|f| {
            let variant = format_ident!("{}", pascal_case(&f.name.to_string()));
            let column = f.name.to_string();
            quote! { Self::#variant => #column }
        });

        let _field_to_variant_arms = queryable.iter().map(|f| {
            let variant = format_ident!("{}", pascal_case(&f.name.to_string()));
            let ident = &f.name;
            quote! { Self::#ident => #field_enum_name::#variant }
        });

        let field_enum = if queryable.is_empty() {
            quote! {
                #[automatically_derived]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                #struct_vis enum #field_enum_name {}
            }
        } else {
            quote! {
                #[automatically_derived]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                #struct_vis enum #field_enum_name {
                    #(#field_variants),*
                }
            }
        };

        let field_impl = if queryable.is_empty() {
            // Emit a vacuous `Field` impl even when there are no
            // queryable fields. The builder's `filters` and `orders`
            // fields are typed `Vec<QueryNode<FieldEnum>>` and
            // `Vec<OrderNode<FieldEnum>>`, which require
            // `FieldEnum: Field`. An empty enum has no variants to
            // match on, but the impl is still required for the
            // builder to compile.
            quote! {
                #[automatically_derived]
                impl ::educore_core::query::Field for #field_enum_name {
                    fn column_name(self) -> &'static str {
                        match self {}
                    }
                }
            }
        } else {
            quote! {
                #[automatically_derived]
                impl ::educore_core::query::Field for #field_enum_name {
                    fn column_name(self) -> &'static str {
                        match self {
                            #(#field_match_arms),*
                        }
                    }
                }
            }
        };

        let (relation_enum, relation_impls) = if relations.is_empty() {
            (quote! {}, quote! {})
        } else {
            let relation_variants: Vec<_> = relations
                .iter()
                .map(|(_info, relation, _builder)| {
                    let variant = format_ident!("{relation}");
                    quote! { #variant }
                })
                .collect();
            let relation_match_arms: syn::Result<Vec<_>> = relations
            .iter()
            .enumerate()
            .map(|(i, (_info, relation, _builder))| {
                let variant = format_ident!("{relation}");
                let name = relation.to_lowercase();
                // The number of relations in a struct is bounded by
                // Rust's struct field count limit, well below
                // `u32::MAX`. The try_from + `?` is the
                // deny-`unwrap_used` / deny-`expect_used` /
                // deny-`cast_possible_truncation` form of "this
                // conversion can never actually fail."
                let id = u32::try_from(i).map_err(|_| {
                    syn::Error::new_spanned(
                        &relation_enum_name,
                        format!("too many relations: {i} exceeds u32::MAX"),
                    )
                })?;
                // Use `#relation_enum_name` (the relation enum)
                // rather than `Self` (which inside the
                // `From<XRelation> for Relation` impl resolves to
                // `Relation`, the OUTPUT type, not the input).
                let arm = quote! { #relation_enum_name::#variant => ::educore_core::query::Relation { id: #id, name: #name } };
                Ok(arm)
            })
            .collect();
            let relation_match_arms = relation_match_arms?;
            let all_relations_slice: Vec<_> = relations
                .iter()
                .map(|(_info, relation, _builder)| {
                    let variant = format_ident!("{relation}");
                    quote! { #relation_enum_name::#variant }
                })
                .collect();

            let relation_enum = quote! {
                #[automatically_derived]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
                #struct_vis enum #relation_enum_name {
                    #(#relation_variants),*
                }
            };

            let relation_impls = quote! {
                #[automatically_derived]
                impl #field_enum_name {
                    #struct_vis fn all_relations() -> &'static [#relation_enum_name] {
                        &[ #(#all_relations_slice),* ]
                    }
                }

                #[automatically_derived]
                impl ::educore_core::query::HasRelations for #field_enum_name {
                    type Relation = #relation_enum_name;
                }

                #[automatically_derived]
                impl From<#relation_enum_name> for ::educore_core::query::Relation {
                    fn from(r: #relation_enum_name) -> Self {
                        match r {
                            #(#relation_match_arms),*
                        }
                    }
                }
            };

            (relation_enum, relation_impls)
        };

        let builder_struct = if relations.is_empty() {
            quote! {
                #[automatically_derived]
                #[derive(Debug, Default)]
                #struct_vis struct #builder_name {
                    school_id: ::std::option::Option<::educore_core::ids::SchoolId>,
                    filters: ::std::vec::Vec<::educore_core::query::QueryNode<#field_enum_name>>,
                    orders: ::std::vec::Vec<::educore_core::query::OrderNode<#field_enum_name>>,
                    offset: u32,
                    limit: ::std::option::Option<u32>,
                }
            }
        } else {
            quote! {
                #[automatically_derived]
                #[derive(Debug, Default)]
                #struct_vis struct #builder_name {
                    school_id: ::std::option::Option<::educore_core::ids::SchoolId>,
                    filters: ::std::vec::Vec<::educore_core::query::QueryNode<#field_enum_name>>,
                    orders: ::std::vec::Vec<::educore_core::query::OrderNode<#field_enum_name>>,
                    offset: u32,
                    limit: ::std::option::Option<u32>,
                    relations: ::std::collections::BTreeSet<#relation_enum_name>,
                }
            }
        };

        let where_methods = if queryable.is_empty() {
            quote! {}
        } else {
            let where_eq = quote! {
                #struct_vis fn where_eq<V>(mut self, field: #field_enum_name, value: V) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    self.filters.push(::educore_core::query::QueryNode::Eq(field, value.into()));
                    self
                }
            };
            let where_ne = quote! {
                #struct_vis fn where_ne<V>(mut self, field: #field_enum_name, value: V) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    self.filters.push(::educore_core::query::QueryNode::Ne(field, value.into()));
                    self
                }
            };
            let where_lt = quote! {
                #struct_vis fn where_lt<V>(mut self, field: #field_enum_name, value: V) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    self.filters.push(::educore_core::query::QueryNode::Lt(field, value.into()));
                    self
                }
            };
            let where_lte = quote! {
                #struct_vis fn where_lte<V>(mut self, field: #field_enum_name, value: V) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    self.filters.push(::educore_core::query::QueryNode::Lte(field, value.into()));
                    self
                }
            };
            let where_gt = quote! {
                #struct_vis fn where_gt<V>(mut self, field: #field_enum_name, value: V) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    self.filters.push(::educore_core::query::QueryNode::Gt(field, value.into()));
                    self
                }
            };
            let where_gte = quote! {
                #struct_vis fn where_gte<V>(mut self, field: #field_enum_name, value: V) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    self.filters.push(::educore_core::query::QueryNode::Gte(field, value.into()));
                    self
                }
            };
            let where_in = quote! {
                #struct_vis fn where_in<V>(mut self, field: #field_enum_name, values: ::std::vec::Vec<V>) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    let v: ::std::vec::Vec<::educore_core::query::Value> =
                        values.into_iter().map(::std::convert::Into::into).collect();
                    self.filters.push(::educore_core::query::QueryNode::In(field, v));
                    self
                }
            };
            let where_not_in = quote! {
                #struct_vis fn where_not_in<V>(mut self, field: #field_enum_name, values: ::std::vec::Vec<V>) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    let v: ::std::vec::Vec<::educore_core::query::Value> =
                        values.into_iter().map(::std::convert::Into::into).collect();
                    self.filters.push(::educore_core::query::QueryNode::NotIn(field, v));
                    self
                }
            };
            let where_between = quote! {
                #struct_vis fn where_between<V>(mut self, field: #field_enum_name, lo: V, hi: V) -> Self
                where V: ::std::convert::Into<::educore_core::query::Value>
                {
                    self.filters.push(::educore_core::query::QueryNode::Between(field, lo.into(), hi.into()));
                    self
                }
            };
            let where_null = quote! {
                #struct_vis fn where_null(mut self, field: #field_enum_name) -> Self {
                    self.filters.push(::educore_core::query::QueryNode::IsNull(field));
                    self
                }
            };
            let where_not_null = quote! {
                #struct_vis fn where_not_null(mut self, field: #field_enum_name) -> Self {
                    self.filters.push(::educore_core::query::QueryNode::IsNotNull(field));
                    self
                }
            };
            let where_like = quote! {
                #struct_vis fn where_like(mut self, field: #field_enum_name, pattern: ::educore_core::query::Pattern) -> Self {
                    self.filters.push(::educore_core::query::QueryNode::Like(field, pattern));
                    self
                }
            };
            let where_ilike = quote! {
                #struct_vis fn where_ilike(mut self, field: #field_enum_name, pattern: ::educore_core::query::Pattern) -> Self {
                    self.filters.push(::educore_core::query::QueryNode::ILike(field, pattern));
                    self
                }
            };

            quote! {
                #where_eq
                #where_ne
                #where_lt
                #where_lte
                #where_gt
                #where_gte
                #where_in
                #where_not_in
                #where_between
                #where_null
                #where_not_null
                #where_like
                #where_ilike
            }
        };

        let order_methods = if queryable.is_empty() {
            quote! {}
        } else {
            quote! {
                #struct_vis fn order_by(mut self, field: #field_enum_name) -> Self {
                    self.orders.push(::educore_core::query::OrderNode::asc(field));
                    self
                }

                #struct_vis fn order_by_desc(mut self, field: #field_enum_name) -> Self {
                    self.orders.push(::educore_core::query::OrderNode::desc(field));
                    self
                }
            }
        };

        let pagination_methods = quote! {
            #struct_vis fn limit(mut self, n: u32) -> Self {
                self.limit = ::std::option::Option::Some(n);
                self
            }

            #struct_vis fn offset(mut self, n: u32) -> Self {
                self.offset = n;
                self
            }

            #struct_vis fn page(mut self, offset: u32, limit: u32) -> Self {
                self.offset = offset;
                self.limit = ::std::option::Option::Some(limit);
                self
            }
        };

        let tenant_methods = quote! {
            #struct_vis fn for_school(mut self, school_id: ::educore_core::ids::SchoolId) -> Self {
                self.school_id = ::std::option::Option::Some(school_id);
                self
            }

            #struct_vis fn school_id(&self) -> ::std::option::Option<::educore_core::ids::SchoolId> {
                self.school_id
            }
        };

        let relation_methods = if relations.is_empty() {
            quote! {}
        } else {
            let mut where_has_methods = Vec::new();
            for (_info, relation, builder) in &relations {
                let relation_variant = format_ident!("{relation}");
                let builder_ty: Ident = syn::parse_str(builder)?;
                let method = format_ident!("where_has_{}", pascal_case(relation));
                where_has_methods.push(quote! {
                #struct_vis fn #method<__F>(mut self, __build: __F) -> Self
                where
                    __F: ::std::ops::FnOnce(#builder_ty) -> #builder_ty,
                {
                    let related: #builder_ty = __build(#builder_ty::new());
                    // `related.__educore_compile()` returns
                    // `QueryNode<<RelatedField>>` (the related
                    // builder's field type). We do not annotate
                    // the type here so the compiler infers it
                    // from the return type of `__educore_compile`.
                    let inner = related.__educore_compile();
                    let inner_rel: ::educore_core::query::QueryNode<::educore_core::query::RelationalField> =
                        ::educore_core::query::to_relational_node(inner);
                    let rel: ::educore_core::query::Relation =
                        #relation_enum_name::#relation_variant.into();
                    self.filters.push(
                        ::educore_core::query::QueryNode::HasRelation(
                            rel,
                            ::std::boxed::Box::new(inner_rel),
                        )
                    );
                    self
                }
            });
            }

            let generic_where_has = quote! {
                #struct_vis fn where_has<__R, __F>(mut self, relation: __R, __build: __F) -> Self
                where
                    __R: ::std::convert::Into<::educore_core::query::Relation>,
                    __F: ::std::ops::FnOnce(::educore_core::query::RelationalField) -> ::educore_core::query::RelationalField,
                {
                    let _rel_field: ::educore_core::query::RelationalField =
                        __build(::educore_core::query::RelationalField);
                    let rel: ::educore_core::query::Relation = relation.into();
                    self.filters.push(
                        ::educore_core::query::QueryNode::HasRelation(
                            rel,
                            ::std::boxed::Box::new(
                                ::educore_core::query::QueryNode::Eq(
                                    _rel_field,
                                    ::educore_core::query::Value::Bool(true),
                                ),
                            ),
                        ),
                    );
                    self
                }
            };

            let with = quote! {
                #struct_vis fn with(mut self, relation: #relation_enum_name) -> Self {
                    self.relations.insert(relation);
                    self
                }

                #struct_vis fn with_many(mut self, relations: &[#relation_enum_name]) -> Self {
                    for r in relations {
                        self.relations.insert(*r);
                    }
                    self
                }

                #struct_vis fn relations(&self) -> impl ::std::iter::Iterator<Item = #relation_enum_name> + '_ {
                    self.relations.iter().copied()
                }
            };

            quote! {
                #(#where_has_methods)*
                #generic_where_has
                #with
            }
        };

        let compile_method = if queryable.is_empty() {
            // No queryable fields on the struct. The user is expected
            // to always add a where_has_<relation> filter before
            // build_query_node. The degenerate tree below is
            // `HasRelation(_, IsNull(RelationalField)) AND
            // HasRelation(_, IsNotNull(RelationalField))` — a
            // never-satisfiable predicate that the storage adapter
            // is expected to recognise as "no rows" if the user
            // actually queries without any filter. The `HasRelation`
            // variant is generic over the outer field type, so we
            // can construct a `QueryNode<BookmarkField>` even when
            // `BookmarkField` is an empty enum (there is no
            // `BookmarkField` value to put in a leaf variant like
            // `IsNull`). The `Relation { id: 0, name: "" }` is a
            // sentinel that the storage adapter should never see in
            // practice (the user is expected to add a real filter).
            quote! {
                #[doc(hidden)]
                #struct_vis fn __educore_compile(self) -> ::educore_core::query::QueryNode<#field_enum_name> {
                    ::educore_core::query::QueryNode::And(
                        ::std::boxed::Box::new(
                            ::educore_core::query::QueryNode::HasRelation(
                                ::educore_core::query::Relation { id: 0, name: "" },
                                ::std::boxed::Box::new(
                                    ::educore_core::query::QueryNode::IsNull(
                                        ::educore_core::query::RelationalField
                                    )
                                ),
                            )
                        ),
                        ::std::boxed::Box::new(
                            ::educore_core::query::QueryNode::HasRelation(
                                ::educore_core::query::Relation { id: 0, name: "" },
                                ::std::boxed::Box::new(
                                    ::educore_core::query::QueryNode::IsNotNull(
                                        ::educore_core::query::RelationalField
                                    )
                                ),
                            )
                        ),
                    )
                }
            }
        } else {
            let all_variants_list = queryable.iter().map(|f| {
                let variant = format_ident!("{}", pascal_case(&f.name.to_string()));
                quote! { Self::#variant }
            });
            // NOTE: the `all_variants` impl is NOT emitted here —
            // it is emitted separately at the module level via
            // `all_variants_helper_out` (below). Emitting it inside
            // the `builder_impl` would nest an impl inside an impl,
            // which the compiler rejects with
            // "implementation is not supported in `trait`s or `impl`s".
            let _ = all_variants_list;
            quote! {
                #[doc(hidden)]
                #struct_vis fn __educore_compile(self) -> ::educore_core::query::QueryNode<#field_enum_name> {
                    let mut iter = self.filters.into_iter();
                    let first = iter.next();
                    match first {
                        ::std::option::Option::None => {
                            let first_variant = #field_enum_name::all_variants()[0];
                            ::educore_core::query::QueryNode::And(
                                ::std::boxed::Box::new(::educore_core::query::QueryNode::IsNull(first_variant)),
                                ::std::boxed::Box::new(::educore_core::query::QueryNode::IsNotNull(first_variant)),
                            )
                        }
                        ::std::option::Option::Some(first) => {
                            iter.fold(first, |acc, next| {
                                ::educore_core::query::QueryNode::And(
                                    ::std::boxed::Box::new(acc),
                                    ::std::boxed::Box::new(next),
                                )
                            })
                        }
                    }
                }
            }
        };

        // Hoist `all_variants` to the outer emission so it is
        // available to the final `let out = ...` block below. When
        // `queryable` is empty, `all_variants_helper` is empty too.
        let all_variants_helper_out: TokenStream2 = if queryable.is_empty() {
            quote! {}
        } else {
            let all_variants_list = queryable.iter().map(|f| {
                let variant = format_ident!("{}", pascal_case(&f.name.to_string()));
                quote! { Self::#variant }
            });
            quote! {
                #[automatically_derived]
                impl #field_enum_name {
                    #[doc(hidden)]
                    #struct_vis fn all_variants() -> &'static [#field_enum_name] {
                        &[ #(#all_variants_list),* ]
                    }
                }
            }
        };

        let build_query_node = quote! {
            #struct_vis fn build_query_node(self)
                -> ::educore_core::error::Result<(::educore_core::query::QueryNode<#field_enum_name>, ::educore_core::query::Page)>
            {
                if self.school_id.is_none() {
                    return ::std::result::Result::Err(
                        ::educore_core::error::DomainError::validation(
                            concat!(
                                stringify!(#builder_name),
                                " requires for_school() before build_query_node()"
                            )
                        )
                    );
                }
                // Extract `limit` and `offset` BEFORE compiling the
                // node, because `__educore_compile` takes `self` by
                // value and a subsequent `self.limit` / `self.offset`
                // would be a use-after-move.
                let limit = self.limit.unwrap_or(50);
                let offset = self.offset;
                let node = self.__educore_compile();
                let page = ::educore_core::query::Page::new(offset, limit);
                ::std::result::Result::Ok((node, page))
            }

            #struct_vis fn orders(&self) -> ::std::vec::Vec<::educore_core::query::OrderNode<#field_enum_name>> {
                self.orders.clone()
            }
        };

        let new_method = quote! {
            #struct_vis fn new() -> Self {
                Self::default()
            }
        };

        let builder_impl = quote! {
            #[automatically_derived]
            impl #builder_name {
                #new_method
                #tenant_methods
                #where_methods
                #order_methods
                #pagination_methods
                #relation_methods
                #compile_method
                #build_query_node
            }
        };

        let struct_query = quote! {
            #[automatically_derived]
            impl #struct_name {
                #struct_vis fn query() -> #builder_name {
                    #builder_name::new()
                }
            }
        };

        // -------- EntityDescriptor emission --------
        //
        // Emit a `pub fn entity_descriptor() -> EntityDescriptor` on
        // the derived struct. The table name is the struct name in
        // snake_case (with a trailing "s" — naive pluralization;
        // adapters may override via a future `#[query(table = "...")]`
        // attribute). The columns are derived from the struct's
        // fields, with `ColumnType::Custom(<TypeName>)` so the wire
        // descriptor carries the Rust type name verbatim. A future
        // Cluster A stage 2 follow-up will map Rust types to
        // `ColumnType::Uuid` / `ColumnType::String` etc. instead of
        // `Custom`.
        //
        // Indexes emit a primary-key index (`idx_pk` on `id`); RLS
        // emits a `tenant_isolation` policy. Foreign keys remain
        // empty pending the `#[foreign_key]` attribute wiring (tracked
        // under Cluster A stage 2).
        //
        // Note: this is a *function*, not a `const`, because
        // `EntityDescriptor` contains `Vec` fields and Rust forbids
        // heap allocation in `const` contexts. The function returns
        // a fresh descriptor on each call; storage adapters
        // cache the result internally as needed.
        let table_name = snake_case(&struct_name.to_string()) + "s";
        let table_name_lit: LitStr = syn::parse_quote!(#table_name);

        let column_entries = field_infos.iter().map(|f| {
            let col_name = f.name.to_string();
            let col_name_lit: LitStr = syn::parse_quote!(#col_name);
            let ty = &f.field.ty;
            quote! {
                ::educore_core::query::ColumnDescriptor {
                    name: #col_name_lit,
                    column_type: ::educore_core::query::ColumnType::Custom(::std::stringify!(#ty)),
                    nullable: true,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                }
            }
        });

        let entity_descriptor_const = quote! {
            #[automatically_derived]
            impl #struct_name {
                /// The dialect-agnostic schema descriptor for this
                /// aggregate. Storage adapters walk this at
                /// `create_schema()` time to emit DDL.
                pub fn entity_descriptor() -> ::educore_core::query::EntityDescriptor {
                    ::educore_core::query::EntityDescriptor {
                        table: #table_name_lit,
                        columns: ::std::vec![
                            #(#column_entries),*
                        ],
                        indexes: ::std::vec![
                            ::educore_core::query::IndexDescriptor {
                                name: "idx_pk",
                                columns: ::std::vec!["id"],
                                unique: true,
                            },
                        ],
                        foreign_keys: ::std::vec![],
                        rls: ::std::vec![
                            ::educore_core::query::RlsPolicy {
                                name: "tenant_isolation",
                                using_expr: "school_id = current_setting('app.school_id')::uuid",
                                with_check_expr: ::std::option::Option::None,
                            },
                        ],
                    }
                }
            }
        };

        let out = quote! {
            #field_enum
            #field_impl
            #all_variants_helper_out
            #relation_enum
            #relation_impls
            #builder_struct
            #builder_impl
            #struct_query
            #entity_descriptor_const
        };

        Ok(out)
    }

    expand_inner(input)
}

// ============================================================================
// Test module — verifies the macro emits correct code
// ============================================================================
