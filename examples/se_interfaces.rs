/*!
Source Engine is managed through "interface" singletons.

These singletons are initialized when the DLL is loaded and add themselves to a global linked list called [`s_pInterfaceRegs`](https://github.com/ValveSoftware/source-sdk-2013/blob/master/mp/src/tier1/interface.cpp#L46).

To access such a singleton you call the appropriate DLL's exported function called [`CreateInterface`](https://github.com/ValveSoftware/source-sdk-2013/blob/master/mp/src/tier1/interface.cpp#L91) with the name of the interface (which includes a version number).
The name is misleading as it implies and instance is created, instead a pointer is returned to the global singleton which implements this interface.

This only allows you to query interfaces which you know by name and version, this code extracts the entire list of interfaces exposed by a DLL by walking this list manually.
*/

extern crate pelite;

use std::{env};

use pelite::pe32::{Rva, Va, Ptr, Pe, PeFile};
use pelite::pe32::exports::{Export};
use pelite::util::{CStr, Pod};

//----------------------------------------------------------------

fn main() {
	let mut args = env::args_os();
	args.next();

	if args.len() == 0 {
		println!("PeLite example: Source Engine interfaces.");
	}
	else {
		for ref path in args {
			match pelite::FileMap::open(path) {
				Ok(file) => {
					match PeFile::from_bytes(&file).and_then(|file| interfaces(file)) {
						Ok(list) => {
							for reg in list {
								println!("{}!{:08X} {}", reg.dll_name, reg.offset, reg.name);
							}
						},
						Err(err) => {
							eprintln!("pelite: error parsing {:?}: {}", path, err);
						},
					}
				},
				Err(err) => {
					eprintln!("pelite: error opening {:?}: {}", path, err);
				},
			};
		}
	}
}

//----------------------------------------------------------------

/// The interface registration as defined in the Valve Source SDK.
///
/// [`class InterfaceReg`](https://github.com/ValveSoftware/source-sdk-2013/blob/master/mp/src/public/tier1/interface.h#L72)
#[derive(Debug)]
#[repr(C)]
pub struct InterfaceReg {
	create_fn: Va,
	name: Ptr<CStr>,
	next: Ptr<InterfaceReg>,
}
unsafe impl Pod for InterfaceReg {}

//----------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct Interface<'a> {
	pub dll_name: &'a str,
	pub name: &'a str,
	pub offset: Rva,
}

pub fn interfaces<'a>(file: PeFile<'a>) -> pelite::Result<Vec<Interface<'a>>> {
	let exports = file.exports()?.by()?;
	let dll_name = exports.dll_name()?.to_str().unwrap();

	// Grab the CreateInterface export
	let create_interface_export = exports.name("CreateInterface")?;
	let create_interface_fn = match create_interface_export {
		Export::Symbol(&rva) => rva,
		_ => return Err(pelite::Error::Null),
	};

	// Grab the linked list of interface registrations
	#[allow(non_snake_case)]
	let s_pInterfaceRegs = {
		// push    ebp
		// mov     ebp, esp
		// pop     ebp
		// jmp     CreateInterfaceInternal
		// ... ... ...
		// push    ebp
		// mov     ebp, esp
		// push    esi
		// mov     esi, s_pInterfaceRegs
		let pat = pelite::pattern::parse("55 8BEC 5D E9$ 55 8BEC 56 8B35*{'}").unwrap();
		let m = file.scanner().exec(&pat, create_interface_fn).unwrap();
		m.1
	};

	// Of course, this linked list isn't yet initialized!
	// Search for the code which constructs this linked list to extract their information
	let mut list = Vec::new();

	//----------------------------------------------------------------
	// CS:GO
	let _ = {
		// Inlined InterfaceReg constructor for this interface
		// ```
		// mov     eax, s_pInterfaceRegs
		// mov     s_InterfaceReg.next, eax
		// mov     s_pInterfaceRegs, offset s_InterfaceReg
		// retn
		// ```

		// Create_fn returns the global singleton
		// ```
		// mov     eax, offset s_Interface
		// retn
		// ```
		let pat = pelite::pattern::parse("A1*{'} A3???? C705*{'}*{*{B8*'} *'} C3").unwrap();

		for m in file.scanner().matches_code(&pat) {
			// Reject false positive matches for the signature
			if m.1 != s_pInterfaceRegs || m.2 != s_pInterfaceRegs {
				continue;
			}

			// Extract the interface information
			let offset = m.3;
			let name = file.derva_str(m.4).unwrap().to_str().unwrap();
			list.push(Interface { dll_name, name, offset });
		}
	};

	//----------------------------------------------------------------
	// TF2, L4D2, etc...
	let _ = {
		// Find the static initializers which register the interface
		// ```
		// push    offset aInterfaceName
		// push    offset create_fn
		// mov     ecx, offset interface_reg
		// call    InterfaceReg::InterfaceReg
		// retn
		// ```

		// Create fn returns the global singleton
		// ```
		// mov     eax, offset g_Interface
		// retn
		// ```
		let pat = pelite::pattern::parse("68*{'} 68*{B8*'} B9???? E8${55 8BEC} C3").unwrap();
		for m in file.scanner().matches_code(&pat) {
			// Extract the interface information
			let name = file.derva_str(m.1).unwrap().to_str().unwrap();
			let offset = m.2;
			list.push(Interface { dll_name, name, offset });
		}
	};

	Ok(list)
}
