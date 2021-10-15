extern crate proc_macro;

use proc_macro::TokenStream;

use itertools::Itertools;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{ExprBlock, Token};
use syn::parse::{Parse, ParseStream, Result};

const TUPLE_LENGTH: usize = 24;

#[proc_macro]
pub fn impl_contains_for_tuples(_: TokenStream) -> TokenStream {
    let all_idents: Vec<_> = ('A'..='Z').chain('a'..='z').map(|x| format_ident!("T{}", x)).collect();

    (1..=TUPLE_LENGTH).map(|i| {
        let type_vars = &all_idents[0..i];
        type_vars.iter().map(|main| {
                let without = type_vars.iter().filter(|&x| x != main);
                let constraints = type_vars.iter().cartesian_product(type_vars.iter())
                    .filter(|(l, r)| l != r)
                    .map(|(l, r)| quote! {
                    (#l, #r): NotEqual
                });
                quote! {
                    unsafe impl<#(#type_vars),*> Contains<#main> for (#(#type_vars,)*) where #(#constraints),* {
                        type Without = (#(#without,)*);
                    }
                }
            }
        ).collect::<TokenStream2>()
    }).collect::<TokenStream2>().into()
}


#[proc_macro]
pub fn invoke_with_idents(t: TokenStream) -> TokenStream {
    let all_idents: Vec<_> = ('A'..='Z').chain('a'..='z').map(|x| format_ident!("T{}", x)).collect();
    let next = syn::parse::<Ident>(t).unwrap();
    let idents = (1..=TUPLE_LENGTH).flat_map(|i| {
        let type_vars = &all_idents[0..i];
        quote! {
            #(#type_vars)*;
        }
    }).collect::<TokenStream2>();
    quote! {
        #next!{#idents}
    }.into()
}
struct MatchTypeInput {
    expr: syn::Expr,
    arms: Vec<Arm>,
    default: DefaultArm,
}

impl Parse for MatchTypeInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = input.parse()?;
        input.parse::<Token![in]>()?;
        let arms = std::iter::from_fn(|| {
            if !input.peek(Token![_]) && !input.is_empty() {
                Some(input.parse())
            } else {
                None
            }
        }).collect::<Result<_>>()?;
        let default = if input.peek(Token![_]) {
            input.parse()?
        } else { Default::default() };
        Ok(Self {
            expr,
            arms,
            default,
        })
    }
}

struct DefaultArm(syn::Ident, syn::Expr);

impl Default for DefaultArm {
    fn default() -> Self {
        Self(
            format_ident!("e"),
            syn::parse_quote!(
                e.infallible()
            ),
        )
    }
}

impl Parse for DefaultArm {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![_]>()?;
        input.parse::<Token![as]>()?;
        let ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let expr = input.parse::<ExprBlock>()?;
        Ok(Self(ident, expr.into()))
    }
}

struct Arm(syn::Type, syn::Ident, syn::Expr);

impl Parse for Arm {
    fn parse(input: ParseStream) -> Result<Self> {
        let r#type = input.parse()?;
        input.parse::<Token![as]>()?;
        let ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let expr = input.parse::<ExprBlock>()?;
        Ok(Self(r#type, ident, expr.into()))
    }
}

#[proc_macro]
pub fn match_type(input: TokenStream) -> TokenStream {
    let MatchTypeInput{ default, expr, arms } = syn::parse_macro_input!(input as MatchTypeInput);
    let input_ident = format_ident!("e");
    let next = matches(input_ident.clone(), &arms, default);
    (quote! {
        {
            let #input_ident = #expr;
            #next
        }
    }).into()
}

fn matches(id: Ident, arms: &[Arm], default: DefaultArm) -> TokenStream2 {
    assert!(!arms.is_empty());
    let (Arm(first_ty, first_id, first_exp), rest) = arms.split_first().unwrap();
    let continuation = if rest.is_empty() {
        let DefaultArm(id, expr) = default;
        quote! {
            Err(#id) => #expr
        }
    } else {
        let next = matches(id.clone(), rest, default);
        quote! {
            Err(#id) => #next
        }
    };
    quote! {
        match #id.take::<#first_ty>() {
            Ok(#first_id) => #first_exp
            #continuation
        }
    }
}