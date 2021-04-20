use fluid_let::fluid_let;
use std::rc::Rc;

use super::{super::Engine, Error, Runtime};

use rust_wasm::*;

fluid_let!(static STORE: &'static Store);

pub struct Wasm {
    instance: Rc<ModuleInst>,
    store: Store,
}

impl Runtime for Wasm {
    fn load(code: &[u8]) -> Result<Self, Error> {
        let mut store = init_store();
        let module = decode_module(std::io::Cursor::new(code)).unwrap();

        // let memory = alloc_mem(
        //     &mut store,
        //     &types::Memory {
        //         limits: types::Limits { min: 17, max: None },
        //     },
        // );

        let print_type = types::Func {
            args: vec![types::I32, types::I32],
            result: vec![],
        };
        let print_wrap = |args: &[values::Value], _res: &mut [values::Value]| {
            if let (Some(values::Value::I32(ptr)), Some(values::Value::I32(len))) =
                (args.get(0), args.get(1))
            {
                STORE.get(|store| {
                    let store = store.unwrap();

                    let mut buf = Vec::with_capacity(*len as usize);
                    for i in 0..*len {
                        let byte = read_mem(store, MemAddr::new(0), (ptr + i) as usize).unwrap();
                        buf.push(byte);
                    }

                    let str = std::str::from_utf8(&buf).unwrap();

                    println!("Print ptr: {:#X}, len: {}: {}", ptr, len, str);
                })
            } else {
                eprintln!("Invalid use of env:print(i32, i32): {:?}", args);
            }
            None
        };

        let print = alloc_func(&mut store, &print_type, Rc::new(print_wrap));

        println!("Module imports");
        for (ns, n, val) in module_imports(&module) {
            println!("\t{}: {} => {:?}", ns, n, val);
        }
        println!("Module exports");
        for (name, val) in module_exports(&module) {
            println!("\t{} => {:?}", name, val);
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

        Ok(Wasm { instance, store })
    }

    fn run_tick(&mut self, _engine: &mut Engine) -> Result<(), Error> {
        println!("Tick");

        // let func: NativeFunc<(), ()> = self.instance.exports.get_native_function("__vg_tick")?;
        // func.call()?;

        let func = match get_export(&self.instance, "__vg_tick") {
            Ok(ExternVal::Func(func)) => func,
            e => {
                panic!("Couldn't get __vg_tick: {:?}", e)
            }
        };

        // fuck it, its not like the codebase isnt cursed already
        let mems: &'static Store = unsafe { std::mem::transmute(&self.store) };

        STORE.set(mems, || {
            invoke_func(&mut self.store, func, vec![]).unwrap();
        });

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

        Ok(Wasm { instance, store })
    }
}
