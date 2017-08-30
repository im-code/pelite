/*!
*/

extern crate pelite;
extern crate lde;

use std::env;

use pelite::pe32::{Rva, Pe, PeFile};
use pelite::pattern as pat;

//----------------------------------------------------------------

fn main() {
	let mut args = env::args_os();
	args.next();

	if args.len() == 0 {
		println!("PeLite example: CSGO Log console variables.");
	}
	else {
		for ref path in args {
			match pelite::FileMap::open(path) {
				Ok(file) => {
					match PeFile::from_bytes(&file).and_then(cvars) {
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

fn display(list: Vec<ConVar>) {
	for cvar in &list {
		println!("{}!{:08X} {:08X} {}", cvar.dll_name, cvar.offset, cvar.flags, cvar.name);
	}
}

//----------------------------------------------------------------

pub struct ConVar<'a> {
	dll_name: &'a str,
	name: &'a str,
	desc: Option<&'a str>,
	default_string: &'a str,
	offset: Rva,
	flags: u32,
	min_value: Option<f32>,
	max_value: Option<f32>,
}

pub fn cvars<'a>(file: PeFile<'a>) -> pelite::Result<Vec<ConVar<'a>>> {
	let dll_name = file.exports()?.dll_name()?.to_str().unwrap();
	let scanner = file.scanner();
	let mut cvars = Vec::new();

	let get_min_max_value = |m: &pat::Match| {
		let has_max = file.derva_copy::<u8>(m.0 + 1).unwrap();
		let max_value = file.derva_copy::<f32>(m.0 + 6).unwrap();
		let has_min = file.derva_copy::<u8>(m.0 + 11).unwrap();
		let min_value = file.derva_copy::<f32>(m.0 + 16).unwrap();
		(
			if has_min != 0 { Some(min_value) } else { None },
			if has_max != 0 { Some(max_value) } else { None },
		)
	};

	// Match static constructors which call [`ConVar::Create`](https://github.com/ValveSoftware/source-sdk-2013/blob/master/mp/src/public/tier1/convar.h#L383)

	// CSGO variant, ConVar with description
	let pat1 = pat::parse("6A? 51 C704????? 6A? 51 C704????? B9*{'} 6A? 68*{'} 68'???? 68*{'} 68*{'} E8$").unwrap();
	for m in scanner.matches_code(&pat1) {
		let (min_value, max_value) = get_min_max_value(&m);
		let offset = m.1;
		let desc = Some(file.derva_str(m.2).unwrap().to_str().unwrap());
		let flags = file.derva_copy(m.3).unwrap();
		let default_string = file.derva_str(m.4).unwrap().to_str().unwrap();
		let name = file.derva_str(m.5).unwrap().to_str().unwrap();
		cvars.push(ConVar { dll_name, name, desc, default_string, offset, flags, min_value, max_value });
	}

	// CSGO variant, ConVar without description
	let pat2 = pat::parse("6A? 51 C704????? 6A? 51 C704????? B9*{'} 6A? 6A00 68'???? 68*{'} 68*{'} E8$").unwrap();
	for m in scanner.matches_code(&pat2) {
		let (min_value, max_value) = get_min_max_value(&m);
		let offset = m.1;
		let desc = None;
		let flags = file.derva_copy(m.2).unwrap();
		let default_string = file.derva_str(m.3).unwrap().to_str().unwrap();
		let name = file.derva_str(m.4).unwrap().to_str().unwrap();
		cvars.push(ConVar { dll_name, name, desc, default_string, offset, flags, min_value, max_value });
	}

	// Old source, ConVar with description and without min/max values
	let pat3 = pat::parse("CC 68*{'} 68'???? 68*{'} 68*{'} B9*{'} E8$").unwrap();
	for m in scanner.matches_code(&pat3) {
		let min_value = None;
		let max_value = None;
		let desc = Some(file.derva_str(m.1).unwrap().to_str().unwrap());
		let flags = file.derva_copy(m.2).unwrap();
		let default_string = file.derva_str(m.3).unwrap().to_str().unwrap();
		let name = file.derva_str(m.4).unwrap().to_str().unwrap();
		let offset = m.5;
		cvars.push(ConVar { dll_name, name, desc, default_string, offset, flags, min_value, max_value });
	}

	// Old source, ConVar with description and with min/max values
	let pat4 = pat::parse("D905*{'} 51 D91C24 D905*{'} 6A01 51 D91C24 6A01 68*{'} 68'???? 68*{'} 68*{'} B9*{'} E8$").unwrap();
	for m in scanner.matches_code(&pat4) {
		let max_value = Some(file.derva_copy(m.1).unwrap());
		let min_value = Some(file.derva_copy(m.2).unwrap());
		let desc = Some(file.derva_str(m.3).unwrap().to_str().unwrap());
		let flags = file.derva_copy(m.4).unwrap();
		let default_string = file.derva_str(m.5).unwrap().to_str().unwrap();
		let name = file.derva_str(m.6).unwrap().to_str().unwrap();
		let offset = m.7;
		cvars.push(ConVar { dll_name, name, desc, default_string, offset, flags, min_value, max_value });
	}

	Ok(cvars)
}
