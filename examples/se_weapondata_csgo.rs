/*!
Weapon Data.
*/

#![allow(bad_style)]

use ::pelite::pe32::{Rva, Pe, PeFile};
use ::pelite;

use ::{Class, Prop};

//----------------------------------------------------------------

pub fn build(client: PeFile) -> pelite::Result<Class> {
	let scanner = client.scanner();

	// Find the initialize function for CCSWeaponData and CWeaponData
	let init_pat = pelite::pattern::parse("55 8BEC 83E4F8 83EC14 53 56 8BF1 57 8B7E?").unwrap();
	let init_m = scanner.find_code(&init_pat).unwrap();

	analyse(client, init_m.0)
}

fn analyse(client: PeFile, code_rva: Rva) -> pelite::Result<Class> {
	let mut props = Vec::new();

	// Grab the code function bytes
	let code_va = client.rva_to_va(code_rva)?;
	let code: &[u8] = {
		let optional_header = client.optional_header();
		let remaining_code = optional_header.BaseOfCode + optional_header.SizeOfCode - code_rva;
		client.derva_slice(code_rva, remaining_code as usize)?
	};

	// Run through the initializing code
	let get_pat = pelite::pattern::parse("E8$ A1???? A801 75? 83C801 C705????*'").unwrap();
	let mut get_name = None;
	use lde::{InsnSet, x86};
	for (opcode, va) in x86::lde(code, code_va) {
		// Find functions which call `CEconItemSchema__GetAttributeDefinition`
		if opcode.starts_with(&[0xE8]) {
			if let Some(get_m) = client.scanner().exec(&get_pat, client.va_to_rva(va).unwrap()) {
				let name = client.derva_c_str(get_m.1)?.to_str().unwrap();
				if let Some(previous_name) = get_name {
					println!("missing offset \"{}\"", previous_name);
				}
				get_name = Some(name);
			}
		}
		// movss dword ptr [esi + dword offset], xmm0
		else if opcode.starts_with(&[0xF3, 0x0F, 0x11, 0x86]) {
			let offset = opcode.read::<u32>(4);
			if let Some(name) = get_name {
				props.push(Prop(name, "Float", offset));
				get_name = None;
			}
			else {
				println!("missing float {:#X}", offset);
			}
		}
		// mov dword ptr [esi + dword offset], reg
		// where reg is eax, ecx or ebx
		else if opcode.starts_with(&[0x89]) && opcode[1] & 0b11_000_111 == 0b10_000_110 {
			let offset = opcode.read::<u32>(2);
			if let Some(name) = get_name {
				props.push(Prop(name, "Int", offset));
				get_name = None;
			}
			else {
				println!("missing int {:#X}", offset);
			}
		}
		// mov byte ptr [esi + dword offset], al
		else if opcode.starts_with(&[0x88, 0x86]) {
			let offset = opcode.read::<u32>(2);
			if let Some(name) = get_name {
				props.push(Prop(name, "Bool", offset));
				get_name = None;
			}
			else {
				println!("missing bool {:#X}", offset);
			}
		}
		// End of the function
		else if opcode.as_ref() == &[0xC3] {
			break;
		}
	}
	
	Ok(Class {
		base: None,
		name: String::from("CCSWeaponData"),
		id: 0,
		size_of: 0,
		props
	})
}
