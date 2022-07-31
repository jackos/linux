//! Virtual Device Module
use kernel::prelude::*;

use kernel::file::{File, Operations};
use kernel::io_buffer::{IoBufferReader, IoBufferWriter};
use kernel::sync::smutex::Mutex;
use kernel::sync::{Ref, RefBorrow};
use kernel::{miscdev, Module};

module! {
    type: VDev,
    name: b"vdev",
    license: b"GPL",
}
struct Device {
    number: usize,
    contents: Mutex<Vec<u8>>,
}

struct VDev {
    _dev: Pin<Box<miscdev::Registration<VDev>>>,
}

#[vtable]
impl Operations for VDev {
    type OpenData = Ref<Device>;
    type Data = Ref<Device>;

    fn open(context: &Ref<Device>, _file: &File) -> Result<Ref<Device>> {
        pr_info!("File for device {} was opened\n", context.number);
        Ok(context.clone())
    }

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

    fn write(
        data: RefBorrow<'_, Device>,
        _file: &File,
        reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was written\n", data.number);
        let copy = reader.read_all()?;
        let len = copy.len();
        *data.contents.lock() = copy;
        Ok(len)
    }
}

impl Module for VDev {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("-----------------------\n");
        pr_info!("initialize vdev module!\n");
        pr_info!("watching for changes...\n");
        pr_info!("-----------------------\n");
        let dev = Ref::try_new(Device {
            number: 0,
            contents: Mutex::new(Vec::new()),
        })?;
        let reg = miscdev::Registration::new_pinned(fmt!("vdev"), dev)?;
        Ok(Self { _dev: reg })
    }
}
