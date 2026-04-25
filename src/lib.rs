mod tokenizer;

use libduckdb_sys::{duckdb_data_chunk, duckdb_function_info, duckdb_vector};
use quack_rs::prelude::*;

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

pub use tokenizer::{and_query, or_query, scalar_tokens, split};
