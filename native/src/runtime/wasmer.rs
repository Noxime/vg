use std::{
    cell::UnsafeCell,
    convert::TryInto,
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use tracing::{debug, trace};
use vg_types::{Call, DeBin, SerBin};
use wasmer::{
    imports,
    vm::{Memory, MemoryStyle, VMMemoryDefinition},
    BaseTunables, Bytes, CompilerConfig, Cranelift, Features, Function, Instance, MemoryError,
    MemoryType, Module, Pages, Store, Target, Tunables, UniversalEngine, Value, WasmerEnv,
};

use super::{Error, Runtime};

pub struct Wasmer {
    instance: Instance,
    env: Env,
}

#[derive(Clone)]
struct Env {
    memory: Arc<Mutex<Vec<u8>>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Env {
    fn new() -> Env {
        Env {
            memory: Arc::new(Mutex::new(vec![])),
            calls: Arc::new(Mutex::new(vec![])),
        }
    }

    fn memory(&self) -> Arc<Mutex<Vec<u8>>> {
        Arc::clone(&self.memory)
    }
}

impl WasmerEnv for Env {
    fn init_with_instance(&mut self, _instance: &Instance) -> Result<(), wasmer::HostEnvInitError> {
        Ok(())
    }
}

#[derive(loupe::MemoryUsage)]
pub struct DupTunables {
    mem: Arc<Mutex<Vec<u8>>>,
    base: BaseTunables,
}

impl Tunables for DupTunables {
    fn memory_style(&self, _memory: &wasmer::MemoryType) -> wasmer::vm::MemoryStyle {
        println!("memory style");
        //self.0.memory_style(memory)
        wasmer::vm::MemoryStyle::Dynamic {
            offset_guard_size: 0,
        }
    }

    fn table_style(&self, table: &wasmer::TableType) -> wasmer::vm::TableStyle {
        println!("table style");
        self.base.table_style(table)
    }

    fn create_host_memory(
        &self,
        ty: &wasmer::MemoryType,
        style: &wasmer::vm::MemoryStyle,
    ) -> Result<std::sync::Arc<dyn wasmer::vm::Memory>, wasmer::MemoryError> {
        println!("create host mem");
        unsafe {
            Ok(Arc::new(CloneMem::new_inner(
                Arc::clone(&self.mem),
                ty,
                style,
                None,
            )))
        }
    }

    unsafe fn create_vm_memory(
        &self,
        ty: &wasmer::MemoryType,
        style: &wasmer::vm::MemoryStyle,
        vm_definition_location: std::ptr::NonNull<wasmer::vm::VMMemoryDefinition>,
    ) -> Result<std::sync::Arc<dyn wasmer::vm::Memory>, wasmer::MemoryError> {
        println!("create vm mem");
        Ok(Arc::new(CloneMem::new_inner(
            Arc::clone(&self.mem),
            ty,
            style,
            Some(vm_definition_location),
        )))
    }

    fn create_host_table(
        &self,
        ty: &wasmer::TableType,
        style: &wasmer::vm::TableStyle,
    ) -> Result<std::sync::Arc<dyn wasmer::vm::Table>, String> {
        println!("create host table");
        self.base.create_host_table(ty, style)
    }

    unsafe fn create_vm_table(
        &self,
        ty: &wasmer::TableType,
        style: &wasmer::vm::TableStyle,
        vm_definition_location: std::ptr::NonNull<wasmer::vm::VMTableDefinition>,
    ) -> Result<std::sync::Arc<dyn wasmer::vm::Table>, String> {
        println!("create vm table");
        self.base.create_vm_table(ty, style, vm_definition_location)
    }
}

#[derive(Debug, loupe::MemoryUsage)]
struct CloneMem {
    memory: Arc<Mutex<Vec<u8>>>,
    ty: MemoryType,
    style: MemoryStyle,
    vm_memory_definition: VMMemoryDefinitionOwnership,
}

unsafe impl Send for CloneMem {}
unsafe impl Sync for CloneMem {}

#[derive(Debug, loupe::MemoryUsage)]
enum VMMemoryDefinitionOwnership {
    /// The `VMMemoryDefinition` is owned by the `Instance` and we should use
    /// its memory. This is how a local memory that's exported should be stored.
    VMOwned(NonNull<VMMemoryDefinition>),
    /// The `VMMemoryDefinition` is owned by the host and we should manage its
    /// memory. This is how an imported memory that doesn't come from another
    /// Wasm module should be stored.
    HostOwned(Box<UnsafeCell<VMMemoryDefinition>>),
}

impl CloneMem {
    unsafe fn new_inner(
        mem: Arc<Mutex<Vec<u8>>>,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm: Option<NonNull<VMMemoryDefinition>>,
    ) -> Self {
        let mut memory = mem.lock().unwrap();
        if memory.is_empty() {
            memory.resize(ty.minimum.bytes().0, 0);
        } else {
            panic!()
        }

        let base_ptr = memory.as_mut_ptr();
        let mem_length = memory.len() as u32;

        debug!(
            "Created {} bytes of memory, vm owned: {:?}",
            memory.len(),
            vm.is_some()
        );

        drop(memory);

        Self {
            memory: mem,
            style: style.clone(),
            ty: *ty,
            vm_memory_definition: if let Some(mem_loc) = vm {
                {
                    let mut ptr = mem_loc;
                    let md = ptr.as_mut();
                    md.base = base_ptr;
                    md.current_length = mem_length;
                }
                VMMemoryDefinitionOwnership::VMOwned(mem_loc)
            } else {
                VMMemoryDefinitionOwnership::HostOwned(Box::new(UnsafeCell::new(
                    VMMemoryDefinition {
                        base: base_ptr,
                        current_length: mem_length,
                    },
                )))
            },
        }
    }

    unsafe fn get_vm_memory_definition(&self) -> NonNull<VMMemoryDefinition> {
        match &self.vm_memory_definition {
            VMMemoryDefinitionOwnership::VMOwned(ptr) => *ptr,
            VMMemoryDefinitionOwnership::HostOwned(boxed_ptr) => {
                NonNull::new_unchecked(boxed_ptr.get())
            }
        }
    }
}

impl Memory for CloneMem {
    /// Returns the type for this memory.
    fn ty(&self) -> MemoryType {
        let minimum = self.size();
        let mut out = self.ty.clone();
        out.minimum = minimum;

        out
    }

    /// Returns the memory style for this memory.
    fn style(&self) -> &MemoryStyle {
        &self.style
    }

    /// Returns the number of allocated wasm pages.
    fn size(&self) -> Pages {
        // TODO: investigate this function for race conditions
        unsafe {
            let md_ptr = self.get_vm_memory_definition();
            let md = md_ptr.as_ref();
            Bytes::from(md.current_length).try_into().unwrap()
        }
    }

    /// Grow memory by the specified amount of wasm pages.
    ///
    /// Returns `None` if memory can't be grown by the specified amount
    /// of wasm pages.
    fn grow(&self, delta: Pages) -> Result<Pages, MemoryError> {
        let mut memory = self.memory.lock().unwrap();
        let len = memory.len();
        memory.resize(len + delta.bytes().0, 0);
        Ok(Bytes::from(memory.len()).try_into().unwrap())
    }

    /// Return a `VMMemoryDefinition` for exposing the memory to compiled wasm code.
    fn vmmemory(&self) -> NonNull<VMMemoryDefinition> {
        debug!("Accessing vm memory");
        let _guard = self.memory.lock().unwrap();
        unsafe { self.get_vm_memory_definition() }
    }
}

impl Runtime for Wasmer {
    const NAME: &'static str = "wasm-jit";

    fn load(code: &[u8]) -> Result<Self, super::Error> {
        let env = Env::new();
        let target = Target::default();
        let tunables = DupTunables {
            mem: env.memory(),
            base: BaseTunables::for_target(&target),
        };
        let engine = UniversalEngine::new(
            Box::new(Cranelift::new()).compiler(),
            target,
            Features::default(),
        );

        let store = Store::new_with_tunables(&engine, tunables);
        let module = Module::new(&store, &code)?;

        let imports = imports! {
            "env" => {
                "call" => Function::new_native_with_env(&store, env.clone(), call),
            },
            "wasi_snapshot_preview1" => {
                "fd_write" => Function::new_native(&store, fd_write),
                "random_get" => Function::new_native(&store, random_get),
                "proc_exit" => Function::new_native(&store, proc_exit),
                "environ_sizes_get" => Function::new_native_with_env(&store, env.clone(), environ_sizes_get),
                "environ_get" => Function::new_native(&store, environ_get),
            }
        };
        let instance = Instance::new(&module, &imports)?;

        let main = instance.exports.get_function("main").unwrap();
        let _ = main.call(&[Value::I32(0), Value::I32(0)]).unwrap();

        Ok(Wasmer { instance, env })
    }

    fn run_tick(&mut self) -> Result<Vec<vg_types::Call>, super::Error> {
        puffin::profile_function!();

        let tick = self.instance.exports.get_function("__vg_tick").unwrap();
        let _ = tick.call(&[]).unwrap();

        let calls = self.env.calls.lock().unwrap().split_off(0);

        Ok(calls)
    }

    fn send(&mut self, value: vg_types::Response) {
        puffin::profile_function!();
        let bytes = value.serialize_bin();

        let alloc = self.instance.exports.get_function("__vg_allocate").unwrap();
        let ptr = alloc.call(&[Value::I64(bytes.len() as i64)]).unwrap()[0].unwrap_i64() as usize;

        let mut memory = self.env.memory.lock().unwrap();

        for (i, b) in bytes.iter().enumerate() {
            memory[ptr + i] = *b;
        }
    }

    fn serialize(&self) -> Result<Vec<u8>, super::Error> {
        todo!()
    }

    fn deserialize(_: &[u8]) -> Result<Self, super::Error> {
        todo!()
    }

    fn duplicate(&self) -> Result<Self, Error> {
        puffin::profile_function!();

        Ok(Self {
            instance: self.instance.clone(),
            env: self.env.clone(),
        })
    }
}

fn call(env: &Env, ptr: u64, len: u64) {
    puffin::profile_function!();

    trace!("Call: {:p}, {:x}", ptr as *const u8, len);
    let memory = env.memory.lock().unwrap();

    let bytes = &memory[ptr as usize..][..len as usize];
    let call = Call::deserialize_bin(bytes).unwrap();
    trace!("Runtime got call: {:#?}", call);

    env.calls.lock().unwrap().push(call);
}

fn fd_write(a: i32, b: i32, c: i32, d: i32) -> i32 {
    todo!()
}

fn random_get(a: i32, b: i32) -> i32 {
    0
}

fn proc_exit(a: i32) {
    todo!()
}

fn environ_sizes_get(env: &Env, count: i32, buf_size: i32) -> i32 {
    let mut memory = env.memory.lock().unwrap();

    // tell the vm that we have no environment vars
    for i in 0..4 {
        memory[count as usize + i] = 0;
    }

    for i in 0..4 {
        memory[buf_size as usize + i] = 0;
    }

    0
}

fn environ_get(a: i32, b: i32) -> i32 {
    todo!()
}
