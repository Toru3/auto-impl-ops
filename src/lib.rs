#![doc = include_str!("../README.md")]
#[cfg(test)]
mod tests;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use strum::{Display, EnumString};
use syn::{parse::Parser, punctuated::Punctuated, spanned::Spanned, *};

fn is_ref(type_: &Type) -> bool {
    matches!(type_, Type::Reference(_))
}

fn remove_reference(type_: &Type) -> &Type {
    match type_ {
        Type::Reference(ref_) => &ref_.elem,
        _ => type_,
    }
}

fn copy_reference(target: &Type, source: &Type) -> Type {
    match source {
        Type::Reference(inner) => {
            let mut out = inner.clone();
            out.elem = Box::new(target.clone());
            Type::Reference(out)
        }
        _ => target.clone(),
    }
}

fn get_last_segment(implement: &ItemImpl) -> Result<&PathSegment> {
    if implement.trait_.is_none() {
        return Err(Error::new(implement.span(), "Is not Trait impl"));
    };
    let trait_ = implement.trait_.as_ref().unwrap();
    if let Some(bang) = trait_.0 {
        return Err(Error::new(bang.span(), "Unexpected negative impl"));
    }
    let segments = &trait_.1.segments;
    if segments.is_empty() {
        return Err(Error::new(segments.span(), "Unexpected empty trait path"));
    }
    Ok(segments.last().unwrap())
}

fn get_rhs_type<'a>(args: &'a PathArguments, self_type: &'a Type) -> Result<&'a Type> {
    match args {
        PathArguments::None => Ok(self_type),
        PathArguments::AngleBracketed(args) => {
            let args = &args.args;
            if args.len() != 1 {
                return Err(Error::new(
                    args.span(),
                    "Number of trait arguments is not 1",
                ));
            }
            if let GenericArgument::Type(rhs_type) = args.first().unwrap() {
                Ok(rhs_type)
            } else {
                Err(Error::new(args.span(), "Is not type"))
            }
        }
        _ => Err(Error::new(args.span(), "Unexpected trait arguments")),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Operate(OpTrait, bool, bool);
impl Operate {
    fn lhs_move(&self) -> bool {
        !self.0.is_assign() && !self.1
    }
    fn rhs_move(&self) -> bool {
        !self.2
    }
    fn require_lhs_clone(&self, op: Self) -> bool {
        (self.lhs_move() || self.0.is_assign()) && op.1
    }
    fn require_rhs_clone(&self, op: Self) -> bool {
        self.rhs_move() && op.2
    }
    fn require_clone(&self, op: Self) -> bool {
        self.require_lhs_clone(op) || self.require_rhs_clone(op)
    }
}

#[derive(Clone, Debug)]
struct Generator<'a> {
    implement: &'a ItemImpl,
    source_op: Operate,
    self_type: &'a Type,
    rhs_type: &'a Type,
}
impl<'a> Generator<'a> {
    fn get_arg_type(is_ref_: bool, target: &Type, source: &Type) -> Type {
        if !is_ref_ {
            remove_reference(target).clone()
        } else if is_ref(target) {
            target.clone()
        } else if is_ref(source) {
            copy_reference(target, source)
        } else {
            parse_quote! {
                &#target
            }
        }
    }
    fn update_where_clause(&self, generics: &mut Generics, op: Operate) {
        let self_type = self.self_type;
        if self.source_op.require_clone(op) {
            let wc = generics.make_where_clause();
            wc.predicates.push(parse_quote! {
                #self_type: Clone
            });
        }
        if self.source_op.lhs_move() && op.0.is_assign() && cfg!(not(feature = "take_mut")) {
            let wc = generics.make_where_clause();
            wc.predicates.push(parse_quote! {
                #self_type: Default
            });
        }
    }
    fn assgin_body(source_op: Operate) -> TokenStream {
        let source_fn_name = source_op.0.to_func_ident();
        if source_op.0.is_assign() {
            quote! {
                self.#source_fn_name(rhs);
            }
        } else if source_op.1 {
            quote! {
                *self = (&*self).#source_fn_name(rhs);
            }
        } else if cfg!(feature = "take_mut") {
            quote! {
                take_mut::take(self, |x| x.#source_fn_name(rhs));
            }
        } else {
            quote! {
                let mut t = Self::default();
                std::mem::swap(&mut t, self);
                let mut u = t.#source_fn_name(rhs);
                std::mem::swap(&mut u, self);
            }
        }
    }
    fn gen_rhs(source_op: Operate, op: Operate) -> TokenStream {
        #[allow(clippy::collapsible_else_if)]
        if source_op.2 {
            if op.2 {
                TokenStream::new()
            } else {
                quote!(let rhs = &rhs;)
            }
        } else {
            if op.2 {
                quote!(let rhs = rhs.clone();)
            } else {
                TokenStream::new()
            }
        }
    }
    fn gen_lhs(source_op: Operate, op: Operate) -> TokenStream {
        #[allow(clippy::collapsible_else_if)]
        if source_op.0.is_assign() {
            if op.1 {
                quote!(let mut lhs = self.clone();)
            } else {
                quote!(let mut lhs = self;)
            }
        } else if source_op.1 {
            if op.1 {
                quote!(let lhs = self;)
            } else {
                quote!(let lhs = &self;)
            }
        } else {
            if op.1 {
                quote!(let lhs = self.clone();)
            } else {
                quote!(let lhs = self;)
            }
        }
    }
    fn generate(&self, op: Operate) -> Result<TokenStream> {
        if op.0.is_assign() && op.1 {
            return Err(Error::new(
                Span::call_site(),
                "Type of LHS of assign operations must not reference",
            ));
        }
        if op == self.source_op {
            return Ok(self.implement.to_token_stream());
        }
        let mut work = self.implement.clone();
        if let Operate(_, false, false) = op {
            work.attrs.push(parse_quote! {
                #[allow(clippy::extra_unused_lifetimes)]
            });
        }
        let rhs_type = Self::get_arg_type(op.2, self.rhs_type, self.self_type);
        let trait_ = op.0;
        *work.trait_.as_mut().unwrap().1.segments.last_mut().unwrap() =
            parse_quote! { #trait_<#rhs_type> };
        *work.self_ty.as_mut() = Self::get_arg_type(op.1, self.self_type, self.rhs_type);
        self.update_where_clause(&mut work.generics, op);
        work.items.clear();
        let fn_name = op.0.to_func_ident();
        let preamble_rhs = Self::gen_rhs(self.source_op, op);
        if op.0.is_assign() {
            let body = Self::assgin_body(self.source_op);
            work.items.push(parse_quote! {
                fn #fn_name(&mut self, rhs: #rhs_type) {
                    #preamble_rhs
                    #body
                }
            });
        } else {
            let rr_self_type = remove_reference(self.self_type);
            work.items.push(parse_quote! {
                type Output = #rr_self_type;
            });
            let preamble_lhs = Self::gen_lhs(self.source_op, op);
            let source_fn_name = self.source_op.0.to_func_ident();
            let body = if self.source_op.0.is_assign() {
                quote! {
                    lhs.#source_fn_name(rhs);
                    lhs
                }
            } else {
                quote! {
                    lhs.#source_fn_name(rhs)
                }
            };
            work.items.push(parse_quote! {
                fn #fn_name(self, rhs: #rhs_type) -> Self::Output {
                    #preamble_lhs
                    #preamble_rhs
                    #body
                }
            });
        }
        Ok(quote!(#work))
    }
}

type Attributes = Punctuated<Ident, token::Comma>;
fn auto_ops_generate(mut attrs: Attributes, implement: ItemImpl) -> Result<TokenStream> {
    let last_segment = get_last_segment(&implement)?;
    let op: OpTrait = last_segment.ident.clone().try_into()?;
    let self_type = &implement.self_ty;
    let rhs_type = get_rhs_type(&last_segment.arguments, self_type)?;
    let generator = Generator {
        implement: &implement,
        source_op: Operate(op, is_ref(self_type), is_ref(rhs_type)),
        self_type,
        rhs_type,
    };
    let list = [
        ("assign_ref", Operate(op.to_assign(), false, true)),
        ("assign_val", Operate(op.to_assign(), false, false)),
        ("ref_ref", Operate(op.to_non_assign(), true, true)),
        ("ref_val", Operate(op.to_non_assign(), true, false)),
        ("val_ref", Operate(op.to_non_assign(), false, true)),
        ("val_val", Operate(op.to_non_assign(), false, false)),
    ];
    let map = HashMap::from(list);
    let rev_map = list.iter().map(|&(v, k)| (k, v)).collect::<HashMap<_, _>>();
    if attrs.is_empty() {
        attrs = list.iter().map(|(x, _)| format_ident!("{}", x)).collect();
    }
    let source = rev_map[&generator.source_op];
    if !attrs.iter().any(|x| x == source) {
        attrs.push(format_ident!("{}", source));
    }
    let mut result = TokenStream::new();
    for i in attrs.iter() {
        let s = i.to_string();
        if let Some(op) = map.get(s.as_str()) {
            let code = generator.generate(*op)?;
            result.extend(code);
        }
    }
    Ok(result)
}

#[derive(Clone, Copy, Debug, Display, EnumString, PartialEq, Eq, Hash)]
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
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    BitXor,
    BitXorAssign,
    Shl,
    ShlAssign,
    Shr,
    ShrAssign,
}
impl TryFrom<Ident> for OpTrait {
    type Error = Error;
    fn try_from(ident: Ident) -> Result<Self> {
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
            BitAnd | BitAndAssign => BitAndAssign,
            BitOr | BitOrAssign => BitOrAssign,
            BitXor | BitXorAssign => BitXorAssign,
            Shl | ShlAssign => ShlAssign,
            Shr | ShrAssign => ShrAssign,
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
            BitAnd | BitAndAssign => BitAnd,
            BitOr | BitOrAssign => BitOr,
            BitXor | BitXorAssign => BitXor,
            Shl | ShlAssign => Shl,
            Shr | ShrAssign => Shr,
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
            BitAnd => format_ident!("bitand"),
            BitAndAssign => format_ident!("bitand_assign"),
            BitOr => format_ident!("bitor"),
            BitOrAssign => format_ident!("bitor_assign"),
            BitXor => format_ident!("bitxor"),
            BitXorAssign => format_ident!("bitxor_assign"),
            Shl => format_ident!("shl"),
            ShlAssign => format_ident!("shl_assign"),
            Shr => format_ident!("shr"),
            ShrAssign => format_ident!("shr_assign"),
        }
    }
}

fn auto_ops_impl_inner(attrs: TokenStream, tokens: TokenStream) -> Result<TokenStream> {
    let a = Punctuated::parse_terminated.parse2(attrs)?;
    let i = parse2(tokens)?;
    auto_ops_generate(a, i)
}

fn auto_ops_impl(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    auto_ops_impl_inner(attrs, tokens).unwrap_or_else(Error::into_compile_error)
}

#[proc_macro_attribute]
pub fn auto_ops(
    attrs: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    auto_ops_impl(attrs.into(), tokens.into()).into()
}
