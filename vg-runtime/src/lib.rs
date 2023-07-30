use vg_interface::{Request, Response};

#[cfg(test)]
mod test;

pub mod executor;
// pub mod savestate;
// pub mod wasi;

/// Type that can provide proper answer values to game requests
pub trait Provider {
    fn provide(&mut self, request: Request) -> Response;
}
