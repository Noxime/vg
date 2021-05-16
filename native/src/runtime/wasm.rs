use std::{cell::Cell, rc::Rc, time::Duration};
use tracing::*;

use super::{super::Engine, Error, Runtime};

use rust_wasm::*;
use vg_types::*;

pub struct Wasm {
    instance: Rc<ModuleInst>,
    store: Store,
    engine: Rc<Cell<*mut Engine>>,
}

impl Runtime for Wasm {
    fn load(code: &[u8]) -> Result<Self, Error> {
        let mut store = init_store();
        let module = decode_module(std::io::Cursor::new(code)).unwrap();

        let engine: Rc<Cell<*mut Engine>> = Rc::new(Cell::new(std::ptr::null_mut()));
        let engine_copy = Rc::clone(&engine);

        let call_type = types::Func {
            args: vec![types::I64, types::I64],
            result: vec![],
        };
        let call_wrap =
            move |mem: &mut [u8], args: &[values::Value], _res: &mut [values::Value]| {
                if let (Some(values::Value::I64(ptr)), Some(values::Value::I64(len))) =
                    (args.get(0), args.get(1))
                {
                    let bytes = &mem[*ptr as usize..][..*len as usize];
                    let call = Call::deserialize_bin(bytes).unwrap();

                    if let Some(engine) = unsafe { engine_copy.get().as_mut() } {
                        engine.call(call);
                    } else {
                        panic!("Host called but no valid engine was not set");
                    }
                } else {
                    eprintln!("Invalid use of env:call(i64, i64): {:?}", args);
                }
                None
            };

        let print = alloc_func(&mut store, &call_type, Rc::new(call_wrap));

        debug!("Module imports");
        for (ns, n, val) in module_imports(&module) {
            debug!("  {}: {} => {:?}", ns, n, val);
        }
        debug!("Module exports");
        for (name, val) in module_exports(&module) {
            debug!("  {} => {:?}", name, val);
        }

        let instance = instantiate_module(&mut store, module, &[ExternVal::Func(print)]).unwrap();

        let func = match get_export(&instance, "main") {
            Ok(ExternVal::Func(func)) => func,
            e => {
                panic!("Couldn't get main: {:?}", e)
            }
        };

        invoke_func(
            &mut store,
            func,
            vec![values::Value::I32(0), values::Value::I32(0)],
        )
        .unwrap();

        Ok(Wasm {
            instance,
            store,
            engine,
        })
    }

    fn run_tick(&mut self, engine: &mut Engine) -> Result<(), Error> {
        puffin::profile_function!();

        let func = match get_export(&self.instance, "__vg_tick") {
            Ok(ExternVal::Func(func)) => func,
            e => {
                panic!("Couldn't get __vg_tick: {:?}", e)
            }
        };

        self.engine.set(engine as *mut _);
        invoke_func(&mut self.store, func, vec![]).unwrap();
        self.engine.set(std::ptr::null_mut());

        Ok(())
    }

    fn send(&mut self, value: vg_types::Response) {
        puffin::profile_function!();

        trace!("Sending {:#?} to runtime", value);
        let bytes = value.serialize_bin();

        let func = match get_export(&self.instance, "__vg_allocate") {
            Ok(ExternVal::Func(func)) => func,
            e => {
                panic!("Couldn't get __vg_allocate: {:?}", e)
            }
        };

        let len = bytes.len();

        let ptr =
            invoke_func(&mut self.store, func, vec![values::Value::I64(len as u64)]).unwrap()[0];

        let ptr = if let values::Value::I64(ptr) = ptr {
            ptr as usize
        } else {
            panic!()
        };

        let mem = match get_export(&self.instance, "memory") {
            Ok(ExternVal::Memory(mem)) => mem,
            e => {
                panic!("Couldn't get __vg_allocate: {:?}", e)
            }
        };

        for (off, byte) in bytes.iter().enumerate() {
            assert!(rust_wasm::write_mem(&mut self.store, mem, ptr + off, *byte).is_none());
        }

        let func = match get_export(&self.instance, "__vg_consume") {
            Ok(ExternVal::Func(func)) => func,
            e => {
                panic!("Couldn't get __vg_consume: {:?}", e)
            }
        };

        invoke_func(&mut self.store, func, vec![]).unwrap();
    }

    fn serialize(&self) -> Result<Vec<u8>, Error> {
        todo!()
    }

    fn deserialize(_bytes: &[u8]) -> Result<Self, Error> {
        todo!()
    }

    fn duplicate(&self) -> Result<Self, Error> {
        puffin::profile_function!();

        let instance = Rc::new(ModuleInst::clone(&self.instance));
        let store = self.store.clone();
        // let engine = Rc::new(Cell::new(std::ptr::null_mut()));
        let engine = Rc::clone(&self.engine);

        Ok(Wasm {
            instance,
            store,
            engine,
        })
    }
}
