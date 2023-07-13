use vg_interface::*;

use crate::executor::step;

extern "C" {
    /// Perform a Request to the runtime, returning the number of bytes required for deserializing the Response
    fn __vg_request(ptr: i32, len: i32) -> i32;
    /// Read back the latest response from the Runtime. May only be called after __vg_request
    fn __vg_response(ptr: i32);
}

/// Advance the internal future until it stalls
#[no_mangle]
pub extern "C" fn __vg_step() -> WaitReason {
    step()
}

pub fn dispatch(req: Request) -> Response {
    // Send request
    let req = req.serialize_bin();
    let len = unsafe { __vg_request(req.as_ptr() as _, req.len() as _) };

    // Allocate space for response and fetch it
    // TODO: Should this be Box<Pin<[u8]>>?
    let mut buf = vec![0; len as usize];
    unsafe { __vg_response(buf.as_mut_ptr() as _) }

    Response::deserialize_bin(&buf).expect("Runtime gave malformed Response")
}
