pub mod region;
pub mod rtti;

use windows::{Win32::Foundation::*, Win32::System::SystemServices::*, Win32::System::Console::*};

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            unsafe { AllocConsole().expect("Could not allocate console!") };            

            let region = region::Region::from_module(None);
            let vftable = rtti::VFTable::find(&region, ".?AVRakPeer@RakNet@@").expect("Could not find RakPeer VFTable!"); 
            
            println!("{vftable:?}");

            let raknet_Recieve = unsafe { vftable.get::<fn (*mut (), u8) -> ()>(24) };
                
            println!("{raknet_Recieve:p} {:?}", *raknet_Recieve);
            
            raknet_Recieve(std::ptr::null_mut(), 0);
        },
        DLL_PROCESS_DETACH => (),
        _ => (),
    }

    true
}
