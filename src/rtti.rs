use crate::region::Region;

#[derive(Debug)]
pub struct VFTable {
	address: *mut usize
}

impl VFTable {
    pub fn find(region: &Region, mangled_name: &str) -> Option<Self> {
        let results = region.search(mangled_name.as_bytes());
        let base = region.base();

        for result in results {
            let ibo_type_descriptor = unsafe { result.cast::<u8>().sub(0x10).offset_from(base.cast::<u8>()) } as u32;
            let xref_ibos = region.search(&ibo_type_descriptor.to_ne_bytes());

            for xref in xref_ibos {
                let complete_obj_loc: usize = unsafe { std::mem::transmute(xref.cast::<u8>().sub(0xC)) };
 
                let vftable_addrs = region.search(&complete_obj_loc.to_ne_bytes());

                if vftable_addrs.is_empty() {
                    return None;
                }

                let vftable_addr = unsafe { vftable_addrs[0].cast::<usize>().add(1) };
                let vftable = Self { address: vftable_addr };

                return Some(vftable);
            }
        }

        None
    }
}

impl VFTable {
    pub unsafe fn get<T>(&self, position: usize) -> *mut *const T {
        self.address.add(position).cast()
    }

    pub unsafe fn set<T>(&self, position: usize, func: *const T) {
        *self.address.add(position).cast() = func
    }
}