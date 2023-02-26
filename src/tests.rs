use super::*;
use pretty_assertions::assert_eq;

#[test]
fn unsupported_trait() {
    assert_eq! {
        auto_ops_impl(
            TokenStream::new(),
        quote!{
            impl<T> Clone for A<T>
            where T: Clone{
                fn clone(&self) -> Self {
                    Self(self.0.clone())
                }
            }
        }).to_string(),
        quote!{
            compile_error!{ "unexpacted Ident: Clone" }
        }.to_string()
    };
}

#[test]
fn add_assign1() {
    assert_eq! {
        auto_ops_impl(
            TokenStream::new(),
            quote! {
                impl<'a, M> AddAssign<&'a A<M>> for A<M>
                where
                    M: Sized + Zero + for<'x> AddAssign<&'x M>,
                {
                    fn add_assign(&mut self, other: &Self) {
                        self.0 += &other.0;
                    }
                }
            },
        ).to_string(),
        quote!{
            impl<'a, M> AddAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                fn add_assign(&mut self, other: &Self) {
                    self.0 += &other.0;
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> AddAssign<A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                fn add_assign(&mut self, rhs: A<M>) {
                    let rhs = &rhs;
                    self.add_assign(rhs);
                }
            }
            impl<'a, M> Add<&'a A<M> > for &'a A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn add(self, rhs: &'a A<M>) -> Self::Output {
                    let mut lhs = self.clone();
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a, M> Add<A<M> > for &'a A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn add(self, rhs: A<M>) -> Self::Output {
                    let mut lhs = self.clone();
                    let rhs = &rhs;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a, M> Add<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                type Output = A<M>;
                fn add(self, rhs: &'a A<M>) -> Self::Output {
                    let mut lhs = self;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> Add<A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                type Output = A<M>;
                fn add(self, rhs: A<M>) -> Self::Output {
                    let mut lhs = self;
                    let rhs = &rhs;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
        }.to_string()
    };
}
#[test]
fn add_assign2() {
    assert_eq! {
        auto_ops_impl(
            TokenStream::new(),
            quote! {
                impl<'a> AddAssign<&'a B> for B {
                    fn add_assign(&mut self, other: &Self) {
                        self.0 += &other.0;
                    }
                }
            },
        ).to_string(),
        quote!{
            impl<'a> AddAssign<&'a B> for B {
                fn add_assign(&mut self, other: &Self) {
                    self.0 += &other.0;
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a> AddAssign<B> for B {
                fn add_assign(&mut self, rhs: B) {
                    let rhs = &rhs;
                    self.add_assign(rhs);
                }
            }
            impl<'a> Add<&'a B> for &'a B
            where
                B: Clone,
            {
                type Output = B;
                fn add(self, rhs: &'a B) -> Self::Output {
                    let mut lhs = self.clone();
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a> Add<B> for &'a B
            where
                B: Clone,
            {
                type Output = B;
                fn add(self, rhs: B) -> Self::Output {
                    let mut lhs = self.clone();
                    let rhs = &rhs;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a> Add<&'a B> for B {
                type Output = B;
                fn add(self, rhs: &'a B) -> Self::Output {
                    let mut lhs = self;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a> Add<B> for B {
                type Output = B;
                fn add(self, rhs: B) -> Self::Output {
                    let mut lhs = self;
                    let rhs = &rhs;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
        }.to_string()
    };
}
#[test]
fn mul() {
    assert_eq! {
        auto_ops_impl(
            TokenStream::new(),
            quote! {
                impl<'a, M> Mul for &'a A<M>
                where
                    M: Sized + Zero,
                    for<'x> &'x M: Mul<Output = M>,
                {
                    type Output = A<M>;
                    fn mul(self, other: Self) -> Self::Output {
                        A(&self.0 * &other.0)
                    }
                }
            },
        ).to_string(),
        quote!{
            impl<'a, M> MulAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Mul<Output = M>,
            {
                fn mul_assign(&mut self, rhs: &'a A<M>) {
                    *self = (&*self).mul(rhs);
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> MulAssign<A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Mul<Output = M>,
            {
                fn mul_assign(&mut self, rhs: A<M>) {
                    let rhs = &rhs;
                    *self = (&*self).mul(rhs);
                }
            }
            impl<'a, M> Mul for &'a A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Mul<Output = M>,
            {
                type Output = A<M>;
                fn mul(self, other: Self) -> Self::Output {
                    A(&self.0 * &other.0)
                }
            }
            impl<'a, M> Mul<A<M> > for &'a A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Mul<Output = M>,
            {
                type Output = A<M>;
                fn mul(self, rhs: A<M>) -> Self::Output {
                    let lhs = self;
                    let rhs = &rhs;
                    lhs.mul(rhs)
                }
            }
            impl<'a, M> Mul<&'a A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Mul<Output = M>,
            {
                type Output = A<M>;
                fn mul(self, rhs: &'a A<M>) -> Self::Output {
                    let lhs = &self;
                    lhs.mul(rhs)
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> Mul<A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Mul<Output = M>,
            {
                type Output = A<M>;
                fn mul(self, rhs: A<M>) -> Self::Output {
                    let lhs = &self;
                    let rhs = &rhs;
                    lhs.mul(rhs)
                }
            }
        }.to_string()
    };
}

#[test]
#[cfg(not(feature = "take_mut"))]
fn div() {
    assert_eq! {
        auto_ops_impl(
            TokenStream::new(),
            quote! {
                impl<'a, M> Div<&'a A<M>> for A<M>
                where
                    M: Sized + Zero,
                    for<'x> &'x M: Div<Output = M>,
                {
                    type Output = Self;
                    fn div(self, other: &Self) -> Self::Output {
                        A(&self.0 / &other.0)
                    }
                }
            },
        ).to_string(),
        quote!{
            impl<'a, M> DivAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
                A<M>: Default,
            {
                fn div_assign(&mut self, rhs: &'a A<M>) {
                    let mut t = Self::default();
                    std::mem::swap(&mut t, self);
                    let mut u = t.div(rhs);
                    std::mem::swap(&mut u, self);
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> DivAssign<A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
                A<M>: Default,
            {
                fn div_assign(&mut self, rhs: A<M>) {
                    let rhs = &rhs;
                    let mut t = Self::default();
                    std::mem::swap(&mut t, self);
                    let mut u = t.div(rhs);
                    std::mem::swap(&mut u, self);
                }
            }
            impl<'a, M> Div<&'a A<M> > for &'a A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn div(self, rhs: &'a A<M>) -> Self::Output {
                    let lhs = self.clone();
                    lhs.div(rhs)
                }
            }
            impl<'a, M> Div<A<M> > for &'a A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn div(self, rhs: A<M>) -> Self::Output {
                    let lhs = self.clone();
                    let rhs = &rhs;
                    lhs.div(rhs)
                }
            }
            impl<'a, M> Div<&'a A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
            {
                type Output = Self;
                fn div(self, other: &Self) -> Self::Output {
                    A(&self.0 / &other.0)
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> Div<A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
            {
                type Output = A<M>;
                fn div(self, rhs: A<M>) -> Self::Output {
                    let lhs = self;
                    let rhs = &rhs;
                    lhs.div(rhs)
                }
            }
        }.to_string()
    };
}

#[test]
#[cfg(feature = "take_mut")]
fn div() {
    assert_eq! {
        auto_ops_impl(
            TokenStream::new(),
            quote! {
                impl<'a, M> Div<&'a A<M>> for A<M>
                where
                    M: Sized + Zero,
                    for<'x> &'x M: Div<Output = M>,
                {
                    type Output = Self;
                    fn div(self, other: &Self) -> Self::Output {
                        A(&self.0 / &other.0)
                    }
                }
            },
        ).to_string(),
        quote!{
            impl<'a, M> DivAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
            {
                fn div_assign(&mut self, rhs: &'a A<M>) {
                    take_mut::take(self, |x| x.div(rhs));
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> DivAssign<A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
            {
                fn div_assign(&mut self, rhs: A<M>) {
                    let rhs = &rhs;
                    take_mut::take(self, |x| x.div(rhs));
                }
            }
            impl<'a, M> Div<&'a A<M> > for &'a A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn div(self, rhs: &'a A<M>) -> Self::Output {
                    let lhs = self.clone();
                    lhs.div(rhs)
                }
            }
            impl<'a, M> Div<A<M> > for &'a A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn div(self, rhs: A<M>) -> Self::Output {
                    let lhs = self.clone();
                    let rhs = &rhs;
                    lhs.div(rhs)
                }
            }
            impl<'a, M> Div<&'a A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
            {
                type Output = Self;
                fn div(self, other: &Self) -> Self::Output {
                    A(&self.0 / &other.0)
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> Div<A<M> > for A<M>
            where
                M: Sized + Zero,
                for<'x> &'x M: Div<Output = M>,
            {
                type Output = A<M>;
                fn div(self, rhs: A<M>) -> Self::Output {
                    let lhs = self;
                    let rhs = &rhs;
                    lhs.div(rhs)
                }
            }
        }.to_string()
    };
}

#[test]
fn add_assign_no_commma() {
    assert_eq! {
        auto_ops_impl(
            TokenStream::new(),
            quote! {
                impl<'a, M> AddAssign<&'a A<M>> for A<M>
                where
                    M: Sized + Zero + for<'x> AddAssign<&'x M>
                {
                    fn add_assign(&mut self, other: &Self) {
                        self.0 += &other.0;
                    }
                }
            },
        ).to_string(),
        quote!{
            impl<'a, M> AddAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>
            {
                fn add_assign(&mut self, other: &Self) {
                    self.0 += &other.0;
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> AddAssign<A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>
            {
                fn add_assign(&mut self, rhs: A<M>) {
                    let rhs = &rhs;
                    self.add_assign(rhs);
                }
            }
            impl<'a, M> Add<&'a A<M> > for &'a A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn add(self, rhs: &'a A<M>) -> Self::Output {
                    let mut lhs = self.clone();
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a, M> Add<A<M> > for &'a A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn add(self, rhs: A<M>) -> Self::Output {
                    let mut lhs = self.clone();
                    let rhs = &rhs;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a, M> Add<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>
            {
                type Output = A<M>;
                fn add(self, rhs: &'a A<M>) -> Self::Output {
                    let mut lhs = self;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> Add<A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>
            {
                type Output = A<M>;
                fn add(self, rhs: A<M>) -> Self::Output {
                    let mut lhs = self;
                    let rhs = &rhs;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
        }
    };
}

#[test]
fn add_assign_select() {
    assert_eq! {
        auto_ops_impl(
            quote! {
                assign_val,
            },
            quote! {
                impl<'a, M> AddAssign<&'a A<M>> for A<M>
                where
                    M: Sized + Zero + for<'x> AddAssign<&'x M>,
                {
                    fn add_assign(&mut self, other: &Self) {
                        self.0 += &other.0;
                    }
                }
            },
        ),
        quote!{
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> AddAssign<A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                fn add_assign(&mut self, rhs: A<M>) {
                    let rhs = &rhs;
                    self.add_assign(rhs);
                }
            }
            impl<'a, M> AddAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                fn add_assign(&mut self, other: &Self) {
                    self.0 += &other.0;
                }
            }
        }
    };
    assert_eq! {
        auto_ops_impl(
            quote! {
                ref_ref,
            },
            quote! {
                impl<'a, M> AddAssign<&'a A<M>> for A<M>
                where
                    M: Sized + Zero + for<'x> AddAssign<&'x M>,
                {
                    fn add_assign(&mut self, other: &Self) {
                        self.0 += &other.0;
                    }
                }
            },
        ),
        quote!{
            impl<'a, M> Add<&'a A<M> > for &'a A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn add(self, rhs: &'a A<M>) -> Self::Output {
                    let mut lhs = self.clone();
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a, M> AddAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                fn add_assign(&mut self, other: &Self) {
                    self.0 += &other.0;
                }
            }
        }
    };
    assert_eq! {
        auto_ops_impl(
            quote! {
                ref_val,
            },
            quote! {
                impl<'a, M> AddAssign<&'a A<M>> for A<M>
                where
                    M: Sized + Zero + for<'x> AddAssign<&'x M>,
                {
                    fn add_assign(&mut self, other: &Self) {
                        self.0 += &other.0;
                    }
                }
            },
        ),
        quote!{
            impl<'a, M> Add<A<M> > for &'a A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
                A<M>: Clone,
            {
                type Output = A<M>;
                fn add(self, rhs: A<M>) -> Self::Output {
                    let mut lhs = self.clone();
                    let rhs = &rhs;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a, M> AddAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                fn add_assign(&mut self, other: &Self) {
                    self.0 += &other.0;
                }
            }
        }
    };
    assert_eq! {
        auto_ops_impl(
            quote! {
                val_ref,
            },
            quote! {
                impl<'a, M> AddAssign<&'a A<M>> for A<M>
                where
                    M: Sized + Zero + for<'x> AddAssign<&'x M>,
                {
                    fn add_assign(&mut self, other: &Self) {
                        self.0 += &other.0;
                    }
                }
            },
        ),
        quote!{
            impl<'a, M> Add<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                type Output = A<M>;
                fn add(self, rhs: &'a A<M>) -> Self::Output {
                    let mut lhs = self;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a, M> AddAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                fn add_assign(&mut self, other: &Self) {
                    self.0 += &other.0;
                }
            }
        }
    };
    assert_eq! {
        auto_ops_impl(
            quote! {
                val_val,
            },
            quote! {
                impl<'a, M> AddAssign<&'a A<M>> for A<M>
                where
                    M: Sized + Zero + for<'x> AddAssign<&'x M>,
                {
                    fn add_assign(&mut self, other: &Self) {
                        self.0 += &other.0;
                    }
                }
            },
        ),
        quote!{
            #[allow(clippy::extra_unused_lifetimes)]
            impl<'a, M> Add<A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                type Output = A<M>;
                fn add(self, rhs: A<M>) -> Self::Output {
                    let mut lhs = self;
                    let rhs = &rhs;
                    lhs.add_assign(rhs);
                    lhs
                }
            }
            impl<'a, M> AddAssign<&'a A<M> > for A<M>
            where
                M: Sized + Zero + for<'x> AddAssign<&'x M>,
            {
                fn add_assign(&mut self, other: &Self) {
                    self.0 += &other.0;
                }
            }
        }.to_string()
    };
}
