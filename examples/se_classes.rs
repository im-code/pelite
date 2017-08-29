/*!
The ClientClass links client and server entities.
*/

extern crate pelite;

use std::env;

use pelite::pe32::{Va, Ptr, Pe, PeFile};
use pelite::util::{CStr, Pod};

//----------------------------------------------------------------

fn main() {
	let mut args = env::args_os();
	args.next();

	if args.len() == 0 {
		println!("PeLite example: Source Engine ClientClasses.");
	}
	else {
		for ref path in args {
			match pelite::FileMap::open(path) {
				Ok(file) => {
					match PeFile::from_bytes(&file).and_then(classes) {
						Ok(list) => display(list),
						Err(err) => eprintln!("pelite: error parsing {:?}: {}", path, err),
					};
				},
				Err(err) => {
					eprintln!("pelite: error opening {:?}: {}", path, err);
				},
			};
		}
	}
}

fn display(list: Vec<Class>) {
	for class in &list {
		println!("{:?}", class);
	}
}

//----------------------------------------------------------------

#[allow(non_snake_case)]
#[derive(Debug)]
#[repr(C)]
struct ClientClass {
	pCreateFn: Va,
	pCreateEventFn: Va,
	pNetworkName: Ptr<CStr>,
	pRecvTable: Va,
	pNext: Ptr<ClientClass>,
	ClassID: i32,
}
unsafe impl Pod for ClientClass {}

//----------------------------------------------------------------

#[derive(Debug)]
pub struct Class<'a> {
	pub network_name: &'a str,
	pub class_id: i32,
	pub size_of: u32,
}

pub fn classes<'a>(client: PeFile<'a>) -> pelite::Result<Vec<Class<'a>>> {
	let mut list = Vec::new();
	let scanner = client.scanner();

	// The ClientClasses aren't fully constructed yet, find these constructors
	// ```
	// mov     eax, g_pClientClassHead
	// mov     s_ClientClass.pNext, eax
	// mov     g_pClientClassHead, offset s_ClientClass
	// retn
	// ```
	let pat = pelite::pattern::parse("A1*{'} A3*{'} C705*{'}*{'???? ???? *{'}} C3").unwrap();

	for m in scanner.matches_code(&pat) {
		// Remove false positives
		if m.1 != m.3 || m.2 != m.4 + 0x10 {
			continue;
		}
		// Now dealing with a ClientClass
		let client_class: &ClientClass = client.derva(m.4).unwrap();
		let network_name = client.deref_str(client_class.pNetworkName).unwrap().to_str().unwrap();
		// Figure out the size of the entity type:
		// The CreateFn is a function to create instances of this entity type, it allocates memory and thus includes its size
		let size_of = if let Ok(bytes) = client.read_bytes(client_class.pCreateFn) {
			// Old Source: TF2, L4D2, etc...
			if bytes.starts_with(&[0x55, 0x8B, 0xEC, 0x56, 0x68]) {
				client.deref_copy(client_class.pCreateFn + 5).unwrap_or(1u32)
			}
			// New Source: CSGO (the allocation function is inlined)
			else if bytes.starts_with(&[0x55, 0x8B, 0xEC, 0xA1]) {
				client.deref_copy(client_class.pCreateFn + 39).unwrap_or(1u32)
			}
			// Unknown...
			else { 2u32 }
		}
		else { 0u32 };
		// Class ids are initialized somewhere else...
		let class_id = 0;
		list.push(Class { network_name, class_id, size_of })
	}

	Ok(list)
}
