use std::{sync::Arc, path::Path};

use anyhow::{anyhow, Result};
use tracing::trace;
use vg_asset::{AssetKind, BinAsset, Assets, Asset};
use vg_interface::{DeBin, Request, Response, SerBin, WaitReason};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use crate::{executor::{GlobalData, MemoryData, TableData, PAGE_SIZE}, Provider};

pub struct WasmtimeInner {
    // TODO: His ass is NOT rollbackable!
    wasi: WasiCtx,
    response: Vec<u8>,
    func: Box<dyn FnMut(Request) -> Response>,
}
pub struct WasmtimeModule {
    engine: Engine,
    module: Module,
}

impl WasmtimeModule {
    pub fn instantiate(
        self: &Arc<Self>,
    ) -> Result<WasmtimeInstance> {
        let mut store = Store::new(
            &self.engine,
            WasmtimeInner {
                wasi: WasiCtxBuilder::new().inherit_stdout().build(),
                response: vec![],
                func: Box::new(|_| unreachable!()),
            },
        );

        // Start out instance with WASI imports
        let mut linker = Linker::<WasmtimeInner>::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |inner| &mut inner.wasi)?;

        linker.func_wrap(
            "env",
            "__vg_request",
            |mut caller: Caller<'_, WasmtimeInner>, ptr: i32, len: i32| -> Result<i32> {
                let mem = caller
                    .get_export("memory")
                    .ok_or(anyhow!("No memory on module"))?
                    .into_memory()
                    .ok_or(anyhow!("Memory 'memory' is not memory"))?;

                // Deserialize request from instance memory
                let bytes = &mem.data(&caller)[ptr as usize..][..len as usize];
                let request = Request::deserialize_bin(bytes)?;

                // Call to engine implementation
                let func = &mut caller.data_mut().func;
                let response = (func)(request);

                // Store response for later fetch
                caller.data_mut().response = response.serialize_bin();
                Ok(caller.data().response.len() as i32)
            },
        )?;

        linker.func_wrap(
            "env",
            "__vg_response",
            |mut caller: Caller<'_, WasmtimeInner>, ptr: i32| -> Result<()> {
                let mem = caller
                    .get_export("memory")
                    .ok_or(anyhow!("No memory on module"))?
                    .into_memory()
                    .ok_or(anyhow!("Memory 'memory' is not memory"))?;

                // TODO: Unnecessary clone?
                let response = caller.data().response.clone();

                // Write response
                mem.write(&mut caller, ptr as usize, &response)?;
                Ok(())
            },
        )?;

        let instance = linker.instantiate(&mut store, &self.module)?;

        // Call default export (either "" or "_start")
        instance
            .get_typed_func(&mut store, "")
            .or_else(|_| instance.get_typed_func(&mut store, "_start"))?
            .call(&mut store, ())?;

        Ok(WasmtimeInstance {
            // module: Arc::clone(self),
            store,
            instance,
        })
    }
}

pub struct WasmtimeInstance {
    // module: Arc<WasmtimeModule>,
    store: Store<WasmtimeInner>,
    instance: Instance,
}

impl AssetKind for WasmtimeInstance {
    /// Bytecode source
    type Data = Asset<BinAsset>;

    fn new(assets: &Arc<Assets>, path: &Path) -> Self::Data {
        assets.get(path)
    }

    fn produce(data: &mut Self::Data) -> Option<Self> {
        let bin = data.get()?;
        super::Instance::new(&bin.bytes, true).ok()
    }
}

impl super::Instance for WasmtimeInstance {
    fn new(
        wasm: &[u8],
        debug: bool,
    ) -> Result<WasmtimeInstance> {
        tracing::debug!(len = wasm.len(), "Creating new Wasmtime instance");

        let engine = Engine::new(
            &Config::new()
                .cache_config_load_default()?
                .debug_info(debug)
                .wasm_backtrace(debug)
                .wasm_backtrace_details(
                    debug
                        .then_some(WasmBacktraceDetails::Enable)
                        .unwrap_or(WasmBacktraceDetails::Disable),
                ),
        )?;

        let module = Module::new(&engine, wasm)?;
        let module = Arc::new(WasmtimeModule { engine, module });

        module.instantiate()
    }

    fn step<T: Provider>(&mut self, provider: &mut T) -> WaitReason {
        let ptr = provider as *mut T as *mut ();

        self.store.data_mut().func = Box::new(move |req| {
            let provider = ptr as *mut T;
            // Safety: Kind of terrible, but uhh... I pinky promise to not call __vg_step outside of this function
            unsafe { (*provider).provide(req) }
        });

        let func = self
            .instance
            .get_func(&mut self.store, "__vg_step")
            .unwrap();

        let mut ret = [Val::I32(0)];
        func.call(&mut self.store, &[], &mut ret).unwrap();

        WaitReason::from_raw(ret[0].unwrap_i32())
    }

    fn get_data(&mut self) -> super::InstanceData {
        trace!("Serializing instance data");

        let exports = self.instance.exports(&mut self.store);
        let mut globals = vec![];
        let mut tables = vec![];
        let mut memories = vec![];

        for export in exports {
            let name = export.name().to_string();

            match export.into_extern() {
                Extern::Func(_) => (), // Functions are not data (functional programmers weep)
                Extern::Global(global) => globals.push((name, global)),
                Extern::Table(table) => tables.push((name, table)),
                Extern::Memory(memory) => memories.push((name, memory)),
                Extern::SharedMemory(_) => todo!("SharedMemory"),
            }
        }

        super::InstanceData {
            memories: memories
                .into_iter()
                .map(|(n, m)| (n, MemoryData::new(m.data(&self.store))))
                .collect(),
            globals: globals
                .into_iter()
                .filter_map(|(n, g)| {
                    (g.ty(&self.store).mutability() == Mutability::Var).then_some((
                        n,
                        match g.get(&mut self.store) {
                            Val::I32(v) => GlobalData::I32(v),
                            Val::I64(v) => GlobalData::I64(v),
                            Val::F32(v) => GlobalData::F32(v),
                            Val::F64(v) => GlobalData::F64(v),
                            Val::V128(_) => todo!(),
                            Val::FuncRef(_) => todo!(),
                            Val::ExternRef(_) => todo!(),
                        },
                    ))
                })
                .collect(),
            tables: tables
                .into_iter()
                .map(|(n, t)| {
                    (
                        n,
                        match t.ty(&self.store).element() {
                            ValType::I32 => TableData::I32(
                                (0..t.size(&self.store))
                                    .map(|i| t.get(&mut self.store, i).unwrap().unwrap_i32())
                                    .collect(),
                            ),
                            ValType::I64 => TableData::I64(
                                (0..t.size(&self.store))
                                    .map(|i| t.get(&mut self.store, i).unwrap().unwrap_i64())
                                    .collect(),
                            ),
                            ValType::F32 => TableData::F32(
                                (0..t.size(&self.store))
                                    .map(|i| {
                                        t.get(&mut self.store, i).unwrap().unwrap_f32().to_bits()
                                    })
                                    .collect(),
                            ),
                            ValType::F64 => TableData::F64(
                                (0..t.size(&self.store))
                                    .map(|i| {
                                        t.get(&mut self.store, i).unwrap().unwrap_f64().to_bits()
                                    })
                                    .collect(),
                            ),
                            ValType::V128 => todo!(),
                            ValType::FuncRef => todo!(),
                            ValType::ExternRef => todo!(),
                        },
                    )
                })
                .collect(),
        }
    }

    fn set_data(&mut self, data: &super::InstanceData) {
        trace!("Deserializing instance data");

        for (name, data) in &data.memories {
            let memory = self
                .instance
                .get_memory(&mut self.store, &name)
                .expect("Tried to set unknown memory");

            // New data might be larger than what we have, grow to match
            let delta = data.pages().saturating_sub(memory.size(&self.store));
            memory
                .grow(&mut self.store, delta)
                .expect("Failed to grow memory on set");

            // Copy pages
            memory
                .data_mut(&mut self.store)
                .chunks_mut(PAGE_SIZE)
                .zip(&data.pages)
                .for_each(|(page, data)| page.copy_from_slice(&data.bytes));
        }

        for (name, data) in &data.globals {
            let global = self
                .instance
                .get_global(&mut self.store, &name)
                .expect("Tried to set unknown global");

            global.set(&mut self.store, data.as_val()).unwrap();
        }

        for (name, data) in &data.tables {
            let table = self
                .instance
                .get_table(&mut self.store, &name)
                .expect("Tried to set unknown table");

            // New data might be larger than what we have, grow to match
            let delta = data.len().saturating_sub(table.size(&self.store));
            table
                .grow(&mut self.store, delta, data.default_val())
                .unwrap();

            for (i, v) in data.iter_val().enumerate() {
                table.set(&mut self.store, i as u32, v).unwrap();
            }
        }
    }
}

impl GlobalData {
    fn as_val(&self) -> Val {
        match self {
            GlobalData::I32(v) => Val::I32(*v),
            GlobalData::I64(v) => Val::I64(*v),
            GlobalData::F32(v) => Val::F32(*v),
            GlobalData::F64(v) => Val::F64(*v),
        }
    }
}

impl TableData {
    fn default_val(&self) -> Val {
        match self {
            TableData::I32(_) => Val::I32(0),
            TableData::I64(_) => Val::I64(0),
            TableData::F32(_) => Val::F32(0),
            TableData::F64(_) => Val::F64(0),
        }
    }

    fn iter_val(&self) -> Box<dyn Iterator<Item = Val> + '_> {
        match self {
            TableData::I32(v) => Box::new(v.iter().copied().map(Val::I32)),
            TableData::I64(v) => Box::new(v.iter().copied().map(Val::I64)),
            TableData::F32(v) => Box::new(v.iter().copied().map(Val::F32)),
            TableData::F64(v) => Box::new(v.iter().copied().map(Val::F64)),
        }
    }
}
