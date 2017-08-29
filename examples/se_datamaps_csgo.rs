/*!
Datamaps manage entity state.
*/

#![allow(bad_style)]

extern crate pelite;

use std::env;

use pelite::pe32::{Va, Ptr, Pe, PeFile};
use pelite::util::{CStr, Pod};

//----------------------------------------------------------------

fn main() {
	let mut args = env::args_os();
	args.next();

	if args.len() == 0 {
		println!("PeLite example: CSGO Datamaps.");
	}
	else {
		for ref path in args {
			match pelite::FileMap::open(path) {
				Ok(file) => {
					match PeFile::from_bytes(&file).and_then(datamaps) {
						Ok(list) => display(list),
						Err(err) => eprintln!("pelite: error parsing {:?}: {}", path, err),
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

fn display(list: Vec<Class>) {
	for class in &list {
		print!("class {}", class.name);
		if let Some(base) = class.base {
			print!(" extends {}", base);
		}
		println!(" {{");
		for field in &class.fields {
			println!("\t#[field_offset({})] {}: {},", field.offset, field.name, field.ty);
		}
		println!("}}");
	}
}

//----------------------------------------------------------------

const TD_OFFSET_COUNT: usize = 2;

#[derive(Debug)]
#[repr(C)]
struct typedescription_t {
	fieldType: i32,
	fieldName: Ptr<CStr>,
	fieldOffset: [i32; TD_OFFSET_COUNT],
	fieldSize: u16,
	flags: i16,
	externalName: Ptr<CStr>,
	pSaveRestoreOps: Va,
	td: Ptr<datamap_t>,
	fieldSizeInBytes: i32,
	override_field: Ptr<typedescription_t>,
	override_count: i32,
	fieldTolerance: f32,
	unk: [i32; 3],
}
// sizeof(typedescription_t) == 60

#[derive(Debug)]
#[repr(C)]
struct datamap_t {
	dataDesc: Ptr<[typedescription_t]>,
	dataNumFields: i32,
	dataClassName: Ptr<CStr>,
	baseMap: Ptr<datamap_t>,
	chains_validated: bool,
	packed_offsets_computed: bool,
	packed_size: i32,
}

unsafe impl Pod for typedescription_t {}
unsafe impl Pod for datamap_t {}

static FIELD_TYPES: [&str; 29] = [
	"Void",
	"Float",
	"String",
	"Vector",
	"Quaternion",
	"Integer",
	"Boolean",
	"Short",
	"Char",
	"Color32",
	"Embedded",
	"Custom",
	"ClassPtr",
	"EHandle",
	"EDict",
	"PositionVector",
	"Time",
	"Tick",
	"ModelName",
	"SoundName",
	"Input",
	"Function",
	"VMatrix",
	"VMatrixWorldSpace",
	"Matrix3x4WorldSpace",
	"Interval",
	"ModelIndex",
	"MaterialIndex",
	"Vector2D",
];

//----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Class<'a> {
	pub name: &'a str,
	pub base: Option<&'a str>,
	pub fields: Vec<Field<'a>>,
}
#[derive(Copy, Clone, Debug)]
pub struct Field<'a> {
	pub ty: &'a str,
	pub name: &'a str,
	pub offset: i32,
}

pub fn datamaps<'a>(client: PeFile<'a>) -> pelite::Result<Vec<Class<'a>>> {
	let mut classes = Vec::new();
	// The datamaps aren't fully constructed yet, find these constructors
	let scanner = client.scanner();
	let pat = pelite::pattern::parse("C705????'???? C705*{'}*{'} C3 CCCCCCCCCCCCCCCCCCCCCC").unwrap();
	for m in scanner.matches_code(&pat) {
		let num = client.derva_copy::<i32>(m.1).unwrap();
		let datamap = client.derva::<datamap_t>(m.2);
		let tydescs = client.derva_slice::<typedescription_t, _>(m.3, num as usize);

		// The match isn't perfect and includes some false positives, skip these
		if let (Ok(datamap), Ok(tydescs)) = (datamap, tydescs) {
			// Ignore further false positives
			if let Ok(class) = dataclass(client, datamap, tydescs) {
				classes.push(class);
			}
		}
	}
	Ok(classes)
}

fn dataclass<'a>(client: PeFile<'a>, datamap: &datamap_t, tydescs: &[typedescription_t]) -> pelite::Result<Class<'a>> {
	let mut fields = Vec::new();
	for tydesc in tydescs {
		if let Ok(field_name) = client.deref_str(tydesc.fieldName) {
			let ty = client.deref(tydesc.td)
				.and_then(|td| client.deref_str(td.dataClassName))
				.map(|name| name.to_str().unwrap())
				.unwrap_or(*FIELD_TYPES.get(tydesc.fieldType as usize).unwrap_or(&"?"));
			fields.push(Field {
				name: field_name.to_str().unwrap(),
				ty,
				offset: tydesc.fieldOffset[0],
			});
		}
	}
	fields.sort_by_key(|field| field.offset);

	let class_name = client.deref_str(datamap.dataClassName)?.to_str().unwrap();
	let base_class = client.deref(datamap.baseMap).and_then(|basemap| {
		Ok(client.deref_str(basemap.dataClassName)?.to_str().unwrap())
	}).ok();

	Ok(Class {
		name: class_name,
		base: base_class,
		fields,
	})
}
