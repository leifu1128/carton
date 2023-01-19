use std::{collections::HashMap, marker::PhantomData};
pub use carton_macros::for_each_carton_type;
use serde::{Serialize, Deserialize};

use super::comms::Comms;

#[derive(Debug, Serialize, Deserialize)]
pub struct RPCRequest {
    pub id: RpcId,

    pub data: RPCRequestData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RPCResponse {
    pub id: RpcId,

    pub data: RPCResponseData,
}


pub(crate) type RpcId = u64;

// Used in multiplexer
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct StreamID(pub(crate) u64);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct FsToken(pub(crate) StreamID);


// Individual channels/streams to avoid head of line blocking
#[allow(non_camel_case_types)]
#[repr(u8)]
pub(crate) enum ChannelId {
    Rpc = 0,
    FileSystem,
    CartonData,

    // Reserved
    NUM_RESERVED_IDS,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct FdId(pub(crate) u64);

// With this interface, creating a Carton is 
//      Pack(data) -> core library packaging -> model.carton
// Loading it is
//      model.carton -> core library unpackaging -> Load(data)
//
// Loading an unpacked model is effectively
//      Pack(data) -> Load(data)
// (from the perspective of a runner)

#[derive(Debug, Serialize, Deserialize)]
pub enum RPCRequestData {
    Load {
        /// This filesystem points to a folder that is of the same structure as the output of `Pack` (for a particular runner)
        /// For a readonly filesystem
        fs: FsToken,

        runner_name: Option<String>,
        required_framework_version: Option<String>,
        runner_compat_version: u64,

        // TODO: fix this
        runner_opts: Option<String>,
        visible_device: Device,

        // The hash of the model
        carton_manifest_hash: String,
    },

    // Pack a model
    Pack {
        /// A token for a read/write filesystem that the below paths reference
        fs: FsToken,

        // The path to user input data
        // If this is a folder, the runner is allowed to place data in a `.carton` subfolder
        // This can be used if it wants to generate a lockfile for example
        input_path: String,

        // A temporary folder generated by the core library. The runner can use this if it needs
        // to generate output in a new folder.
        // (In some cases, the input can be wrapped as-is and doesn't need to be copied into a new folder)
        // This folder is owned by the core library and will be deleted by it
        temp_folder: String,
    },

    Seal {
        tensors: HashMap<String, Handle<Tensor>>
    },

    InferWithTensors {
        tensors: HashMap<String, Handle<Tensor>>
    },

    InferWithHandle {
        handle: SealHandle
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RPCResponseData {
    Load {
        name: String,
        
        // TODO: Change this to a runnerinfo struct
        runner: String,
    },

    Pack {
        // The path to the output directory. This can be in the temp folder passed into `Pack`
        // Note: this must be a *directory* even if the input was a file
        // This references a path on the FS that was passed in
        // during the request
        output_path: String
    },

    Seal {
        handle: SealHandle
    },

    Infer {
        tensors: HashMap<String, Handle<Tensor>>
    },

    /// Something went wrong
    Error {
        e: String,
    },

    // This should be used only when something is expected to take a long time (e.g generating a lockfile for a python project)
    SlowLog {
        e: String,
    }

}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct SealHandle(u64);

impl SealHandle {
    pub fn new(v: u64) -> Self {
        SealHandle(v)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Device {
    CPU,
    GPU {
        /// The UUID of the specified device
        /// This must include the `GPU-` or `MIG-GPU-` prefix
        /// See https://docs.nvidia.com/cuda/cuda-c-programming-guide/index.html#env-vars
        uuid: Option<String>
    }
}

for_each_carton_type! {
    /// TODO: We should to manually implement serialization and not depend on ndarray's serialization
    /// staying the same. Or just pin to a specific ndarray version
    #[derive(Debug, Serialize, Deserialize)]
    pub enum Tensor {
        $($CartonType(ndarray::ArrayD::<$RustType>),)*

        // A Nested Tensor / Ragged Tensor
        // NestedTensor(Vec<Tensor>)
    }
}

for_each_carton_type! {
    $(
        impl From<ndarray::ArrayD<$RustType>> for Tensor {
            fn from(item: ndarray::ArrayD<$RustType>) -> Self {
                Tensor::$CartonType(item)
            }
        }
    )*
}


// If we're running in wasm, wrap inner and derive serialize and deserialize normally (because we can't use shared memory)
if_wasm! {
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Handle<T> {
        inner: T
    }

    impl Handle<Tensor> {
        fn new(inner: Tensor, runner: &Comms) -> Self {
            Self { inner }
        }
    }
}

if_not_wasm! {
    // This stores info about a shared memory region
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Handle<T> {
        // The ID of the file descriptor backing this item
        fd_id: FdId,

        // The size in bytes of the region
        size_bytes: u64,

        _pd: PhantomData<T>
    }
}

#[cfg(not(target_family = "wasm"))]
impl Handle<Tensor> {
    async fn new(t: Tensor, comms: &Comms) -> Self {
        // Actually build or get the shared memory region backing the tensor

        // let fd_id = runner.send_fd(fd).await;

        todo!()
    }
}