use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc, Mutex,
    },
    time::Duration,
};

const PAGE_SIZE: usize = u16::MAX as _;

// use serde::{Deserialize, Serialize};
use serde::{Serialize, Deserialize};
use tracing::{trace};
use vg_types::{DeBin, SerBin};
use wasmtime::{
    Caller, Config, Engine, Instance, LinearMemory, Linker, MemoryCreator, MemoryType,
    Module, Store, Val,
};

use super::{Error, Runtime};

// Intermediate representation of a wasm runtime, intended for serialization
#[derive(Serialize, Deserialize, Clone)]
struct Intermediate {
    memories: Vec<Vec<u8>>,
    code: Vec<u8>,
}

// This handles serialization and cloning of wasm memories
struct MemoryManager {
    memories: Mutex<Vec<ArcMovingMemory>>,
}

impl MemoryManager {
    fn new() -> Self {
        MemoryManager {
            memories: Mutex::new(vec![]),
        }
    }

    fn as_trait(self: &Arc<MemoryManager>) -> Arc<dyn MemoryCreator> {
        let cloned = Arc::clone(self);
        cloned
    }
}

unsafe impl MemoryCreator for MemoryManager {
    fn new_memory(
        &self,
        ty: MemoryType,
        reserved_size_in_bytes: Option<u64>,
        _guard_size_in_bytes: u64,
    ) -> Result<Box<dyn LinearMemory>, String> {
        trace!(
            "New memory created: min: {}, reserved: {:?}",
            ty.limits().min(),
            reserved_size_in_bytes
        );

        assert!(
            reserved_size_in_bytes.is_none(),
            "We should never create static memory"
        );

        let len = ty.limits().min() as usize * PAGE_SIZE;

        let mem = ArcMovingMemory(Arc::new(MovingMemory {
            data: vec![0; len],
            moveable: AtomicBool::new(true),
        }));

        let mut lock = self.memories.lock().unwrap();
        lock.push(mem.clone());

        Ok(Box::new(mem))
    }
}

#[derive(Clone)]
struct ArcMovingMemory(Arc<MovingMemory>);

impl ArcMovingMemory {
    fn is_movable(&self) -> bool {
        self.0.moveable.load(Relaxed)
    }

    fn clear_movable(&self) {
        self.0.moveable.store(false, Relaxed);
    }

    // God damn unsafe, but deal with it
    fn set_data(&self, b: &[u8]) {
        let data = &self.0.data as *const Vec<u8> as *mut Vec<u8>;
        let data = unsafe { data.as_mut().unwrap() };
        data.copy_from_slice(b);
    }
}

struct MovingMemory {
    data: Vec<u8>,
    moveable: AtomicBool,
}

unsafe impl LinearMemory for ArcMovingMemory {
    fn size(&self) -> u32 {
        (self.0.data.len() / PAGE_SIZE) as u32
    }

    fn maximum(&self) -> Option<u32> {
        None
    }

    fn grow(&mut self, delta: u32) -> Option<u32> {
        // Client signals that now is a good time to serialize eg. End of frame
        if delta as usize == vg_types::MOVE_TRIGGER_MAGIC {
            trace!("Memory move trigger detected");
            self.0.moveable.store(true, Relaxed);
        }

        // We don't actually need to give any memory
        None
    }

    fn as_ptr(&self) -> *mut u8 {
        trace!("Memory pointer requested");
        self.0.data.as_ptr() as *mut u8 // TODO: Is this okay?
    }
}

struct Context {
    calls: Vec<vg_types::Call>,
}

pub struct WasmtimeRT {
    instance: Instance,
    // TODO: We NEED to reuse this module instead of recompiling on each deserialize
    module: Module,
    store: Store<Context>,
    mem_manager: Arc<MemoryManager>,
}

impl WasmtimeRT {
    fn new_engine() -> Result<(Engine, Arc<MemoryManager>), Error> {
        puffin::profile_function!();
        let mem_manager = Arc::new(MemoryManager::new());

        let config = Config::new()
            .cranelift_nan_canonicalization(true) // Make floats determenistic
            .cranelift_opt_level(wasmtime::OptLevel::Speed) // Speedy
            .static_memory_maximum_size(0) // Force using dynamic memory which may change location
            .with_host_memory(mem_manager.as_trait())
            .clone();
        Ok((Engine::new(&config)?, mem_manager))
    }

    fn new(module: Module, engine: Engine, mem_manager: Arc<MemoryManager>) -> Result<Self, Error> {
        puffin::profile_function!();

        let (instance, store) = {
            puffin::profile_scope!("link_and_instantiate");
            // Define WASM imports
            let mut linker = Linker::new(&engine);

            // VG API
            linker.func_wrap("vg", "call", vg_call)?;

            // WASI API
            linker.func_wrap("wasi_snapshot_preview1", "fd_write", wasi_fd_write)?;
            linker.func_wrap("wasi_snapshot_preview1", "random_get", wasi_random_get)?;
            linker.func_wrap("wasi_snapshot_preview1", "proc_exit", wasi_proc_exit)?;
            linker.func_wrap(
                "wasi_snapshot_preview1",
                "environ_sizes_get",
                wasi_environ_sizes_get,
            )?;
            linker.func_wrap("wasi_snapshot_preview1", "environ_get", wasi_environ_get)?;
            linker.func_wrap("wasi_snapshot_preview1", "sched_yield", wasi_sched_yield)?;

            let mut store = Store::new(&engine, Context { calls: vec![] });
            (linker.instantiate(&mut store, &module)?, store)
        };

        Ok(Self {
            instance,
            module,
            store,
            mem_manager,
        })
    }

    fn to_intermediate(&mut self) -> Result<Intermediate, Error> {
        puffin::profile_function!();
        // This triggers memory growth which invalidates the memory pointer, meaning
        // we can safely ser/de the memory
        let move_fn = self
            .instance
            .get_func(&mut self.store, "__vg_move")
            .unwrap();
        move_fn.call(&mut self.store, &[]).unwrap();

        // Check if _all_ memories for this client were properly moved
        let memories = self.mem_manager.memories.lock().unwrap();
        assert!(
            memories.iter().all(|m| m.is_movable()),
            "Cannot serialize safely when all memories are not moveable"
        );
        memories.iter().for_each(ArcMovingMemory::clear_movable);

        Ok(Intermediate {
            memories: memories.iter().map(|m| m.0.data.clone()).collect(),
            code: self.module.serialize()?,
        })
    }

    fn from_intermediate(s: Intermediate) -> Result<Self, Error> {
        puffin::profile_function!();
        let (engine, mem_manager) = Self::new_engine()?;

        // Deserialize already compiled WASM
        let module = unsafe { Module::deserialize(&engine, s.code)? };

        // Link and instantiate
        let mut this = Self::new(module, engine, mem_manager)?;

        {
            puffin::profile_scope!("deserialize_write_memory");
            // Write the deserialized state to memories
            let mut memories = this.mem_manager.memories.lock().unwrap();
            for (dst, src) in memories.iter_mut().zip(s.memories.as_slice()) {
                dst.set_data(src.as_slice());
            }
            drop(memories);
        }

        {
            puffin::profile_scope!("deserialize_trigger_move");
            let move_fn = this
                .instance
                .get_func(&mut this.store, "__vg_move")
                .unwrap();
            move_fn.call(&mut this.store, &[]).unwrap();
        }

        Ok(this)
    }
}

impl Runtime for WasmtimeRT {
    const NAME: &'static str = "wasmtime";

    fn load(code: &[u8]) -> Result<Self, Error> {
        puffin::profile_function!();
        // Create a config and new memory manager
        let (engine, mem_manager) = Self::new_engine()?;

        // Compile the WASM
        let module = Module::from_binary(&engine, code)?;

        // Link and instantiate
        let mut this = Self::new(module, engine, mem_manager)?;

        // Initialize the client, ready for __vg_tick
        let main_fn = this
            .instance
            .get_func(&mut this.store, "main")
            .expect("No main fn in client");
        main_fn.call(&mut this.store, &[Val::I32(0), Val::I32(0)])?;

        Ok(this)
    }

    fn run_tick(&mut self, dt: Duration) -> Result<Vec<vg_types::Call>, Error> {
        puffin::profile_function!();

        let tick_fn = self
            .instance
            .get_func(&mut self.store, "__vg_tick")
            .unwrap();

        tick_fn
            .call(&mut self.store, &[dt.as_secs_f64().into()])
            .unwrap();

        let calls = self.store.data_mut().calls.split_off(0);

        Ok(calls)
    }

    fn send(&mut self, value: vg_types::Response) {
        puffin::profile_function!();
        let bytes = value.serialize_bin();

        // Allocate space in client for the response struct
        let alloc_fn = self
            .instance
            .get_func(&mut self.store, "__vg_allocate")
            .unwrap();
        let ptr = alloc_fn
            .call(&mut self.store, &[Val::I64(bytes.len() as _)])
            .unwrap()[0]
            .unwrap_i64();

        let mem = self.instance.get_memory(&mut self.store, "memory").unwrap();
        mem.write(&mut self.store, ptr as _, &bytes).unwrap();
    }

    fn serialize(&mut self) -> Result<Vec<u8>, Error> {
        puffin::profile_function!();
        trace!("Serializing WASM state");

        let s = self.to_intermediate()?;

        {
            puffin::profile_scope!("archive");
            bincode::serialize(&s).map_err(Into::into)
        }
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
        puffin::profile_function!();

        let s = {
            puffin::profile_scope!("unarchive");
            bincode::deserialize(bytes)?
        };

        Self::from_intermediate(s)
    }

    fn duplicate(&mut self) -> Result<Self, Error> {
        puffin::profile_function!();
        Self::from_intermediate(self.to_intermediate()?)
    }
}

fn vg_call(mut caller: Caller<Context>, ptr: u64, len: u64) {
    puffin::profile_function!();
    // let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
    trace!("vg_call: ptr: {:#p}, len: {:#X}", ptr as *const u8, len);
    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();

    let mut buf = vec![0u8; len as _];
    mem.read(&mut caller, ptr as _, &mut buf).unwrap();

    let call = vg_types::Call::deserialize_bin(&buf).unwrap();
    trace!("Host got call {:?}", call);

    caller.data_mut().calls.push(call);
}

fn wasi_sched_yield() -> i32 {
    // Do nothing for now
    0
}

fn wasi_fd_write(_caller: Caller<Context>, a: i32, b: i32, c: i32, d: i32) -> i32 {
    trace!("wasi_fd_write: a: {}, b: {}, c: {}, d: {}", a, b, c, d);

    todo!()
}

fn wasi_random_get(_caller: Caller<Context>, a: i32, b: i32) -> i32 {
    trace!("wasi_random_get: a: {}, b: {}", a, b);

    0
}

fn wasi_proc_exit(_caller: Caller<Context>, code: i32) {
    trace!("wasi_proc_exit: code: {}", code);

    todo!()
}

fn wasi_environ_sizes_get(mut caller: Caller<Context>, count: i32, buf_size: i32) -> i32 {
    trace!(
        "wasi_environ_sizes_get: count: {:#p}, buf_size: {:#p}",
        count as *const u32,
        buf_size as *const u32
    );

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    memory.write(&mut caller, count as _, &[0; 4]).unwrap();
    memory.write(&mut caller, buf_size as _, &[0; 4]).unwrap();

    0
}

fn wasi_environ_get(_caller: Caller<Context>, a: i32, b: i32) -> i32 {
    trace!("wasi_environ_get: a: {}, b: {}", a, b);

    todo!()
}
