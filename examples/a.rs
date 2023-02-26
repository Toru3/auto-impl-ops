use auto_impl_ops::*;
use num_traits::*;
use std::ops::*;

#[derive(Clone, Default)]
struct A<T>(T);
#[derive(Clone, Default)]
struct B(i32);

// from assign_ref
#[auto_ops]
impl<'a, M> AddAssign<&'a A<M>> for A<M>
where
    M: Sized + Zero + for<'x> AddAssign<&'x M>,
{
    fn add_assign(&mut self, other: &Self) {
        self.0 += &other.0;
    }
}

#[auto_ops]
impl<'a, G> SubAssign<&'a A<G>> for A<G>
where
    G: Sized + Zero + for<'x> SubAssign<&'x G>,
{
    fn sub_assign(&mut self, other: &Self) {
        self.0 -= &other.0;
    }
}

#[auto_ops]
impl<'a, K> RemAssign<&'a A<K>> for A<K>
where
    K: Sized
        + Clone
        + Zero
        + for<'x> AddAssign<&'x K>
        + for<'x> SubAssign<&'x K>
        + for<'x> RemAssign<&'x K>,
    for<'x> &'x K: Mul<Output = K> + Div<Output = K>,
{
    fn rem_assign(&mut self, other: &Self) {
        self.0 %= &other.0;
    }
}

#[auto_ops]
impl<'a, R> MulAssign<&'a R> for A<R>
where
    R: Sized + Zero + for<'x> MulAssign<&'x R>,
{
    fn mul_assign(&mut self, other: &R) {
        self.0 *= other;
    }
}

#[auto_ops]
impl<'a, R> DivAssign<&'a R> for A<R>
where
    R: Sized + Zero + for<'x> DivAssign<&'x R>,
{
    fn div_assign(&mut self, other: &R) {
        self.0 /= other;
    }
}

#[auto_ops]
impl<'a> AddAssign<&'a B> for B {
    fn add_assign(&mut self, other: &Self) {
        self.0 += &other.0;
    }
}

// from ref_ref
#[auto_ops]
impl<M> Mul for &A<M>
where
    M: Sized + Zero,
    for<'x> &'x M: Mul<Output = M>,
{
    type Output = A<M>;
    fn mul(self, other: Self) -> Self::Output {
        A(&self.0 * &other.0)
    }
}

// from val_ref
#[auto_ops]
impl<M> Div<&A<M>> for A<M>
where
    M: Sized + Zero,
    for<'x> &'x M: Div<Output = M>,
{
    type Output = Self;
    fn div(self, other: &Self) -> Self::Output {
        A(&self.0 / &other.0)
    }
}

#[auto_ops]
impl<'a, M> BitAndAssign<&'a A<M>> for A<M>
where
    M: Sized + for<'x> BitAndAssign<&'x M>,
{
    fn bitand_assign(&mut self, other: &Self) {
        self.0 &= &other.0;
    }
}

#[auto_ops]
impl<'a, M> BitOrAssign<&'a A<M>> for A<M>
where
    M: Sized + for<'x> BitOrAssign<&'x M>,
{
    fn bitor_assign(&mut self, other: &Self) {
        self.0 |= &other.0;
    }
}

#[auto_ops]
impl<'a, M> BitXorAssign<&'a A<M>> for A<M>
where
    M: Sized + for<'x> BitXorAssign<&'x M>,
{
    fn bitxor_assign(&mut self, other: &Self) {
        self.0 ^= &other.0;
    }
}

#[auto_ops]
impl<'a, M> ShlAssign<&'a A<M>> for A<M>
where
    M: Sized + for<'x> ShlAssign<&'x M>,
{
    fn shl_assign(&mut self, other: &Self) {
        self.0 <<= &other.0;
    }
}

#[auto_ops]
impl<'a, M> ShrAssign<&'a A<M>> for A<M>
where
    M: Sized + for<'x> ShrAssign<&'x M>,
{
    fn shr_assign(&mut self, other: &Self) {
        self.0 >>= &other.0;
    }
}

#[auto_ops]
impl<M> ShlAssign<u8> for A<M>
where
    M: Sized + ShlAssign<u8>,
{
    fn shl_assign(&mut self, other: u8) {
        self.0 <<= other;
    }
}

#[auto_ops]
impl<M> ShrAssign<u8> for A<M>
where
    M: Sized + ShrAssign<u8>,
{
    fn shr_assign(&mut self, other: u8) {
        self.0 >>= other;
    }
}

#[derive(Clone, Default)]
struct C<T>(T);

#[auto_ops(val_val, ref_val)]
impl<T: AddAssign> AddAssign for C<T> {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}
#[auto_ops(val_ref, ref_ref)]
impl<T> AddAssign<&C<T>> for C<T>
where
    T: for<'x> AddAssign<&'x T>,
{
    fn add_assign(&mut self, other: &Self) {
        self.0 += &other.0;
    }
}

#[derive(Clone, Default)]
struct D<T>(T);

// from val_val
#[auto_ops]
impl<T> Add for D<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, other: Self) -> Self {
        D(self.0 + other.0)
    }
}

#[derive(Clone, Default)]
struct E<T>(T);

// from assign_val
#[auto_ops]
impl<T> AddAssign<&E<T>> for E<T>
where
    T: for<'x> AddAssign<&'x T>,
{
    fn add_assign(&mut self, other: &Self) {
        self.0 += &other.0;
    }
}

#[derive(Clone, Default)]
struct F<T>(T);

// from ref_val
#[auto_ops]
impl<'a, T> Add<F<T>> for &'a F<T>
where
    for<'x> &'x T: Add<T, Output = T>,
{
    type Output = F<T>;
    fn add(self, other: F<T>) -> F<T> {
        F(&self.0 + other.0)
    }
}

fn main() {}
