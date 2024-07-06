pub mod region;
pub mod rtti;

use windows::Win32::System::SystemServices::*;
use windows::Win32::System::Console::AllocConsole;

use region::Region;

#[no_mangle]
pub extern "system" fn DllMain(_: usize, fwd_reason: u32, _: usize) -> bool {
    if fwd_reason == DLL_PROCESS_ATTACH {
    	unsafe { let _ = AllocConsole(); };
        let mut region = Region::from_module(None);

        let vftable = region.find_vftable(".?AVRakPeer@RakNet@@");
        let addr = vftable.unwrap().get::<u8>(3);

        println!("{:?}", addr);
    }

    true
}
