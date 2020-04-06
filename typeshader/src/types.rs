#![allow(unreachable_code)]
#[allow(non_camel_case_types)]
use crate::List;

pub use half::f16;

pub trait Type: std::any::Any {
    fn to_any(&self) -> AnyType;
}

impl dyn Type {
    /// Is the underlying type T
    pub fn is<T: Type>(&self) -> bool {
        self.type_id() == std::any::TypeId::of::<T>()
    }

    /// Is the underlying type T
    pub fn downcast<T: Type + Copy>(&self) -> Option<T> {
        if self.is::<T>() {
            unsafe {
                Some(*(self as *const dyn Type as *const T))
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum AnyType {
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F16(f16),
    F32(f32),
    F64(f64),
    // all elements must be of same type
    Array(Box<[AnyType]>),
    // members can be any type
    Struct(Box<[AnyType]>),
}

impl<T: Type> From<T> for AnyType {
    fn from(t: T) -> AnyType {
        t.to_any()
    }
}

impl From<&dyn Type> for AnyType {
    fn from(t: &dyn Type) -> AnyType {
        t.to_any()
    }
}

impl AnyType {
    pub fn to_dyn(&self) -> &dyn Type {
        match self {
            AnyType::Bool(v) => v,
            AnyType::U8(v) => v,
            AnyType::I8(v) => v,
            AnyType::U16(v) => v,
            AnyType::I16(v) => v,
            AnyType::U32(v) => v,
            AnyType::I32(v) => v,
            AnyType::U64(v) => v,
            AnyType::I64(v) => v,
            AnyType::F16(v) => v,
            AnyType::F32(v) => v,
            AnyType::F64(v) => v,
            AnyType::Array(_) => todo!(),
            AnyType::Struct(_) => todo!(),
        }
    }
    // Try to convert the `AnyType` back to a real type
    pub fn downcast<T: Type + Copy>(self) -> Option<T> {
        self.to_dyn().downcast()
    }
}

// array
impl<T: Type + Copy, const N: usize> Type for [T; N] {
    fn to_any(&self) -> AnyType {
        AnyType::Array(
            Vec::from(Box::new(*self) as Box<[T]>)
                .into_iter()
                .map(From::from)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }
}

// structure
pub struct Struct<T: List>(pub T);
// huh?
impl<T: 'static + List> Type for Struct<T> {
    fn to_any(&self) -> AnyType {
        todo!()
    }
}

pub type Vector<T, const N: usize> = [T; N];
pub type Matrix<T, const N: usize> = [T; N*N];


impl Type for bool {
    fn to_any(&self) -> AnyType {
        AnyType::Bool(*self)
    }
}
impl Type for u8 {
    fn to_any(&self) -> AnyType {
        AnyType::U8(*self)
    }
}
impl Type for i8 {
    fn to_any(&self) -> AnyType {
        AnyType::I8(*self)
    }
}
impl Type for u16 {
    fn to_any(&self) -> AnyType {
        AnyType::U16(*self)
    }
}
impl Type for i16 {
    fn to_any(&self) -> AnyType {
        AnyType::I16(*self)
    }
}
impl Type for u32 {
    fn to_any(&self) -> AnyType {
        AnyType::U32(*self)
    }
}
impl Type for i32 {
    fn to_any(&self) -> AnyType {
        AnyType::I32(*self)
    }
}
impl Type for u64 {
    fn to_any(&self) -> AnyType {
        AnyType::U64(*self)
    }
}
impl Type for i64 {
    fn to_any(&self) -> AnyType {
        AnyType::I64(*self)
    }
}
impl Type for f16 {
    fn to_any(&self) -> AnyType {
        AnyType::F16(*self)
    }
}
impl Type for f32 {
    fn to_any(&self) -> AnyType {
        AnyType::F32(*self)
    }
}
impl Type for f64 {
    fn to_any(&self) -> AnyType {
        AnyType::F64(*self)
    }
}
