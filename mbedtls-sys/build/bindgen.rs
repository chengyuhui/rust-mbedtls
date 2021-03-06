/*
 * Rust bindings for mbedTLS
 *
 * (C) Copyright 2016 Jethro G. Beekman
 *
 * This program is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License as published by the Free
 * Software Foundation; either version 2 of the License, or (at your option)
 * any later version. Alternatively, you can redistribute it and/or modify it
 * under the terms of the Apache License, Version 2.0. 
 */

use bindgen;

use std::fs::File;
use std::io::{stderr,Write};

use headers;

#[derive(Debug)]
struct StderrLogger;

impl bindgen::Logger for StderrLogger {
    fn error(&self, msg: &str) { let _=writeln!(stderr(),"Bindgen ERROR: {}",msg); }
    fn warn(&self, msg: &str) { let _=writeln!(stderr(),"Bindgen WARNING: {}",msg); }
}

impl super::BuildConfig {
	pub fn bindgen(&self) {
		let header=self.out_dir.join("bindgen-input.h");
		File::create(&header).and_then(|mut f|Ok(
			for h in headers::enabled_ordered() {
				try!(writeln!(f,"#include <mbedtls/{}>",h));
			}
		)).expect("bindgen-input.h I/O error");

		let include=self.mbedtls_src.join("include");
		
		let logger=StderrLogger;
		let mut bindgen=bindgen::Builder::new(header.into_os_string().into_string().unwrap());
		let bindings=bindgen
			.log(&logger)
			.clang_arg("-Dmbedtls_t_udbl=mbedtls_t_udbl;") // bindgen can't handle unused uint128
			.clang_arg(format!("-DMBEDTLS_CONFIG_FILE=<{}>",self.config_h.to_str().expect("config.h UTF-8 error")))
			.clang_arg(format!("-I{}",include.to_str().expect("include/ UTF-8 error")))
			.match_pat(include.to_str().expect("include/ UTF-8 error"))
			.match_pat(self.config_h.to_str().expect("config.h UTF-8 error"))
			.use_core(true)
			.derive_debug(false) // buggy :(
			.ctypes_prefix(vec!["types".to_owned(),"raw_types".to_owned()])
			.remove_prefix("mbedtls_")
			.rust_enums(false)
			.convert_macros(true)
			.macro_int_types(vec!["sint","sint","sint","slonglong","sint","sint","sint","slonglong"].into_iter())
			.generate().expect("bindgen error");

		let bindings_rs=self.out_dir.join("bindings.rs");
		File::create(&bindings_rs).and_then(|mut f|{
			try!(bindings.write(Box::new(&mut f)));
			f.write_all(b"use ::types::*;\n") // for FILE, time_t, etc.
		}).expect("bindings.rs I/O error");

		let mod_bindings=self.out_dir.join("mod-bindings.rs");
		File::create(&mod_bindings).and_then(|mut f|
			f.write_all(b"mod bindings;\n")
		).expect("mod-bindings.rs I/O error");
	}
}
