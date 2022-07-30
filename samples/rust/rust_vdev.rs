//! Virtual Device Module
use kernel::prelude::*;

module! {
    type: VDev,
    name: b"vdev",
    license: b"GPL",
}

struct VDev;

impl kernel::Module for VDev {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        // Print a banner to make sure our moudle is working
        pr_info!("-----------------------\n");
        pr_info!("initialize vdev module!\n");
        pr_info!("-----------------------\n");
        Ok(VDev)
    }
}
