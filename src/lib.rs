#![feature(c_unwind)]
#![allow(non_snake_case)]

#[macro_use] extern crate gmod;

mod cpp;

use std::{ffi::{CStr, c_void}, os::raw::c_char};

static mut DETOUR: Option<gmod::detour::GenericDetour<RunStringEx>> = None;

#[cfg_attr(all(target_os = "windows", target_pointer_width = "64"), abi("fastcall"))]
#[cfg_attr(all(target_os = "windows", target_pointer_width = "32"), abi("stdcall"))]
#[type_alias(RunStringEx)]
extern "cdecl" fn RunStringEx_Detour(this: *mut c_void, path: *const c_char, unk1: *const c_char, src: *const c_char, unk2: bool, unk3: bool, unk4: bool, unk5: bool) -> usize {
	enum VecOrPtr {
		Vec(Vec<u8>),
		Ptr(*const c_char)
	}
	impl VecOrPtr {
		#[inline]
		fn as_ptr(&self) -> *const c_char {
			match self {
				Self::Vec(vec) => vec.as_ptr() as _,
				Self::Ptr(ptr) => *ptr
			}
		}
	}

	let src = match cpp::preprocess(unsafe { CStr::from_ptr(src) }.to_bytes()).unwrap() {
		Ok(mut src) => {
			src.push(0);
			VecOrPtr::Vec(src)
		},
		Err(err) => {
			eprintln!("Failed to preprocess: {err}");
			VecOrPtr::Ptr(src)
		}
	};

	unsafe {
		DETOUR.as_ref().unwrap().call(
			this, path, unk1, src.as_ptr(),
			unk2, unk3, unk4, unk5
		)
	}
}

#[gmod13_open]
unsafe fn gmod13_open(lua: gmod::lua::State) {
	if cpp::can_preprocess() {
		let (_lib, _path) = open_library_srv!("lua_shared").expect("Failed to find lua_shared!");

		let RunStringEx: RunStringEx = find_gmod_signature!((_lib, _path) -> {

			win64_x86_64: [@SIG = "40 55 53 56 57 41 54 41 56 41 57 48 8D AC 24 ? ? ? ? 48 81 EC ? ? ? ? 48 8B 05 ? ? ? ? 48 33 C4 48 89 85 ? ? ? ? 49 8B F1 4D 8B F8 4C 8B F2 48 8B F9 4D 85 C9 0F 84"],
			win32_x86_64: [@SIG = "55 8B EC 81 EC ? ? ? ? 56 57 8B 7D 10 8B F1 85 FF 0F 84 ? ? ? ? 57 E8 ? ? ? ? 83 C4 04 83 F8 01 0F 8C ? ? ? ? 80 3F 1B 75 35 8B 06 6A 1B 68 ? ? ? ? 56 FF 90 ? ? ? ?"],

			linux64_x86_64: [@SIG = "55 48 89 E5 41 57 41 56 41 55 49 89 F5 41 54 53 48 89 FB 48 81 EC ? ? ? ? 44 89 85 ? ? ? ? 8B 45 18 44 89 8D ? ? ? ? 44 8B 75 10 89 85"],
			linux32_x86_64: [@SIG = "55 89 E5 57 56 53 81 EC ? ? ? ? 8B 45 18 8B 55 14 8B 5D 08 8B 7D 0C 89 85 ? ? ? ? 8B 45 1C 8B 75 10 89 85 ? ? ? ? 8B 45 20 89 85 ? ? ? ? 8B 45 24 89 85 ? ? ? ? 65 A1 ? ? ? ? 89 45 E4 31 C0 85 D2 0F 84 ? ? ? ? 89 14 24 89 95"],

			win32: [@SIG = "55 8B EC 8B 55 10 81 EC ? ? ? ? 56 8B F1 57 85 D2 0F 84 ? ? ? ? 8B CA 8D 79 01 8D 49 00 8A 01 41 84 C0 75 F9 2B CF 83 F9 01 0F 8C ? ? ? ? 80 3A 1B 75 35"],
			linux32: [@SIG = "55 89 E5 57 56 53 81 EC ? ? ? ? 8B 45 18 8B 55 14 8B 5D 08 8B 7D 0C 89 85 ? ? ? ? 8B 45 1C 8B 75 10 89 85 ? ? ? ? 8B 45 20 89 85 ? ? ? ? 8B 45 24 89 85 ? ? ? ? 65 A1 ? ? ? ? 89 45 E4 31 C0 85 D2 0F 84 ? ? ? ? 89 14 24 89 95 ? ? ? ?"],

		}).expect("Failed to find CLuaInterface::RunStringEx");

		let detour = gmod::detour::GenericDetour::new::<RunStringEx>(RunStringEx, RunStringEx_Detour).expect("Failed to detour CLuaInterface::RunStringEx");
		detour.enable().expect("Failed to enable CLuaInterface::RunStringEx detour");

		DETOUR = Some(detour);
	}
}