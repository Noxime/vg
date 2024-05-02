/// Represents () type
pub struct Nil;

impl std::ops::FromResidual<Nil> for Nil {
    fn from_residual(_: Nil) -> Self {
        Nil
    }
}

/// A Check is a convenience structure for performing polling-style checks in
/// in the engine
pub enum Check<T = Nil> {
    Pass(T),
    Fail,
}

pub const PASS: Check = Check::Pass(Nil);
pub const FAIL: Check = Check::Fail;

impl<T> std::ops::Try for Check<T> {
    type Output = T;
    type Residual = Nil;

    fn from_output(v: T) -> Self {
        Check::Pass(v)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Check::Pass(v) => std::ops::ControlFlow::Continue(v),
            Check::Fail => std::ops::ControlFlow::Break(Nil),
        }
    }
}

impl<T> std::ops::FromResidual<Nil> for Check<T> {
    fn from_residual(_: Nil) -> Self {
        Check::Fail
    }
}

impl<T> From<Option<T>> for Check<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => Check::Pass(v),
            None => Check::Fail,
        }
    }
}
impl From<bool> for Check<Nil> {
    fn from(value: bool) -> Self {
        match value {
            true => PASS,
            false => FAIL,
        }
    }
}
