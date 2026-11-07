//! Reduce a V16 metadata snapshot to what opengov-cli's static codegen needs: the calls of the
//! given pallets (or of every pallet). Storage, events, errors, constants, runtime APIs, docs, and
//! the `RuntimeCall` variants of dropped pallets are removed, along with every type only they used.
//! Pallet and call indices are unchanged, so encoded calls are byte-identical.
//!
//! Usage: strip-metadata <input.scale> <output.scale> [Pallet1,Pallet2,...]

use codec::{Decode, Encode};
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use scale_info::TypeDef;
use std::collections::BTreeSet;
use subxt_utils_stripmetadata::StripMetadata;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	if args.len() < 3 || args.len() > 4 {
		eprintln!("usage: {} <input.scale> <output.scale> [Pallet1,Pallet2,...]", args[0]);
		std::process::exit(2);
	}
	let bytes = std::fs::read(&args[1]).expect("input metadata file should be readable");
	let mut prefixed =
		RuntimeMetadataPrefixed::decode(&mut &bytes[..]).expect("input should be valid metadata");
	let RuntimeMetadata::V16(ref mut md) = prefixed.1 else {
		panic!("only V16 metadata is supported");
	};

	let keep: Option<BTreeSet<String>> =
		args.get(3).map(|list| list.split(',').map(|p| p.trim().to_string()).collect());
	if let Some(keep) = &keep {
		for name in keep {
			assert!(
				md.pallets.iter().any(|p| &p.name == name),
				"pallet `{name}` is not in {}",
				args[1]
			);
		}
		md.pallets.retain(|p| keep.contains(&p.name));
	}

	// Keep only calls (and System's associated types, which subxt uses).
	for pallet in &mut md.pallets {
		pallet.storage = None;
		pallet.event = None;
		pallet.error = None;
		pallet.constants.clear();
		pallet.view_functions.clear();
		pallet.docs.clear();
	}
	md.apis.clear();

	// Drop outer enum variants for removed pallets/items so their types become unreferenced.
	let callable: BTreeSet<String> =
		md.pallets.iter().filter(|p| p.calls.is_some()).map(|p| p.name.clone()).collect();
	let call_ty = md.outer_enums.call_enum_ty.id;
	let event_ty = md.outer_enums.event_enum_ty.id;
	let error_ty = md.outer_enums.error_enum_ty.id;
	for ty in &mut md.types.types {
		if let TypeDef::Variant(v) = &mut ty.ty.type_def {
			if ty.id == call_ty {
				v.variants.retain(|var| callable.contains(&var.name));
			} else if ty.id == event_ty || ty.id == error_ty {
				v.variants.clear();
			}
		}
	}

	// Docs become `#[doc]` attributes on every generated item and are never needed here.
	for ty in &mut md.types.types {
		ty.ty.docs.clear();
		match &mut ty.ty.type_def {
			TypeDef::Composite(c) => c.fields.iter_mut().for_each(|f| f.docs.clear()),
			TypeDef::Variant(v) =>
				for var in &mut v.variants {
					var.docs.clear();
					var.fields.iter_mut().for_each(|f| f.docs.clear());
				},
			_ => {},
		}
	}

	// Garbage-collect unreferenced types (all remaining pallets kept, no runtime APIs).
	md.strip_metadata(|_| true, |_| false);

	std::fs::write(&args[2], prefixed.encode()).expect("output should be writable");
}
