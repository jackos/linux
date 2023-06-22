//! Virtual Device Module
use kernel::prelude::*;
use kernel::{
    miscdev,
    Module,
    io_buffer::{IoBufferReader, IoBufferWriter},
    sync::{Ref, RefBorrow, smutex::Mutex},
    file::{flags, File, Operations}
};

module! {
    type: VDev,
    name: b"vdev",
    license: b"GPL",
    params: {
        devices: u32 {
            default: 1,
            permissions: 0o644,
            description: b"Number of virtual devices",
        },
    },
}

struct VDev {
    _devs: Vec<Pin<Box<miscdev::Registration<VDev>>>>,
}

struct Device {
    number: usize,
    contents: Mutex<Vec<u8>>,
}

impl Module for VDev {
    fn init(_name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        let count = {
            let lock = module.kernel_param_lock();
            (*devices.read(&lock)).try_into()?
        };
        pr_info!("-----------------------\n");
        pr_info!("starting {} vdevices!\n", count);
        pr_info!("watching for changes...\n");
        pr_info!("-----------------------\n");
        let mut devs = Vec::try_with_capacity(count)?;
        for i in 0..count {
            let dev = Ref::try_new(Device {
                number: i,
                contents: Mutex::new(Vec::new()),
            })?;
            let reg = miscdev::Registration::new_pinned(fmt!("vdev{i}"), dev)?;
            devs.try_push(reg)?;
        }
        Ok(Self { _devs: devs })
    }
}

#[vtable]
impl Operations for VDev {
    // The data that is passed into the open method
    type OpenData = Ref<Device>;
    // The data that is returned by running an open method
    type Data = Ref<Device>;

    fn open(context: &Ref<Device>, file: &File) -> Result<Ref<Device>> {
        pr_info!("File for device {} was opened\n", context.number);
        if file.flags() & flags::O_ACCMODE == flags::O_WRONLY {
            context.contents.lock().clear();
        }
        Ok(context.clone())
    }

    // Read the data contents and write them into the buffer provided
    fn read(
        data: RefBorrow<'_, Device>,
        _file: &File,
        writer: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was read\n", data.number);
        let offset = offset.try_into()?;
        let vec = data.contents.lock();
        let len = core::cmp::min(writer.len(), vec.len().saturating_sub(offset));
        writer.write_slice(&vec[offset..][..len])?;
        Ok(len)
    }

    // Read from the buffer and write the data in the contents after locking the mutex
    fn write(
        data: RefBorrow<'_, Device>,
        _file: &File,
        reader: &mut impl IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was written\n", data.number);
        let offset = offset.try_into()?;
        let len = reader.len();
        let new_len = len.checked_add(offset).ok_or(EINVAL)?;
        let mut vec = data.contents.lock();
        if new_len > vec.len() {
            vec.try_resize(new_len, 0)?;
        }
        reader.read_slice(&mut vec[offset..][..len])?;
        Ok(len)
    }
}
