mod tokenizer;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use libduckdb_sys::{duckdb_data_chunk, duckdb_function_info, duckdb_vector};
    use quack_rs::prelude::*;

    use crate::tokenizer;

    fn set_error(info: duckdb_function_info, message: impl AsRef<str>) {
        unsafe { ScalarFunctionInfo::new(info) }.set_error(message.as_ref());
    }

    unsafe fn read_optional_str(reader: &VectorReader, row: usize) -> Option<&str> {
        if unsafe { reader.is_valid(row) } {
            Some(unsafe { reader.read_str(row) })
        } else {
            None
        }
    }

    unsafe fn write_split_result(
        input: duckdb_data_chunk,
        output: duckdb_vector,
        info: duckdb_function_info,
        argc: usize,
    ) {
        let text = unsafe { VectorReader::new(input, 0) };
        let separator = (argc >= 2).then(|| unsafe { VectorReader::new(input, 1) });
        let options = (argc >= 3).then(|| unsafe { VectorReader::new(input, 2) });
        let mut out = unsafe { VectorWriter::new(output) };

        for row in 0..text.row_count() {
            let Some(text) = (unsafe { read_optional_str(&text, row) }) else {
                unsafe { out.set_null(row) };
                continue;
            };
            let separator = separator
                .as_ref()
                .and_then(|reader| unsafe { read_optional_str(reader, row) })
                .unwrap_or(" ");
            let options = options
                .as_ref()
                .and_then(|reader| unsafe { read_optional_str(reader, row) });

            match tokenizer::split(text, separator, options) {
                Ok(value) => unsafe { out.write_varchar(row, &value) },
                Err(message) => {
                    set_error(info, message);
                    return;
                }
            }
        }
    }

    unsafe fn write_query_result(
        input: duckdb_data_chunk,
        output: duckdb_vector,
        info: duckdb_function_info,
        argc: usize,
        operator: &str,
    ) {
        let text = unsafe { VectorReader::new(input, 0) };
        let options = (argc >= 2).then(|| unsafe { VectorReader::new(input, 1) });
        let mut out = unsafe { VectorWriter::new(output) };

        for row in 0..text.row_count() {
            let Some(text) = (unsafe { read_optional_str(&text, row) }) else {
                unsafe { out.set_null(row) };
                continue;
            };
            let options = options
                .as_ref()
                .and_then(|reader| unsafe { read_optional_str(reader, row) });

            match tokenizer::fts_query(text, options, operator) {
                Ok(value) => unsafe { out.write_varchar(row, &value) },
                Err(message) => {
                    set_error(info, message);
                    return;
                }
            }
        }
    }

    quack_rs::scalar_callback!(vaporetto_split_1, |info, input, output| {
        unsafe { write_split_result(input, output, info, 1) };
    });

    quack_rs::scalar_callback!(vaporetto_split_2, |info, input, output| {
        unsafe { write_split_result(input, output, info, 2) };
    });

    quack_rs::scalar_callback!(vaporetto_split_3, |info, input, output| {
        unsafe { write_split_result(input, output, info, 3) };
    });

    quack_rs::scalar_callback!(vaporetto_and_query_1, |info, input, output| {
        unsafe { write_query_result(input, output, info, 1, "AND") };
    });

    quack_rs::scalar_callback!(vaporetto_and_query_2, |info, input, output| {
        unsafe { write_query_result(input, output, info, 2, "AND") };
    });

    quack_rs::scalar_callback!(vaporetto_or_query_1, |info, input, output| {
        unsafe { write_query_result(input, output, info, 1, "OR") };
    });

    quack_rs::scalar_callback!(vaporetto_or_query_2, |info, input, output| {
        unsafe { write_query_result(input, output, info, 2, "OR") };
    });

    fn varchar_function(
        name: &str,
        argc: usize,
        function: quack_rs::scalar::builder::ScalarFn,
    ) -> ExtResult<ScalarFunctionBuilder> {
        let mut builder = ScalarFunctionBuilder::try_new(name)?;
        for _ in 0..argc {
            builder = builder.param(TypeId::Varchar);
        }
        Ok(builder.returns(TypeId::Varchar).function(function))
    }

    fn register_functions(con: &Connection) -> ExtResult<()> {
        unsafe {
            con.register_scalar(varchar_function("vaporetto_split", 1, vaporetto_split_1)?)?;
            con.register_scalar(varchar_function("vaporetto_split", 2, vaporetto_split_2)?)?;
            con.register_scalar(varchar_function("vaporetto_split", 3, vaporetto_split_3)?)?;
            con.register_scalar(varchar_function(
                "vaporetto_and_query",
                1,
                vaporetto_and_query_1,
            )?)?;
            con.register_scalar(varchar_function(
                "vaporetto_and_query",
                2,
                vaporetto_and_query_2,
            )?)?;
            con.register_scalar(varchar_function(
                "vaporetto_or_query",
                1,
                vaporetto_or_query_1,
            )?)?;
            con.register_scalar(varchar_function(
                "vaporetto_or_query",
                2,
                vaporetto_or_query_2,
            )?)?;
        }
        Ok(())
    }

    quack_rs::entry_point_v2!(duckdb_vaporetto_init_c_api, register_functions);
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::ffi::CString;
    use std::os::raw::c_char;
    use std::ptr;
    use std::slice;

    use libduckdb_sys::{
        DUCKDB_TYPE_DUCKDB_TYPE_VARCHAR, DuckDBSuccess, duckdb_add_scalar_function_to_set,
        duckdb_connect, duckdb_connection, duckdb_create_logical_type,
        duckdb_create_scalar_function, duckdb_create_scalar_function_set, duckdb_data_chunk,
        duckdb_data_chunk_get_size, duckdb_data_chunk_get_vector, duckdb_destroy_logical_type,
        duckdb_destroy_scalar_function, duckdb_destroy_scalar_function_set, duckdb_disconnect,
        duckdb_extension_access, duckdb_extension_info, duckdb_function_info, duckdb_logical_type,
        duckdb_register_scalar_function_set, duckdb_rs_extension_api_init, duckdb_scalar_function,
        duckdb_scalar_function_add_parameter, duckdb_scalar_function_set_error,
        duckdb_scalar_function_set_function, duckdb_scalar_function_set_name,
        duckdb_scalar_function_set_return_type, duckdb_scalar_function_t, duckdb_string_t,
        duckdb_string_t_data, duckdb_validity_row_is_valid, duckdb_validity_set_row_invalid,
        duckdb_vector, duckdb_vector_assign_string_element_len,
        duckdb_vector_ensure_validity_writable, duckdb_vector_get_data, duckdb_vector_get_validity,
    };

    use crate::tokenizer;

    const DUCKDB_API_VERSION: &str = "v1.2.0";
    const DUCKDB_STRING_INLINE_MAX_LEN: usize = 12;

    // DuckDB-Wasm's main module exports a JavaScript-legalized lseek symbol,
    // but Rust/Emscripten side modules import the raw i64 ABI. Keep libc's
    // unused seek path local so dlopen can link the extension.
    #[unsafe(no_mangle)]
    pub extern "C" fn lseek(_fd: i32, _offset: i64, _whence: i32) -> i64 {
        -1
    }

    struct VarcharVector {
        vector: duckdb_vector,
        data: *mut duckdb_string_t,
        validity: *mut u64,
    }

    impl VarcharVector {
        unsafe fn new(vector: duckdb_vector) -> Self {
            Self {
                vector,
                data: unsafe { duckdb_vector_get_data(vector) as *mut duckdb_string_t },
                validity: unsafe { duckdb_vector_get_validity(vector) },
            }
        }

        unsafe fn read(&self, row: usize) -> Option<&str> {
            if !self.validity.is_null()
                && !unsafe { duckdb_validity_row_is_valid(self.validity, row as _) }
            {
                return None;
            }

            // DuckDB-Wasm's C API helper can expose trailing inline buffer bytes
            // for short strings, so read the inlined representation directly.
            let raw = unsafe { self.data.add(row) } as *const u8;
            let len = unsafe { ptr::read_unaligned(raw as *const u32) } as usize;
            let ptr = if len <= DUCKDB_STRING_INLINE_MAX_LEN {
                unsafe { raw.add(4) }
            } else {
                let mut value = unsafe { *self.data.add(row) };
                (unsafe { duckdb_string_t_data(&mut value) }) as *const u8
            };
            let bytes = unsafe { slice::from_raw_parts(ptr, len) };
            Some(std::str::from_utf8(bytes).unwrap_or(""))
        }

        unsafe fn set_null(&self, row: usize) {
            unsafe { duckdb_vector_ensure_validity_writable(self.vector) };
            let validity = unsafe { duckdb_vector_get_validity(self.vector) };
            unsafe { duckdb_validity_set_row_invalid(validity, row as _) };
        }

        unsafe fn write(&self, row: usize, value: &str) {
            unsafe {
                duckdb_vector_assign_string_element_len(
                    self.vector,
                    row as _,
                    value.as_ptr() as *const c_char,
                    value.len() as _,
                )
            };
        }
    }

    fn c_string_lossy(message: impl AsRef<str>) -> CString {
        let bytes = message
            .as_ref()
            .as_bytes()
            .iter()
            .copied()
            .filter(|byte| *byte != 0)
            .collect::<Vec<_>>();
        CString::new(bytes).expect("NUL bytes removed")
    }

    unsafe fn set_error(info: duckdb_function_info, message: impl AsRef<str>) {
        let message = c_string_lossy(message);
        unsafe { duckdb_scalar_function_set_error(info, message.as_ptr()) };
    }

    unsafe fn write_split_result(
        input: duckdb_data_chunk,
        output: duckdb_vector,
        info: duckdb_function_info,
        argc: usize,
    ) {
        let row_count = unsafe { duckdb_data_chunk_get_size(input) } as usize;
        let text = unsafe { VarcharVector::new(duckdb_data_chunk_get_vector(input, 0)) };
        let separator = (argc >= 2)
            .then(|| unsafe { VarcharVector::new(duckdb_data_chunk_get_vector(input, 1)) });
        let options = (argc >= 3)
            .then(|| unsafe { VarcharVector::new(duckdb_data_chunk_get_vector(input, 2)) });
        let out = unsafe { VarcharVector::new(output) };

        for row in 0..row_count {
            let Some(text) = (unsafe { text.read(row) }) else {
                unsafe { out.set_null(row) };
                continue;
            };
            let separator = separator
                .as_ref()
                .and_then(|reader| unsafe { reader.read(row) })
                .unwrap_or(" ");
            let options = options
                .as_ref()
                .and_then(|reader| unsafe { reader.read(row) });

            match tokenizer::split(text, separator, options) {
                Ok(value) => unsafe { out.write(row, &value) },
                Err(message) => {
                    unsafe { set_error(info, message) };
                    return;
                }
            }
        }
    }

    unsafe fn write_query_result(
        input: duckdb_data_chunk,
        output: duckdb_vector,
        info: duckdb_function_info,
        argc: usize,
        operator: &str,
    ) {
        let row_count = unsafe { duckdb_data_chunk_get_size(input) } as usize;
        let text = unsafe { VarcharVector::new(duckdb_data_chunk_get_vector(input, 0)) };
        let options = (argc >= 2)
            .then(|| unsafe { VarcharVector::new(duckdb_data_chunk_get_vector(input, 1)) });
        let out = unsafe { VarcharVector::new(output) };

        for row in 0..row_count {
            let Some(text) = (unsafe { text.read(row) }) else {
                unsafe { out.set_null(row) };
                continue;
            };
            let options = options
                .as_ref()
                .and_then(|reader| unsafe { reader.read(row) });

            match tokenizer::fts_query(text, options, operator) {
                Ok(value) => unsafe { out.write(row, &value) },
                Err(message) => {
                    unsafe { set_error(info, message) };
                    return;
                }
            }
        }
    }

    unsafe extern "C" fn vaporetto_split_1(
        info: duckdb_function_info,
        input: duckdb_data_chunk,
        output: duckdb_vector,
    ) {
        unsafe { write_split_result(input, output, info, 1) };
    }

    unsafe extern "C" fn vaporetto_split_2(
        info: duckdb_function_info,
        input: duckdb_data_chunk,
        output: duckdb_vector,
    ) {
        unsafe { write_split_result(input, output, info, 2) };
    }

    unsafe extern "C" fn vaporetto_split_3(
        info: duckdb_function_info,
        input: duckdb_data_chunk,
        output: duckdb_vector,
    ) {
        unsafe { write_split_result(input, output, info, 3) };
    }

    unsafe extern "C" fn vaporetto_and_query_1(
        info: duckdb_function_info,
        input: duckdb_data_chunk,
        output: duckdb_vector,
    ) {
        unsafe { write_query_result(input, output, info, 1, "AND") };
    }

    unsafe extern "C" fn vaporetto_and_query_2(
        info: duckdb_function_info,
        input: duckdb_data_chunk,
        output: duckdb_vector,
    ) {
        unsafe { write_query_result(input, output, info, 2, "AND") };
    }

    unsafe extern "C" fn vaporetto_or_query_1(
        info: duckdb_function_info,
        input: duckdb_data_chunk,
        output: duckdb_vector,
    ) {
        unsafe { write_query_result(input, output, info, 1, "OR") };
    }

    unsafe extern "C" fn vaporetto_or_query_2(
        info: duckdb_function_info,
        input: duckdb_data_chunk,
        output: duckdb_vector,
    ) {
        unsafe { write_query_result(input, output, info, 2, "OR") };
    }

    unsafe fn varchar_type() -> duckdb_logical_type {
        unsafe { duckdb_create_logical_type(DUCKDB_TYPE_DUCKDB_TYPE_VARCHAR) }
    }

    unsafe fn add_varchar_parameter(function: duckdb_scalar_function) {
        let mut ty = unsafe { varchar_type() };
        unsafe { duckdb_scalar_function_add_parameter(function, ty) };
        unsafe { duckdb_destroy_logical_type(&mut ty) };
    }

    unsafe fn set_varchar_return_type(function: duckdb_scalar_function) {
        let mut ty = unsafe { varchar_type() };
        unsafe { duckdb_scalar_function_set_return_type(function, ty) };
        unsafe { duckdb_destroy_logical_type(&mut ty) };
    }

    unsafe fn add_overload(
        set: libduckdb_sys::duckdb_scalar_function_set,
        name: &CString,
        argc: usize,
        callback: duckdb_scalar_function_t,
    ) -> Result<(), String> {
        let mut function = unsafe { duckdb_create_scalar_function() };
        if function.is_null() {
            return Err("duckdb_create_scalar_function failed".to_string());
        }

        unsafe { duckdb_scalar_function_set_name(function, name.as_ptr()) };
        for _ in 0..argc {
            unsafe { add_varchar_parameter(function) };
        }
        unsafe { set_varchar_return_type(function) };
        unsafe { duckdb_scalar_function_set_function(function, callback) };

        let result = unsafe { duckdb_add_scalar_function_to_set(set, function) };
        unsafe { duckdb_destroy_scalar_function(&mut function) };

        if result != DuckDBSuccess {
            return Err(format!(
                "duckdb_add_scalar_function_to_set failed for {}({argc})",
                name.to_string_lossy()
            ));
        }
        Ok(())
    }

    unsafe fn register_function_set(
        con: duckdb_connection,
        name: &str,
        overloads: &[(usize, duckdb_scalar_function_t)],
    ) -> Result<(), String> {
        let name = CString::new(name).expect("function name has no NUL");
        let mut set = unsafe { duckdb_create_scalar_function_set(name.as_ptr()) };
        if set.is_null() {
            return Err(format!(
                "duckdb_create_scalar_function_set failed for {}",
                name.to_string_lossy()
            ));
        }

        for (argc, callback) in overloads {
            if let Err(error) = unsafe { add_overload(set, &name, *argc, *callback) } {
                unsafe { duckdb_destroy_scalar_function_set(&mut set) };
                return Err(error);
            }
        }

        let result = unsafe { duckdb_register_scalar_function_set(con, set) };
        unsafe { duckdb_destroy_scalar_function_set(&mut set) };

        if result != DuckDBSuccess {
            return Err(format!(
                "duckdb_register_scalar_function_set failed for {}",
                name.to_string_lossy()
            ));
        }
        Ok(())
    }

    unsafe fn register_functions(con: duckdb_connection) -> Result<(), String> {
        unsafe {
            register_function_set(
                con,
                "vaporetto_split",
                &[
                    (1, Some(vaporetto_split_1)),
                    (2, Some(vaporetto_split_2)),
                    (3, Some(vaporetto_split_3)),
                ],
            )?;
            register_function_set(
                con,
                "vaporetto_and_query",
                &[
                    (1, Some(vaporetto_and_query_1)),
                    (2, Some(vaporetto_and_query_2)),
                ],
            )?;
            register_function_set(
                con,
                "vaporetto_or_query",
                &[
                    (1, Some(vaporetto_or_query_1)),
                    (2, Some(vaporetto_or_query_2)),
                ],
            )?;
        }
        Ok(())
    }

    unsafe fn report_init_error(
        info: duckdb_extension_info,
        access: *const duckdb_extension_access,
        message: impl AsRef<str>,
    ) {
        if access.is_null() {
            return;
        }
        if let Some(set_error) = unsafe { (*access).set_error } {
            let message = c_string_lossy(message);
            unsafe { set_error(info, message.as_ptr()) };
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn duckdb_vaporetto_init_c_api(
        info: duckdb_extension_info,
        access: *const duckdb_extension_access,
    ) -> bool {
        match unsafe { duckdb_rs_extension_api_init(info, access, DUCKDB_API_VERSION) } {
            Ok(true) => {}
            Ok(false) => return false,
            Err(message) => {
                unsafe { report_init_error(info, access, message) };
                return false;
            }
        }

        let Some(get_database) = (unsafe { (*access).get_database }) else {
            unsafe { report_init_error(info, access, "get_database function pointer is null") };
            return false;
        };

        let db = unsafe { *get_database(info) };
        let mut con: duckdb_connection = ptr::null_mut();
        if unsafe { duckdb_connect(db, &mut con) } != DuckDBSuccess {
            unsafe {
                report_init_error(
                    info,
                    access,
                    "duckdb_connect failed during extension initialization",
                )
            };
            return false;
        }

        let result = unsafe { register_functions(con) };
        unsafe { duckdb_disconnect(&mut con) };

        if let Err(message) = result {
            unsafe { report_init_error(info, access, message) };
            return false;
        }

        true
    }
}

pub use tokenizer::{and_query, or_query, scalar_tokens, split};
