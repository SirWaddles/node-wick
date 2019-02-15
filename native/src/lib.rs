#[macro_use]
extern crate neon;
extern crate neon_serde;
extern crate john_wick_parse;

use neon::prelude::*;
use std::fs;
use std::io::Write;
use john_wick_parse::{assets, read_texture as read_texture_asset};
use john_wick_parse::archives::PakExtractor;

fn parse_err(err: assets::ParserError) -> String {
    err.get_properties().into_iter().rev().fold(String::new(), |acc, v| acc + "\n" + v)
}

fn read_asset(mut cx: FunctionContext) -> JsResult<JsValue> {
    let asset_path = cx.argument::<JsString>(0)?.value();
    let package = match assets::Package::from_file(&asset_path) {
        Ok(data) => data,
        Err(err) => return cx.throw_error(parse_err(err)),
    };

    let js_asset = neon_serde::to_value(&mut cx, &package)?;
    Ok(js_asset)
}

fn read_texture(mut cx: FunctionContext) -> JsResult<JsValue> {
    let asset_path = cx.argument::<JsString>(0)?.value();
    let package = match assets::Package::from_file(&asset_path) {
        Ok(data) => data,
        Err(err) => return cx.throw_error(parse_err(err)),
    };

    let texture_data = match read_texture_asset(package) {
        Ok(data) => data,
        Err(err) => return cx.throw_error(parse_err(err)),
    };

    let tex_buffer = {
        let mut buffer = JsBuffer::new(&mut cx, texture_data.len() as u32)?;
        let guard = cx.lock();
        let mut contents = buffer.borrow(&guard);
        let mut slice = contents.as_mut_slice();
        slice.copy_from_slice(&texture_data);
        buffer
    };

    Ok(tex_buffer.upcast())
}

fn read_texture_to_file(mut cx: FunctionContext) -> JsResult<JsValue> {
    let asset_path = cx.argument::<JsString>(0)?.value();
    let package = match assets::Package::from_file(&asset_path) {
        Ok(data) => data,
        Err(err) => return cx.throw_error(parse_err(err)),
    };

    let texture_path = cx.argument::<JsString>(1)?.value();

    let texture_data = match read_texture_asset(package) {
        Ok(data) => data,
        Err(err) => return cx.throw_error(parse_err(err)),
    };

    let mut file = fs::File::create(texture_path).unwrap();
    file.write_all(&texture_data).unwrap();
    Ok(JsBoolean::new(&mut cx, true).upcast())
}

declare_types! {
    pub class JsPakExtractor for PakExtractor {
        init(mut cx) {
            let asset_path = cx.argument::<JsString>(0)?.value();
            let key = cx.argument::<JsString>(1)?.value();
            let extractor = match PakExtractor::new(&asset_path, &key) {
                Ok(data) => data,
                Err(err) => return cx.throw_error(parse_err(err)),
            };

            Ok(extractor)
        }

        method get_file_list(mut cx) {
            let this = cx.this();
            let file_list: Vec<String> = {
                let guard = cx.lock();
                let extractor = this.borrow(&guard);
                extractor.get_entries().into_iter().map(|v| v.get_filename().to_owned()).collect()
            };
            let js_entries = JsArray::new(&mut cx, file_list.len() as u32);
            for (i, obj) in file_list.iter().enumerate() {
                let js_string = cx.string(obj);
                js_entries.set(&mut cx, i as u32, js_string).unwrap();
            }
            
            Ok(js_entries.upcast())
        }

        method get_file(mut cx) {
            let mut this = cx.this();
            let file_index = cx.argument::<JsNumber>(0)?.value() as usize;
            let file: Vec<u8> = {
                let guard = cx.lock();
                let mut extractor = this.borrow_mut(&guard);
                let file = extractor.get_entries().get(file_index).unwrap().clone();
                extractor.get_file(&file)
            };
            let js_buffer = {
                let mut buffer = JsBuffer::new(&mut cx, file.len() as u32)?;
                let guard = cx.lock();
                let mut contents = buffer.borrow(&guard);
                let mut slice = contents.as_mut_slice();
                slice.copy_from_slice(&file);
                buffer
            };
            
            Ok(js_buffer.upcast())
        }
    }
}

register_module!(mut cx, {
    cx.export_function("read_asset", read_asset)?;
    cx.export_function("read_texture", read_texture)?;
    cx.export_function("read_texture_to_file", read_texture_to_file)?;
    cx.export_class::<JsPakExtractor>("PakExtractor")?;
    Ok(())
});
