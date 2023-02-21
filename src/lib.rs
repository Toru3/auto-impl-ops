#![doc = include_str!("../README.md")]
#[cfg(test)]
mod tests;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::str::FromStr;
use strum::{Display, EnumString};
use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    *,
};

fn remove_reference(ref_: &TypeReference) -> &Type {
    &ref_.elem
}

fn add_reference(target: Type, ref_: TypeReference) -> TypeReference {
    let mut out = ref_;
    out.elem = Box::new(target);
    out
}

fn generate_where_clause(where_clause: &Option<WhereClause>) -> TokenStream {
    if let Some(x) = where_clause {
        if x.predicates.trailing_punct() {
            x.to_token_stream()
        } else {
            quote! {
                #x,
            }
        }
    } else {
        quote! {where}
    }
}

fn ref_assign(implement: &OpImpl, op: OpTrait, rhs_type: &TypeReference) -> Result<TokenStream> {
    let mut no_ref_impl = implement.clone();
    *no_ref_impl.op_trait.arg.as_mut().unwrap() = remove_reference(rhs_type).clone();
    let last_arg = no_ref_impl.group.inputs.last_mut().unwrap();
    if let FnArg::Typed(rhs) = last_arg {
        *rhs.pat = Pat::Ident(PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: format_ident!("rhs"),
            subpat: None,
        });
        *rhs.ty = remove_reference(rhs_type).clone();
    } else {
        return Err(Error::new(
            last_arg.span(),
            format!("unexpacted arg: {}", last_arg.to_token_stream()),
        ));
    }
    let fn_name = op.to_func_ident();
    no_ref_impl.group.block.stmts = vec![parse2::<Stmt>(quote! {
        self.#fn_name(&rhs);
    })
    .unwrap()];
    let generics = &implement.generics;
    let op_non_assign = op.to_non_assign();
    let self_type = &implement.self_type;
    let ref_self_type = add_reference(implement.self_type.clone(), rhs_type.clone());
    let orig_where_clause = &implement.where_clause;
    let where_clause = generate_where_clause(&implement.where_clause);
    let fn_name_non_assign = op_non_assign.to_func_ident();
    let rr_rhs_type = remove_reference(rhs_type);
    let t = quote! {
        #implement
        #[allow(clippy::extra_unused_lifetimes)]
        #no_ref_impl
        impl #generics #op_non_assign<#rhs_type> for #ref_self_type
        #where_clause
            #self_type: Clone,
        {
            type Output = #self_type;
            fn #fn_name_non_assign(self, rhs: #rhs_type) -> Self::Output {
                let mut out = self.clone();
                out.#fn_name(rhs);
                out
            }
        }
        impl #generics #op_non_assign<#rr_rhs_type> for #ref_self_type
        #where_clause
            #self_type: Clone,
        {
            type Output = #self_type;
            fn #fn_name_non_assign(self, rhs: #rr_rhs_type) -> Self::Output {
                let mut out = self.clone();
                out.#fn_name(&rhs);
                out
            }
        }
        impl #generics #op_non_assign<#rhs_type> for #self_type
        #orig_where_clause
        {
            type Output = Self;
            fn #fn_name_non_assign(mut self, rhs: #rhs_type) -> Self::Output {
                self.#fn_name(rhs);
                self
            }
        }
        #[allow(clippy::extra_unused_lifetimes)]
        impl #generics #op_non_assign<#rr_rhs_type> for #self_type
        #orig_where_clause
        {
            type Output = Self;
            fn #fn_name_non_assign(mut self, rhs: #rr_rhs_type) -> Self::Output {
                self.#fn_name(&rhs);
                self
            }
        }
    };
    Ok(t)
}

fn ref_ref(
    implement: &OpImpl,
    op: OpTrait,
    self_type: &TypeReference,
    rhs_type: &TypeReference,
) -> Result<TokenStream> {
    let rr_self_type = remove_reference(self_type);
    let rr_rhs_type = remove_reference(rhs_type);
    let generics = &implement.generics;
    let where_clause = &implement.where_clause;
    let fn_name = op.to_func_ident();
    let op_assign = op.to_assign();
    let assign_fn_name = op_assign.to_func_ident();
    let t = quote! {
        #implement
        impl #generics #op<#rr_rhs_type> for #self_type
        #where_clause
        {
            type Output = #rr_self_type;
            fn #fn_name(self, rhs: #rr_rhs_type) -> Self::Output {
                self.#fn_name(&rhs)
            }
        }
        impl #generics #op<#rhs_type> for #rr_self_type
        #where_clause
        {
            type Output = Self;
            fn #fn_name(self, rhs: #rhs_type) -> Self::Output {
                (&self).#fn_name(rhs)
            }
        }
        impl #generics #op<#rr_rhs_type> for #rr_self_type
        #where_clause
        {
            type Output = Self;
            fn #fn_name(self, rhs: #rr_rhs_type) -> Self::Output {
                (&self).#fn_name(&rhs)
            }
        }
        impl #generics #op_assign<#rhs_type> for #rr_self_type
        #where_clause
        {
            fn #assign_fn_name(&mut self, rhs: #rhs_type) {
                *self = (&*self).#fn_name(rhs);
            }
        }
        impl #generics #op_assign<#rr_rhs_type> for #rr_self_type
        #where_clause
        {
            fn #assign_fn_name(&mut self, rhs: #rr_rhs_type) {
                *self = (&*self).#fn_name(&rhs);
            }
        }
    };
    Ok(t)
}

fn non_ref_ref(
    implement: &OpImpl,
    op: OpTrait,
    self_type: &Type,
    rhs_type: &TypeReference,
) -> Result<TokenStream> {
    let rr_rhs_type = remove_reference(rhs_type);
    let generics = &implement.generics;
    let orig_where_clause = &implement.where_clause;
    let where_clause = generate_where_clause(&implement.where_clause);
    let fn_name = op.to_func_ident();
    let op_assign = op.to_assign();
    let assign_fn_name = op_assign.to_func_ident();
    let take_mut = cfg!(feature = "take_mut");
    let b1 = if take_mut {
        quote! {
            take_mut::take(self, |x| x.#fn_name(rhs));
        }
    } else {
        quote! {
            let mut t = Self::default();
            std::mem::swap(&mut t, self);
            let mut u = t.#fn_name(rhs);
            std::mem::swap(&mut u, self);
        }
    };
    let b2 = if take_mut {
        quote! {
            take_mut::take(self, |x| x.#fn_name(&rhs));
        }
    } else {
        quote! {
            let mut t = Self::default();
            std::mem::swap(&mut t, self);
            let mut u = t.#fn_name(&rhs);
            std::mem::swap(&mut u, self);
        }
    };
    let default = if take_mut {
        quote! {
            #orig_where_clause
        }
    } else {
        quote! {
            #where_clause
                #self_type: Default,
        }
    };
    let t = quote! {
        #implement
        impl #generics #op<#rr_rhs_type> for &#self_type
        #where_clause
            #self_type: Clone,
        {
            type Output = #self_type;
            fn #fn_name(self, rhs: #rr_rhs_type) -> Self::Output {
                self.clone().#fn_name(&rhs)
            }
        }
        impl #generics #op<#rr_rhs_type> for #self_type
        #orig_where_clause
        {
            type Output = Self;
            fn #fn_name(self, rhs: #rr_rhs_type) -> Self::Output {
                self.#fn_name(&rhs)
            }
        }
        impl #generics #op<#rhs_type> for &#self_type
        #where_clause
            #self_type: Clone,
        {
            type Output = #self_type;
            fn #fn_name(self, rhs: #rhs_type) -> Self::Output {
                self.clone().#fn_name(rhs)
            }
        }
        impl #generics #op_assign<#rhs_type> for #self_type
        #default
        {
            fn #assign_fn_name(&mut self, rhs: #rhs_type) {
                #b1
            }
        }
        impl #generics #op_assign<#rr_rhs_type> for #self_type
        #default
        {
            fn #assign_fn_name(&mut self, rhs: #rr_rhs_type) {
                #b2
            }
        }
    };
    Ok(t)
}

fn auto_ops_generate(implement: OpImpl) -> Result<TokenStream> {
    let op = implement.op_trait.ident;
    let type_ = &implement.op_trait.arg;
    if op.is_assign() {
        if let Some(Type::Reference(rhs_type)) = type_ {
            ref_assign(&implement, op, rhs_type)
        } else {
            Err(Error::new(
                Span::call_site(),
                "not implemented: type of RHS is not reference",
            ))
        }
    } else {
        let self_type = &implement.self_type;
        let is_self_ref = matches!(self_type, Type::Reference(_));
        let (is_rhs_ref, rhs_type) = match type_ {
            None => (is_self_ref, self_type),
            Some(Type::Reference(_)) => (true, type_.as_ref().unwrap()),
            Some(x) => (false, x),
        };
        if is_rhs_ref {
            let rhs_type = if let Type::Reference(rhs_type) = rhs_type {
                rhs_type
            } else {
                unreachable!()
            };
            if let Type::Reference(self_type) = self_type {
                ref_ref(&implement, op, self_type, rhs_type)
            } else {
                non_ref_ref(&implement, op, self_type, rhs_type)
            }
        } else {
            Err(Error::new(
                Span::call_site(),
                "not implemented: type of RHS is not reference",
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, Display, EnumString, PartialEq, Eq)]
enum OpTrait {
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Rem,
    RemAssign,
}
impl Parse for OpTrait {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        if let Ok(x) = Self::from_str(&ident.to_string()) {
            Ok(x)
        } else {
            Err(Error::new(
                ident.span(),
                format!("unexpacted Ident: {}", ident),
            ))
        }
    }
}
impl ToTokens for OpTrait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new(&self.to_string(), Span::call_site()));
    }
}

impl OpTrait {
    fn to_assign(self) -> Self {
        use OpTrait::*;
        match self {
            Add | AddAssign => AddAssign,
            Sub | SubAssign => SubAssign,
            Mul | MulAssign => MulAssign,
            Div | DivAssign => DivAssign,
            Rem | RemAssign => RemAssign,
        }
    }
    fn to_non_assign(self) -> Self {
        use OpTrait::*;
        match self {
            Add | AddAssign => Add,
            Sub | SubAssign => Sub,
            Mul | MulAssign => Mul,
            Div | DivAssign => Div,
            Rem | RemAssign => Rem,
        }
    }
    fn is_assign(self) -> bool {
        self.to_assign() == self
    }
    fn to_func_ident(self) -> Ident {
        use OpTrait::*;
        match self {
            Add => format_ident!("add"),
            AddAssign => format_ident!("add_assign"),
            Sub => format_ident!("sub"),
            SubAssign => format_ident!("sub_assign"),
            Mul => format_ident!("mul"),
            MulAssign => format_ident!("mul_assign"),
            Div => format_ident!("div"),
            DivAssign => format_ident!("div_assign"),
            Rem => format_ident!("rem"),
            RemAssign => format_ident!("rem_assign"),
        }
    }
}

#[derive(Clone, Debug)]
struct BracedImplItemMethod {
    item_type: Option<ItemType>,
    fn_token: Token![fn],
    ident: Ident,
    inputs: Punctuated<FnArg, token::Comma>,
    output: ReturnType,
    block: Block,
}
impl Parse for BracedImplItemMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let content2;
        let _brace_token = braced!(content in input);
        let item_type = if content.peek(Token![type]) {
            Some(content.parse()?)
        } else {
            None
        };
        let fn_token = content.parse()?;
        let ident = content.parse()?;
        let _paren_token = parenthesized!(content2 in content);
        let inputs = content2.parse_terminated(FnArg::parse)?;
        let output = content.parse()?;
        let block = content.parse()?;
        Ok(BracedImplItemMethod {
            item_type,
            fn_token,
            ident,
            inputs,
            output,
            block,
        })
    }
}
impl ToTokens for BracedImplItemMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use proc_macro2::{Delimiter, Group};
        let inner = {
            let mut t = TokenStream::new();
            let tokens = &mut t;
            self.item_type.to_tokens(tokens);
            self.fn_token.to_tokens(tokens);
            self.ident.to_tokens(tokens);
            let inner = {
                let mut t = TokenStream::new();
                let tokens = &mut t;
                self.inputs.to_tokens(tokens);
                t
            };
            tokens.append(Group::new(Delimiter::Parenthesis, inner));
            self.output.to_tokens(tokens);
            self.block.to_tokens(tokens);
            t
        };
        tokens.append(Group::new(Delimiter::Brace, inner));
    }
}

#[derive(Clone, Debug, derive_syn_parse::Parse)]
struct RestrictPath {
    ident: OpTrait,
    _colon2_token: Option<token::Colon2>,
    #[prefix(Option<Token![<]> as lt_token)]
    #[parse_if(lt_token.is_some())]
    arg: Option<Type>,
    #[parse_if(lt_token.is_some())]
    _gt_token: Option<token::Gt>,
}
impl ToTokens for RestrictPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use proc_macro2::{Punct, Spacing};
        self.ident.to_tokens(tokens);
        if self.arg.is_some() {
            tokens.append(Punct::new('<', Spacing::Alone));
            self.arg.to_tokens(tokens);
            tokens.append(Punct::new('>', Spacing::Alone));
        }
    }
}

#[derive(Clone, Debug, derive_syn_parse::Parse)]
struct OpImpl {
    impl_token: Token![impl],
    generics: Generics,
    op_trait: RestrictPath,
    for_token: Token![for],
    self_type: Type,
    #[peek(Token![where])]
    where_clause: Option<WhereClause>,
    group: BracedImplItemMethod,
}
impl ToTokens for OpImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.impl_token.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.op_trait.to_tokens(tokens);
        self.for_token.to_tokens(tokens);
        self.self_type.to_tokens(tokens);
        self.where_clause.to_tokens(tokens);
        self.group.to_tokens(tokens);
    }
}

fn auto_ops_attr_parse(attrs: ParseStream) -> Result<TokenStream> {
    if attrs.is_empty() {
        Ok(TokenStream::new())
    } else {
        dbg!(&attrs);
        Err(Error::new(
            Span::call_site(),
            format!("unexpacted arg: {}", attrs),
        ))
    }
}

fn auto_ops_parse(input: ParseStream) -> Result<TokenStream> {
    let implement = input.parse::<OpImpl>()?;
    auto_ops_generate(implement)
}

fn auto_ops_impl(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    let mut a = auto_ops_attr_parse
        .parse2(attrs)
        .unwrap_or_else(Error::into_compile_error);
    let i = auto_ops_parse
        .parse2(tokens)
        .unwrap_or_else(Error::into_compile_error);
    a.extend(i);
    a
}

#[proc_macro_attribute]
pub fn auto_ops(
    attrs: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    auto_ops_impl(attrs.into(), tokens.into()).into()
}
