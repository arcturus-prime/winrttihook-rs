use std::ffi::CString;

use windows::core::PCSTR;
use windows::Win32::System::Diagnostics::Debug::{
    IMAGE_NT_HEADERS64, IMAGE_OPTIONAL_HEADER64, IMAGE_SECTION_HEADER,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;

pub struct Region {
    sections: Vec<*mut [u8]>,
}

impl Region {
    fn new() -> Self {
        Region {
            sections: Vec::new(),
        }
    }

    pub unsafe fn from_module_raw(base: *mut u8) -> Self {
        let mut region = Region::new();

        let dos = base as *mut IMAGE_DOS_HEADER;
        let coff = base.add((*dos).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;

        let optional = &mut (*coff).OptionalHeader as *mut IMAGE_OPTIONAL_HEADER64;
        let optional_size = (*coff).FileHeader.SizeOfOptionalHeader as usize;

        let sections_start = optional.cast::<u8>().add(optional_size) as *mut IMAGE_SECTION_HEADER;
        let sections_length = (*coff).FileHeader.NumberOfSections as usize;

        let sections = std::slice::from_raw_parts_mut(sections_start, sections_length);

        for section in sections {
            let start = base.add(section.VirtualAddress as usize);
            let size = section.Misc.VirtualSize as usize;

            let bytes = std::slice::from_raw_parts_mut(start, size);

            region.sections.push(bytes);
        }

        let headers_length = sections_start
            .add(sections_length)
            .cast::<u8>()
            .offset_from(base);

        let headers = std::slice::from_raw_parts_mut(base, headers_length as usize);

        region.sections.push(headers);

        region
    }

    pub fn from_module(module_name: Option<CString>) -> Self {
        let module = if module_name.is_none() {
            std::ptr::null()
        } else {
            module_name.unwrap().as_ptr().cast()
        };
        
        let handle = unsafe { GetModuleHandleA(PCSTR::from_raw(module)) }.unwrap();

        unsafe {
            let base: *mut u8 = std::mem::transmute(handle);
            Self::from_module_raw(base)
        }
    }

    pub fn search(&mut self, bytes: &[u8]) -> Vec<*mut [u8]> {
        let mut matches: Vec<*mut [u8]> = Vec::new();

        for i in 0..self.sections.len() {
            let section = self.sections[i];

            for j in 0..section.len() - bytes.len() {
                let k = j + bytes.len();

                let subslice = unsafe { &mut (*section) };
                if bytes == subslice {
                    matches.push(unsafe { &mut (*section)[j..k] });
                }
            }
        }

        matches
    }

    pub fn merge(&mut self, mut other: Self) {
        self.sections.append(&mut other.sections);
    }

    pub fn base(&self) -> *mut [u8] {
        let mut smallest = self.sections[0];

        for section in &self.sections {
            if unsafe { (*smallest).as_ptr() > (**section).as_ptr() } {
                smallest = *section;
            }
        }

        smallest
    }
}
