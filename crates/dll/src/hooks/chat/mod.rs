// 0x005649F0
//
// [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvMemberFunction) })]
// private static int ClientSystem_AddTextToScroll_Impl(IntPtr This, PStringBase<ushort>* text, eChatTypes type, byte unknown, StringInfo* info) {
//     var eventArgs = new ChatTextAddedEventArgs(text->ToString().TrimEnd('\r', '\n'), (ChatType)type);
//     StandaloneLoader.Backend.HandleChatTextAdded(eventArgs);

//     if (eventArgs.Eat) {
//         return 0;
//     }

//     return _ClientSystem_AddTextToScrollHook!.OriginalFunction(This, text, type, unknown, info);
// }

use std::{
    ffi::{c_void, CStr, OsString},
    os::windows::ffi::OsStringExt,
};

use once_cell::sync::Lazy;
use retour::GenericDetour;

type PStringBase = *const u16;
type eChatTypes = u32;
unsafe fn wide_char_ptr_to_string(ptr: PStringBase) -> String {
    let mut wide_chars = Vec::new();
    let mut i = 0;
    loop {
        let ch = *ptr.add(i);
        if ch == 0 {
            break; // Null terminator
        }
        wide_chars.push(ch);
        i += 1;
    }
    OsString::from_wide(&wide_chars)
        .to_string_lossy()
        .into_owned()
}

type fn_OnChatCommand_Impl = extern "system" fn(
    This: *mut c_void,
    text: PStringBase,
    chatType: eChatTypes,
    unk: u8,
    info: *mut c_void,
) -> i32;

extern "system" fn Hook_AddTextToScroll_Impl(
    This: *mut c_void,
    text: PStringBase,
    chatType: eChatTypes,
    unk: u8,
    info: *mut c_void,
) -> i32 {
    println!("AddTextToScroll_Impl");

    // let pstring = unsafe { PStringBase::from_mut_ptr(text as *mut PSRefBuffer) };
    // println!("pstring to_string is {pstring}");
    let wide_str = unsafe { wide_char_ptr_to_string(text) };
    println!("wide_str is {wide_str}");

    let ret_val = Hook_AddTextToScroll.call(This, text, chatType, unk, info);

    ret_val
}

pub static Hook_AddTextToScroll: Lazy<GenericDetour<fn_OnChatCommand_Impl>> = Lazy::new(|| {
    println!("AddTextToScroll_Impl");

    let address = 0x005649F0 as isize;
    let ori: fn_OnChatCommand_Impl = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl).unwrap() };
});
