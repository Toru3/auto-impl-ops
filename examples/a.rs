use auto_impl_ops::*;
use num_traits::*;
use std::ops::*;

#[derive(Clone, Default)]
struct A<T>(T);
#[derive(Clone, Default)]
struct B(i32);

// from assign_ref
#[auto_ops]
impl<M> AddAssign<&A<M>> for A<M>
where
    M: Sized + Zero + for<'x> AddAssign<&'x M>,
{
    fn add_assign(&mut self, other: &Self) {
        self.0 += &other.0;
    }
}

#[auto_ops]
impl<G> SubAssign<&A<G>> for A<G>
where
    G: Sized + Zero + for<'x> SubAssign<&'x G>,
{
    fn sub_assign(&mut self, other: &Self) {
        self.0 -= &other.0;
    }
}

#[auto_ops]
impl<K> RemAssign<&A<K>> for A<K>
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
impl<R> MulAssign<&R> for A<R>
where
    R: Sized + Zero + for<'x> MulAssign<&'x R>,
{
    fn mul_assign(&mut self, other: &R) {
        self.0 *= other;
    }
}

#[auto_ops]
impl<R> DivAssign<&R> for A<R>
where
    R: Sized + Zero + for<'x> DivAssign<&'x R>,
{
    fn div_assign(&mut self, other: &R) {
        self.0 /= other;
    }
}

#[auto_ops]
impl AddAssign<&B> for B {
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
impl<M> BitAndAssign<&A<M>> for A<M>
where
    M: Sized + for<'x> BitAndAssign<&'x M>,
{
    fn bitand_assign(&mut self, other: &Self) {
        self.0 &= &other.0;
    }
}

#[auto_ops]
impl<M> BitOrAssign<&A<M>> for A<M>
where
    M: Sized + for<'x> BitOrAssign<&'x M>,
{
    fn bitor_assign(&mut self, other: &Self) {
        self.0 |= &other.0;
    }
}

#[auto_ops]
impl<M> BitXorAssign<&A<M>> for A<M>
where
    M: Sized + for<'x> BitXorAssign<&'x M>,
{
    fn bitxor_assign(&mut self, other: &Self) {
        self.0 ^= &other.0;
    }
}

#[auto_ops]
impl<M> ShlAssign<&A<M>> for A<M>
where
    M: Sized + for<'x> ShlAssign<&'x M>,
{
    fn shl_assign(&mut self, other: &Self) {
        self.0 <<= &other.0;
    }
}

#[auto_ops]
impl<M> ShrAssign<&A<M>> for A<M>
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
impl<T> Add<F<T>> for &F<T>
where
    for<'x> &'x T: Add<T, Output = T>,
{
    type Output = F<T>;
    fn add(self, other: F<T>) -> F<T> {
        F(&self.0 + other.0)
    }
}

struct Vector2<T> {
    x: T,
    y: T,
}
struct Matrix2<T> {
    v00: T,
    v01: T,
    v10: T,
    v11: T,
}

#[auto_ops(ref_ref, ref_val, val_ref, val_val)]
impl<T> Mul<&Vector2<T>> for &Matrix2<T>
where
    T: Add<Output = T>,
    for<'x> &'x T: Mul<Output = T>,
{
    type Output = Vector2<T>;
    fn mul(self, other: &Vector2<T>) -> Self::Output {
        Vector2 {
            x: &self.v00 * &other.x + &self.v01 * &other.y,
            y: &self.v10 * &other.x + &self.v11 * &other.y,
        }
    }
}

#[auto_ops(ref_ref, ref_val, val_ref, val_val)]
impl<M> Mul<C<M>> for &A<M>
where
    for<'x> &'x M: Mul<Output = M>,
{
    type Output = D<M>;
    fn mul(self, other: C<M>) -> Self::Output {
        D(&self.0 * &other.0)
    }
}

fn main() {}
