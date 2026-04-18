use crate::util::error;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::cmp::Ordering;
use syn::ext::IdentExt;
use syn::parse::{ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    parse_quote, GenericParam, Generics, ImplGenerics, LitBool, LitStr, PathArguments, Result,
    Token, WhereClause, WherePredicate,
};

pub(super) trait DelegateTarget: Default {
    fn try_update(&mut self, key: &str, lit: LitStr) -> Option<Result<()>>;
}

#[derive(Clone, Copy, Default)]
pub(crate) enum InlineMode {
    #[default]
    Yes,
    No,
    Always,
    Never,
}

impl InlineMode {
    fn from_lit(lit: &LitStr) -> Result<Self> {
        match lit.value().as_str() {
            "yes" => Ok(InlineMode::Yes),
            "no" => Ok(InlineMode::No),
            "always" => Ok(InlineMode::Always),
            "never" => Ok(InlineMode::Never),
            _ => error!(
                lit.span(),
                "invalid value for \"inline\": expected \"yes\", \"no\", \"always\", or \"never\""
            ),
        }
    }

    pub(crate) fn as_bracket_tokens(self) -> TokenStream2 {
        match self {
            InlineMode::Yes => quote!([inline]),
            InlineMode::No => quote!([]),
            InlineMode::Always => quote!([inline(always)]),
            InlineMode::Never => quote!([inline(never)]),
        }
    }
}

pub(crate) fn parse_inline_mode(attr: TokenStream2) -> Result<InlineMode> {
    if attr.is_empty() {
        return Ok(InlineMode::Yes);
    }
    (|input: ParseStream| -> Result<InlineMode> {
        let key = input.call(Ident::parse_any)?;
        if key != "inline" {
            return error!(key.span(), "invalid key for a delegatable_trait attribute");
        }
        let _: Token![=] = input.parse()?;
        let val: LitStr = input.parse()?;
        InlineMode::from_lit(&val)
    })
    .parse2(attr)
}

#[derive(Default)]
pub(super) struct DelegateArgs<T: DelegateTarget> {
    pub(crate) target: T,
    pub(crate) where_clauses: Punctuated<WherePredicate, Comma>,
    pub(crate) generics: Vec<GenericParam>,
    pub(crate) inhibit_automatic_where_clause: bool,
    pub(crate) inline: Option<InlineMode>,
}

impl<T: DelegateTarget> DelegateArgs<T> {
    fn add_key_value(&mut self, key: Ident, lit: LitStr) -> Result<()> {
        let span = key.span();
        match &*key.to_string() {
            "where" => {
                let where_clause_val =
                    lit.parse_with(Punctuated::<WherePredicate, Comma>::parse_terminated)?;
                self.where_clauses.extend(where_clause_val);
            }
            "generics" => {
                let generics_val =
                    lit.parse_with(Punctuated::<GenericParam, Comma>::parse_terminated)?;
                self.generics.extend(generics_val);
            }
            "automatic_where_clause" => {
                let auto_where_val: LitBool = lit.parse()?;
                self.inhibit_automatic_where_clause = !auto_where_val.value;
            }
            "inline" => {
                self.inline = Some(InlineMode::from_lit(&lit)?);
            }
            key => self
                .target
                .try_update(key, lit)
                .unwrap_or_else(|| error!(span, "invalid key for a delegate attribute"))?,
        }
        Ok(())
    }
}

pub(super) fn delegate_attr_as_trait_and_iter<T: DelegateTarget>(
    outer_steam: ParseStream<'_>,
) -> Result<(syn::Path, DelegateArgs<T>)> {
    let items;
    syn::parenthesized!(items in outer_steam);
    let path = items.parse()?;
    let mut delegate_args = DelegateArgs::default();
    while !items.is_empty() {
        let _: Token![,] = items.parse()?;
        let key = items.call(Ident::parse_any)?;
        let _: Token![=] = items.parse()?;
        let val = items.parse()?;
        delegate_args.add_key_value(key, val)?;
    }
    Ok((path, delegate_args))
}

impl<T: DelegateTarget> DelegateArgs<T> {
    pub fn from_tokens(tokens: TokenStream2) -> Result<(syn::Path, Self)> {
        let (path, mut res) = delegate_attr_as_trait_and_iter.parse2(tokens)?;
        res.generics.sort_unstable_by(|x, y| match (x, y) {
            (GenericParam::Lifetime(_), GenericParam::Lifetime(_)) => Ordering::Equal,
            (GenericParam::Lifetime(_), _) => Ordering::Less,
            (_, GenericParam::Lifetime(_)) => Ordering::Greater,
            _ => Ordering::Equal,
        });
        Ok((path, res))
    }
}

pub(super) fn delegate_macro<I>(
    input: &I,
    attrs: Vec<syn::Attribute>,
    delegate_single: impl Fn(&I, TokenStream2) -> Result<TokenStream2>,
) -> TokenStream2 {
    // Parse the input tokens into a syntax tree
    let mut delegate_attributes = attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident("delegate"))
        .map(|attr| attr.meta.to_token_stream().into_iter().skip(1).collect())
        .peekable();
    if delegate_attributes.peek().is_none() {
        return error!(
            proc_macro2::Span::call_site(),
            "No #[delegate] attribute specified. If you want to delegate an implementation of trait `SomeTrait` add the attribute:\n#[delegate(SomeTrait)]"
        ).unwrap_or_else(|x| x.to_compile_error());
    }

    let iter = delegate_attributes.map(|attr| delegate_single(input, attr));
    iter.flat_map(|x| x.unwrap_or_else(|err| err.to_compile_error()))
        .collect()
}

pub(super) fn trait_info(trait_path_full: &syn::Path) -> Result<(&Ident, impl ToTokens + '_)> {
    let trait_segment = trait_path_full.segments.last().unwrap();
    let trait_ident: &Ident = &trait_segment.ident;
    let trait_generics = match &trait_segment.arguments {
        PathArguments::None => None,
        PathArguments::AngleBracketed(seg) => Some(super::util::TailingPunctuated(&seg.args)),
        _ => return error!(trait_path_full.span(), "cannot delegate to Fn* traits"),
    };
    Ok((trait_ident, trait_generics))
}

pub(super) fn merge_impl_generics(
    impl_generics: ImplGenerics,
    added_generics: Vec<GenericParam>,
) -> impl Iterator<Item = GenericParam> {
    let tokens = impl_generics.into_token_stream();
    let impl_generics = if tokens.is_empty() {
        Punctuated::new()
    } else {
        let generics: Generics = parse_quote!(#tokens);
        generics.params
    };
    // Make sure all lifetimes come first
    impl_generics.into_iter().merge_by(added_generics, |x, _| {
        matches!(x, GenericParam::Lifetime(_))
    })
}

pub(super) fn merge_generics<'a>(
    impl_generics: &'a Punctuated<GenericParam, Token![,]>,
    added_generics: &'a [GenericParam],
) -> impl Iterator<Item = &'a GenericParam> {
    // Make sure all lifetimes come first
    impl_generics.iter().merge_by(added_generics, |&x, _| {
        matches!(x, GenericParam::Lifetime(_))
    })
}

pub(super) fn build_where_clause(
    mut explicit_where_clauses: Punctuated<WherePredicate, Token![,]>,
    where_clause: Option<&WhereClause>,
) -> WhereClause {
    // Merges the where clause based on the type generics with all the where clauses specified
    // via "where" macro attributes.
    explicit_where_clauses.extend(where_clause.into_iter().flat_map(|n| n.predicates.clone()));
    WhereClause {
        where_token: Default::default(),
        predicates: explicit_where_clauses,
    }
}

pub(super) fn add_auto_where_clause(
    clause: &mut WhereClause,
    trait_path_full: &syn::Path,
    ty: &syn::Type,
) {
    clause.predicates.push(parse_quote!(#ty : #trait_path_full))
}
