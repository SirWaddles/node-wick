#[macro_use]
extern crate neon;
extern crate neon_serde;
extern crate john_wick_parse;

use neon::prelude::*;
use std::fs;
use std::io::Write;
use std::cell::Cell;
use john_wick_parse::{assets, read_texture as read_texture_asset};
use john_wick_parse::archives::PakExtractor;

fn parse_err(err: assets::ParserError) -> String {
    err.get_properties().into_iter().rev().fold(String::new(), |acc, v| acc + "\n" + v)
}

fn get_buffer_contents(cx: &mut MethodContext<JsUndefined>, buffer: Handle<JsBuffer>) -> Vec<u8> {
    let guard = cx.lock();
    let contents = buffer.borrow(&guard);
    let slice = contents.as_slice();
    let mut buffer_data = vec![0u8; slice.len()];
    buffer_data.as_mut_slice().copy_from_slice(slice);

    buffer_data
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

fn read_pak_key(mut cx: FunctionContext) -> JsResult<JsString> {
    let asset_path = cx.argument::<JsString>(0)?.value();
    let header = match PakExtractor::new_header(&asset_path) {
        Ok(data) => data,
        Err(err) => return cx.throw_error(parse_err(err)),
    };

    Ok(JsString::new(&mut cx, header.get_key_guid().to_string()))
}

pub struct Package {
    package: Cell<assets::Package>,
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

        method get_mount_point(mut cx) {
            let this = cx.this();
            let mount_point = {
                let guard = cx.lock();
                let extractor = this.borrow(&guard);
                extractor.get_mount_point().to_owned()
            };
            Ok(JsString::new(&mut cx, mount_point).upcast())
        }
    }

    pub class JsPackage for Package {
        init(mut cx) {
            match cx.len() {
                1 => {
                    let asset_path = cx.argument::<JsString>(0)?.value();
                    let package = match assets::Package::from_file(&asset_path) {
                        Ok(data) => data,
                        Err(err) => return cx.throw_error(parse_err(err)),
                    };
                    Ok(Package {
                        package: Cell::new(package),
                    })
                },
                _ => {
                    let uasset_js = cx.argument::<JsBuffer>(0)?;
                    let uexp_js = cx.argument::<JsBuffer>(1)?;
                    let uasset = get_buffer_contents(&mut cx, uasset_js);
                    let uexp = get_buffer_contents(&mut cx, uexp_js);

                    let ubulk_js = match cx.argument_opt(2) {
                        Some(arg) => Some(arg.downcast_or_throw(&mut cx)?),
                        None => None,
                    };
                    let ubulk = match ubulk_js {
                        Some(buffer) => Some(get_buffer_contents(&mut cx, buffer)),
                        None => None,
                    };

                    let package = match assets::Package::from_buffer(uasset, uexp, ubulk) {
                        Ok(data) => data,
                        Err(err) => return cx.throw_error(parse_err(err)),
                    };
                    Ok(Package {
                        package: Cell::new(package),
                    })
                }
            }
        }

        method get_data(mut cx) {
            /*
             * Okay so this is stupid
             * 
             * Neon's ref locks the internal value quite a bit. I need a mutable borrow on the CallContext in order
             * to generate the JS serialization. According to the documentation the solution seems to be "just clone it lol"
             * Interior mutability seemed like the only solution, but since no reference can outlast the lock, that leaves
             * std::cell::Cell to the rescue. But I needed something to put back in the Cell while I'm using it. 
             * Hence the incredibly dumb ::empty function
             * 
             * If there's a far better way of doing this that I'm being ignorant of, please let me know.
             */
            let this = cx.this();
            let package = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                data.package.replace(assets::Package::empty())
            };
            let js_data = neon_serde::to_value(&mut cx, &package)?;
            {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                data.package.set(package);
            }
            Ok(js_data)
        }

        method get_texture(mut cx) {
            let this = cx.this();
            let package = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                data.package.replace(assets::Package::empty())
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
    }
}

register_module!(mut cx, {
    cx.export_function("read_texture_to_file", read_texture_to_file)?;
    cx.export_function("read_pak_key", read_pak_key)?;
    cx.export_class::<JsPakExtractor>("PakExtractor")?;
    cx.export_class::<JsPackage>("Package")?;
    Ok(())
});
