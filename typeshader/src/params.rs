use std::marker::PhantomData;

use crate::types::Type;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Binding(pub(crate) usize);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Location(pub(crate) usize);

impl From<usize> for Binding {
    fn from(n: usize) -> Binding {
        Binding(n)
    }
}

impl From<usize> for Location {
    fn from(n: usize) -> Location {
        Location(n)
    }
}

// skip this binding/location
pub struct S;

pub trait Listable: Sized {
    const COUNT: usize;
    fn get(&self) -> Option<&dyn Type>;
}

impl Listable for S {
    const COUNT: usize = 0;
    fn get(&self) -> Option<&dyn Type> {
        None
    }
}

impl<T: Type> Listable for T {
    const COUNT: usize = 1;
    fn get(&self) -> Option<&dyn Type> {
        Some(self)
    }
}

pub struct Iter<'a, K: From<usize>, T: List> {
    index: usize,
    list: &'a T,
    _type: PhantomData<K>,
}

impl<'a, K: From<usize>, T: List> Iter<'a, K, T> {
    fn new(list: &'a T) -> Self {
        Self {
            list,
            index: 0,
            _type: PhantomData,
        }
    }
}

impl<'a, K: From<usize>, T: List> Iterator for Iter<'a, K, T> {
    type Item = (K, &'a dyn Type);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let i = self.index;
            self.index += 1;
            // reached the end
            if i == T::LEN {
                return None;
            }

            if let Some(v) = self.list.get(i) {
                return Some((i.into(), v));
            }
        }
    }
}

macro_rules! impl_trait {
    ($trait: ident, $typ: ty) => {
        pub trait $trait: List + Sized {
            fn iter<'a>(&'a self) -> Iter<'a, $typ, Self> {
                Iter::new(self)
            }
        }

        impl<T: List> $trait for T {}
    };
}

impl_trait!(Uniforms, Binding);
impl_trait!(Inputs, Location);
impl_trait!(Outputs, Location);

pub trait List: Sized {
    // total ex. skips
    const COUNT: usize;
    // including skips
    const LEN: usize;
    fn get(&self, index: usize) -> Option<&dyn Type>;
}

impl List for () {
    const COUNT: usize = 0;
    const LEN: usize = 0;
    fn get(&self, _: usize) -> Option<&dyn Type> {
        None
    }
}

macro_rules! one {
    ($_: ident) => {
        1
    };
}

macro_rules! impl_list {
    ($($t: ident),*) => {
        #[allow(non_snake_case, unused_variables)]
        impl<$($t: Listable, )*> List for ($($t, )*) {
            const COUNT: usize = $($t::COUNT + )* 0;
            const LEN: usize = $(one!($t) + )* 0;
            fn get(&self, index: usize) -> Option<&dyn Type> {
                let ($($t, )*) = self;
                let count = 0;
                $(
                    if index == count { return $t.get() }
                    let count = count + 1;
                )*
                None
            }
        }
    };
}

macro_rules! impl_all {
    ($t: ident) => {
        impl_list!($t);
    };
    ($head: ident, $($tail: ident),+) => {
        impl_all!($($tail),*); // expand subgroups
        impl_list!($head, $($tail),*);
    };
}

#[cfg(not(feature = "64"))]
impl_all! {
    AA, BA, CA, DA,
    AB, BB, CB, DB,
    AC, BC, CC, DC,
    AD, BD, CD, DD
}

#[cfg(feature = "64")]
impl_all! {
    AA, BA, CA, DA, EA, FA, GA, HA,
    AB, BB, CB, DB, EB, FB, GB, HB,
    AC, BC, CC, DC, EC, FC, GC, HC,
    AD, BD, CD, DD, ED, FD, GD, HD,
    AE, BE, CE, DE, EE, FE, GE, HE,
    AF, BF, CF, DF, EF, FF, GF, HF,
    AG, BG, CG, DG, EG, FG, GG, HG,
    AH, BH, CH, DH, EH, FH, GH, HH
}
