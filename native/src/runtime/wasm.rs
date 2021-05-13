use log::*;
use std::{cell::Cell, rc::Rc};

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
        println!("Module exports");
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

    fn serialize(&self) -> Result<Vec<u8>, Error> {
        todo!()
    }

    fn deserialize(_bytes: &[u8]) -> Result<Self, Error> {
        todo!()
    }

    fn duplicate(&self) -> Result<Self, Error> {
        let instance = Rc::new(ModuleInst::clone(&self.instance));
        let store = self.store.clone();
        let engine = Rc::new(Cell::new(std::ptr::null_mut()));

        Ok(Wasm {
            instance,
            store,
            engine,
        })
    }
}
