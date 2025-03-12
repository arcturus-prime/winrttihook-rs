use crate::region::Region;

use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

#[derive(Debug)]
pub struct VFTable {
    address: *mut fn(),
}

impl VFTable {
    pub fn find(region: &Region, mangled_name: &str) -> Option<Self> {
        let results = region.search(mangled_name.as_bytes());
        let base = region.base().cast::<u8>();

        for result in results {
            let result_byte = result.cast::<u8>();
            let ibo_type_descriptor = unsafe { result_byte.sub(0x10).offset_from(base) } as u32;

            let xref_ibos = region.search(&ibo_type_descriptor.to_ne_bytes());

            for xref in xref_ibos {
                let xref_byte = xref.cast::<u8>();
                let complete_obj_loc: usize = unsafe { std::mem::transmute(xref_byte.sub(0xC)) };

                let vftable_addrs = region.search(&complete_obj_loc.to_ne_bytes());

                if vftable_addrs.is_empty() {
                    continue;
                }

                let vftable_addr = unsafe { vftable_addrs[0].cast::<fn()>().add(1) };
                let vftable = Self {
                    address: vftable_addr,
                };

                return Some(vftable);
            }
        }

        None
    }
}

impl VFTable {
    pub unsafe fn get<T: Copy>(&self, position: usize) -> &'static T {
        self.address.add(position).cast::<T>().as_ref().unwrap_unchecked()
    }

    pub unsafe fn set<T>(&self, position: usize, func: T) {
        let mut flags = PAGE_PROTECTION_FLAGS(0);
        let target = self.address.add(position);

        VirtualProtect(target.cast(), 8, PAGE_EXECUTE_READWRITE, &mut flags).unwrap();

        *self.address.add(position).cast() = func;

        VirtualProtect(target.cast(), 8, flags, &mut flags).unwrap();
    }
}
