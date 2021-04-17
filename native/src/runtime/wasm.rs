use super::{super::Engine, Error, Runtime};

use wasmer::{
    imports, Extern, ExternType, Function, Global, Instance, Memory, MemoryType, Module,
    NativeFunc, Store,
};
pub struct Wasm {
    instance: Instance,
    memory: Memory,
    code: Vec<u8>,
}

fn load(code: &[u8], from: Option<(&Memory, &Instance)>) -> Result<Wasm, Error> {
    let store = Store::default();
    let module = Module::new(&store, code)?;

    let memory = if let Some((mem, _)) = from.clone() {
        let new = Memory::new(&store, *mem.ty())?;

        new.grow(mem.size() - new.size())?;
        assert_eq!(new.data_size(), mem.data_size());

        for (new, old) in new.view::<u8>().iter().zip(mem.view().iter()) {
            new.set(old.get())
        }

        assert!(!new.same(mem));

        new
    } else {
        Memory::new(&store, MemoryType::new(32, None, false))?
    };

    let imports = imports! {
        "env" => {
            "print" => Function::new_native(&store, print),
            "memory" => memory.clone(),
        }
    };

    let mut instance = Instance::new(&module, &imports)?;

    if let Some((_, old)) = from {
        for (name, ext) in old.exports.iter() {
            match ext {
                Extern::Global(val) => {
                    let new = if val.ty().mutability.is_mutable() {
                        Global::new_mut(&store, val.get())
                    } else {
                        Global::new(&store, val.get())
                    };

                    instance.exports.insert(name,Extern::Global(new));
                    println!("Set {}", name);
                },
                _ => {}
            }
        }
        // for (name, ext) in instance.exports.iter() {
        //     if let Some(old) = old.exports.get_extern(name) {
        //         match (ext, old) {
        //             (Extern::Global(new), Extern::Global(old)) => {
        //                 if new.ty().mutability.is_mutable() {
        //                     new.set(old.get())?
        //                 }
        //             }
        //             (Extern::Function(_), Extern::Function(_)) => {}
        //             (Extern::Table(new), Extern::Table(old)) => {
        //                 for i in 0..new.size() {
        //                     new.set(i, old.get(i).unwrap())?;
        //                 }
        //             }
        //             (Extern::Memory(_), Extern::Memory(_)) => {}
        //             (new, old) => panic!(
        //                 "The extern signature changed at runtime {:?} -> {:?}",
        //                 old, new
        //             ),
        //         }
        //     }
        // }
    }
    // Register and bootstrap the runtime on WASM side.
    // if from.is_none() {
    //     let func: NativeFunc<(i32, i32), i32> = instance.exports.get_native_function("main")?;
    //     func.call(0, 0)?;
    // }

    for (name, extrn) in instance.exports.iter() {
        println!("Export: {}: {:?}", name, extrn);
    }

    Ok(Wasm {
        instance,
        memory,
        code: code.to_vec(),
    })
}

fn print(s: i32) {
    println!("WASM: {}", s);
}

impl Runtime for Wasm {
    fn load(code: &[u8]) -> Result<Self, Error> {
        load(code, None)
    }

    fn run_tick(&mut self, _engine: &mut Engine) -> Result<(), Error> {
        println!("Tick");

        let func: NativeFunc<(), ()> = self.instance.exports.get_native_function("__vg_tick")?;
        func.call()?;

        Ok(())
    }

    fn serialize(&self) -> Result<Vec<u8>, Error> {
        todo!()
    }

    fn deserialize(_bytes: &[u8]) -> Result<Self, Error> {
        todo!()
    }

    fn duplicate(&self) -> Result<Self, Error> {
        load(&self.code, Some((&self.memory, &self.instance)))
    }
}
