use crate::region::Region;

#[derive(Debug)]
pub struct VFTable {
	address: *const usize
}

impl VFTable {
	pub fn get<T>(&self, position: usize) -> *const T {
		let func_ptr = unsafe { self.address.add(position) };

		func_ptr.cast::<T>()
	}

	pub fn new(address: *const usize) -> Self {
		VFTable { address: address }
	}
}

impl Region {
    pub fn find_vftable(&mut self, mangled_name: &str) -> Option<VFTable> {
        let results = self.search(mangled_name.as_bytes());
        let base = self.base();

        for result in results {
            let ibo_type_descriptor = unsafe { result.cast::<u8>().sub(0x10).offset_from(base.cast::<u8>()) } as u32;
            let xref_ibos = self.search(&ibo_type_descriptor.to_ne_bytes());

            for xref in xref_ibos {
                let complete_obj_loc: usize = unsafe { std::mem::transmute(xref.cast::<u8>().sub(0xC)) };

                let vftable_addrs = self.search(&complete_obj_loc.to_ne_bytes());

                if vftable_addrs.is_empty() {
                	return None;
                }

                let vftable_addr = unsafe { vftable_addrs[0].cast::<usize>().add(1) };
                let vftable = VFTable::new(vftable_addr);

                return Some(vftable);
            }
        }

        None
    }
}
