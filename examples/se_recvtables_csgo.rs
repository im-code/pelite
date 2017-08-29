/*!
RecvTables for networking entity data.
*/

#![allow(bad_style)]

extern crate pelite;

use std::mem;

use pelite::pe32::{Va, Ptr, Pe, PeFile};
use pelite::util::{CStr, Pod};

//----------------------------------------------------------------

fn main() {
	let mut args = env::args_os();
	args.next();

	if args.len() == 0 {
		println!("PeLite example: CSGO RecvTables.");
	}
	else {
		for ref path in args {
			match pelite::FileMap::open(path) {
				Ok(file) => {
					match PeFile::from_bytes(&file).and_then(recvtables) {
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

//----------------------------------------------------------------

#[derive(Debug)]
#[repr(C)]
struct RecvTable {
	pProps: Ptr<RecvProp>,
	nProps: i32,
	pDecoder: Va,
	pNetTableName: Ptr<CStr>,
	bInitialized: u8,
	bInMainList: u8,
}
#[derive(Debug, Clone)]
#[repr(C)]
struct RecvProp {
	pVarName: Ptr<CStr>,
	RecvType: i32,
	Flags: i32,
	StringBufferSize: i32,
	bInsideArray: u8,
	pExtraData: Va,
	pArrayProp: Ptr<RecvProp>,
	ArrayLengthProxy: Va,
	ProxyFn: Va,
	DataTableProxyFn: Va,
	RecvTable: Ptr<RecvTable>,
	Offset: i32,
	ElementStride: i32,
	nElements: i32,
	pParentArrayPropName: Ptr<CStr>,
}
unsafe impl Pod for RecvTable {}
unsafe impl Pod for RecvProp {}

//----------------------------------------------------------------

pub fn recvtables(client: PeFile) -> pelite::Result<Vec<Class>> {
	// The RecvTables aren't constructed yet...
	let mut classes = Vec::new();

	// This pattern is quite the sight, isn't it?
	// To find one of these constructors, search a typical netvar and xref it.
	// `m.1`: End of constructor code
	// `m.2`: Address of first RecvProp of the RecvTable's props
	// `m.3`: Number of RecvProps
	// `m.4`: Name of the datatable
	// `m.5`: Start of constructor code
	let pat = pelite::pattern::parse("A1???? A801 0F85${'C705????*{'} C705????'???? C705???????? C705????*{'}} 83C801 'C705????00000000 A3").unwrap();
	for m in client.scanner().matches_code(&pat) {
		if let Ok(class) = build_recvtable(client, &m) {
			classes.push(class);
		}
	}

	// Variation of the above for DT_CSPlayer and others
	let patv = pelite::pattern::parse("55 8BEC A1???? 83EC? A801 0F85${'C705????*{'} B801000000 C705????'???? C705???????? C705????*{'}} 83C801 'B9???? A3").unwrap();
	for m in client.scanner().matches_code(&patv) {
		if let Ok(class) = build_recvtable(client, &m) {
			classes.push(class);
		}
	}

	Ok(classes)
}

fn build_recvtable(client: PeFile, m: &pelite::pattern::Match) -> pelite::Result<Class> {
	let props_rva = m.2;
	let code: &[u8] = client.derva_slice(m.5, (m.1 - m.5) as usize)?;
	let &n_props: &i32 = client.derva(m.3)?;
	let net_table_name = client.derva_c_str(m.4)?.to_str().unwrap();

	// Allocate memory to initialize the props
	let mut recv_props = vec![unsafe { mem::zeroed::<RecvProp>() }; n_props as usize];
	let props_size = (n_props as usize * mem::size_of::<RecvProp>()) as u32;
	let props_ptr = recv_props.as_mut_ptr() as *mut u8;

	// Run through the code initializing the memory
	use lde::{InsnSet, x86};
	for (opcode, _) in x86::lde(code, m.5) {
		// mov dword ptr addr, imm32
		if opcode.starts_with(&[0xC7, 0x05]) {
			let rva = client.va_to_rva(opcode.read::<Va>(2)).unwrap();
			let imm = opcode.read::<u32>(6);
			if rva >= props_rva && rva - props_rva < props_size {
				unsafe { *(props_ptr.offset((rva - props_rva) as isize) as *mut u32) = imm; }
			}
		}
		// mov byte ptr addr, imm8
		if opcode.starts_with(&[0xC6, 0x05]) {
			let rva = client.va_to_rva(opcode.read::<Va>(2)).unwrap();
			let imm = opcode.read::<u8>(6);
			if rva >= props_rva && rva - props_rva < props_size {
				unsafe { *(props_ptr.offset((rva - props_rva) as isize) as *mut u8) = imm; }
			}
		}
	}

	let mut props = Vec::new();
	for recv_prop in recv_props {
		if let Ok(var_name) = client.deref_c_str(recv_prop.pVarName) {
			props.push(Prop {
				name: var_name.to_str().unwrap().to_owned(),
				ty: recv_prop.RecvType.to_string(),
				offset: recv_prop.Offset as u32,
			});
		}
	}

	Ok(Class {
		base: None,
		name: net_table_name.to_owned(),
		id: 0,
		size_of: 0,
		props
	})
}

// fn static_constructors<'a>(client: PeFile<'a>) -> pelite::Result<&'a [Va]> {
// 	let pat = pattern::parse("68*{'} 68{'} E8???? 59 59 C705????02000000").unwrap();
// 	let m = client.scanner().find_code(&pat).expect("static constructors not found");
// 	let end = m.1;
// 	let start = m.2;
// 	let len = ((end - start) / 4) as usize;
// 	client.derva_slice(start, len)
// }
