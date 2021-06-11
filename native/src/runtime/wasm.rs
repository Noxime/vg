use std::rc::Rc;
use tracing::*;

use super::{Error, Runtime};

use rust_wasm::*;
use vg_types::*;

pub struct Wasm {
    instance: Rc<ModuleInst>,
    store: Store<Vec<Call>>,
}

impl Runtime for Wasm {
    fn load(code: &[u8]) -> Result<Self, Error> {
        let mut store = init_store();
        let module = decode_module(std::io::Cursor::new(code)).unwrap();

        let call_type = types::Func {
            args: vec![types::I64, types::I64],
            result: vec![],
        };
        let call_wrap = move |engine: &mut Vec<Call>,
                              mem: &mut [u8],
                              args: &[values::Value],
                              _res: &mut [values::Value]| {
            let (ptr, len) = if let (Some(values::Value::I64(ptr)), Some(values::Value::I64(len))) =
                (args.get(0), args.get(1))
            {
                (ptr, len)
            } else {
                eprintln!("Invalid use of env:call(i64, i64): {:?}", args);
                return None;
            };

            let bytes = &mem[*ptr as usize..][..*len as usize];
            let call = Call::deserialize_bin(bytes).unwrap();

            engine.push(call);

            // TODO: Pass calls back to engine
            None
        };

        let call = alloc_func(&mut store, &call_type, Rc::new(call_wrap));

        debug!("Module imports");
        for (ns, n, val) in module_imports(&module) {
            debug!("  {}: {} => {:?}", ns, n, val);
        }
        debug!("Module exports");
        for (name, val) in module_exports(&module) {
            debug!("  {} => {:?}", name, val);
        }

        let fd_write = alloc_func(
            &mut store,
            &types::Func {
                args: vec![types::I32, types::I32, types::I32, types::I32],
                result: vec![types::I32],
            },
            Rc::new(|_, _mem, _args, _res| {
                trace!("fd_write: {:?}", _args);
                None
            }),
        );

        let random_get = alloc_func(
            &mut store,
            &types::Func {
                args: vec![types::I32, types::I32],
                result: vec![types::I32],
            },
            Rc::new(|_, _mem, _args, _res| {
                trace!("random_get: {:?}", _args);
                None
            }),
        );

        let proc_exit = alloc_func(
            &mut store,
            &types::Func {
                args: vec![types::I32],
                result: vec![],
            },
            Rc::new(|_, _mem, _args, _res| {
                trace!("proc_exit: {:?}", _args);
                None
            }),
        );

        let environ_sizes_get = alloc_func(
            &mut store,
            &types::Func {
                args: vec![types::I32, types::I32],
                result: vec![types::I32],
            },
            Rc::new(|_, mem, args, _res| {
                trace!("environ_sizes_get: {:?}", args);

                if let [values::Value::I32(count), values::Value::I32(buf_size)] = args {
                    let count = *count as usize;
                    for i in 0..4 {
                        mem[count + i] = 0u8;
                    }
                    let buf_size = *buf_size as usize;
                    for i in 0..4 {
                        mem[buf_size + i] = 0u8;
                    }
                }

                None
            }),
        );

        let environ_get = alloc_func(
            &mut store,
            &types::Func {
                args: vec![types::I32, types::I32],
                result: vec![types::I32],
            },
            Rc::new(|_, _mem, _args, _res| {
                trace!("environ_get: {:?}", _args);
                None
            }),
        );

        let instance = instantiate_module(
            &mut vec![],
            &mut store,
            module,
            &[
                ExternVal::Func(call),
                ExternVal::Func(fd_write),
                ExternVal::Func(random_get),
                ExternVal::Func(proc_exit),
                ExternVal::Func(environ_sizes_get),
                ExternVal::Func(environ_get),
            ],
        )
        .unwrap();

        let func = match get_export(&instance, "main") {
            Ok(ExternVal::Func(func)) => func,
            e => {
                panic!("Couldn't get main: {:?}", e)
            }
        };

        invoke_func(
            &mut vec![],
            &mut store,
            func,
            vec![values::Value::I32(0), values::Value::I32(0)],
        )
        .unwrap();

        Ok(Wasm { instance, store })
    }

    fn run_tick(&mut self) -> Result<Vec<Call>, Error> {
        puffin::profile_function!();

        let func = match get_export(&self.instance, "__vg_tick") {
            Ok(ExternVal::Func(func)) => func,
            e => {
                panic!("Couldn't get __vg_tick: {:?}", e)
            }
        };

        // self.engine.set(Some(engine));
        let mut calls = vec![];
        invoke_func(&mut calls, &mut self.store, func, vec![]).unwrap();
        // self.engine.set(None);

        Ok(calls)
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

        let ptr = invoke_func(
            &mut vec![],
            &mut self.store,
            func,
            vec![values::Value::I64(len as u64)],
        )
        .unwrap()[0];

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

        // let func = match get_export(&self.instance, "__vg_consume") {
        //     Ok(ExternVal::Func(func)) => func,
        //     e => {
        //         panic!("Couldn't get __vg_consume: {:?}", e)
        //     }
        // };

        // invoke_func(&mut self.store, func, vec![]).unwrap();
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

        Ok(Wasm { instance, store })
    }
}
