use neon::prelude::*;
use std::fs;
use std::io::Write;
use std::cell::Cell;
use john_wick_parse::{assets, read_asset, read_asset_from_file, read_texture as read_texture_asset, read_sound};
use john_wick_parse::dispatch::Extractor;

fn parse_err(err: assets::ParserError) -> String {
    err.get_properties().into_iter().rev().fold(String::new(), |acc, v| acc + "\n" + v)
}

fn get_buffer_contents_fn(cx: &mut FunctionContext, buffer: Handle<JsBuffer>) -> Vec<u8> {
    let guard = cx.lock();
    let contents = buffer.borrow(&guard);
    let slice = contents.as_slice();
    let mut buffer_data = vec![0u8; slice.len()];
    buffer_data.as_mut_slice().copy_from_slice(slice);

    buffer_data
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
    let package = match read_asset_from_file(&asset_path) {
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
    let header = match Extractor::new_header(&asset_path) {
        Ok(data) => data,
        Err(err) => return cx.throw_error(parse_err(err)),
    };

    Ok(JsString::new(&mut cx, header.get_key_guid().to_string()))
}

fn read_locale(mut cx: FunctionContext) -> JsResult<JsValue> {
    let locres_js = cx.argument::<JsBuffer>(0)?;
    let locale_data = get_buffer_contents_fn(&mut cx, locres_js);
    let package = match assets::locale::FTextLocalizationResource::from_buffer(&locale_data) {
        Ok(data) => data,
        Err(err) => return cx.throw_error(parse_err(err)),
    };
    let js_data = neon_serde::to_value(&mut cx, &package)?;
    Ok(js_data)
}

pub struct Package {
    package: Cell<assets::Package>,
}

declare_types! {
    pub class JsExtractor for Extractor {
        init(mut cx) {
            let asset_path = cx.argument::<JsString>(0)?.value();
            let key = cx.argument::<JsString>(1)?.value();
            let extractor = match Extractor::new(&asset_path, Some(&key)) {
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
                extractor.get_file_list().clone()
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
            let file_index = cx.argument::<JsString>(0)?.value();
            let file: Vec<u8> = {
                let guard = cx.lock();
                let mut extractor = this.borrow_mut(&guard);
                extractor.get_file(&file_index).unwrap()
            };
            let js_buffer = {
                let buffer = JsBuffer::new(&mut cx, file.len() as u32)?;
                let guard = cx.lock();
                let contents = buffer.borrow(&guard);
                let slice = contents.as_mut_slice();
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
            let arg = cx.argument::<JsValue>(0)?;
            if arg.is_a::<JsString>() {
                let asset_path = cx.argument::<JsString>(0)?.value();
                let package = match read_asset_from_file(&asset_path) {
                    Ok(data) => data,
                    Err(err) => return cx.throw_error(parse_err(err)),
                };
                Ok(Package {
                    package: Cell::new(package),
                })
            } else if arg.is_a::<JsBuffer>() {
                let uasset_js = cx.argument::<JsBuffer>(0)?;
                let uasset = get_buffer_contents(&mut cx, uasset_js);

                let ubulk_js = match cx.argument_opt(1) {
                    Some(arg) => {
                        if arg.is_a::<JsBuffer>() {
                            let buf_ref = arg.downcast_or_throw(&mut cx)?;
                            Some(get_buffer_contents(&mut cx, buf_ref))
                        } else {
                            None
                        }
                    },
                    None => None,
                };

                let package = match read_asset(&uasset, match ubulk_js { Some(ref a) => Some(a.as_slice()), None => None}) {
                    Ok(data) => data,
                    Err(err) => return cx.throw_error(parse_err(err)),
                };
                Ok(Package {
                    package: Cell::new(package),
                })
            } else {
                cx.throw_error(format!("Incorrect Type"))
            }
        }

        method get_data(mut cx) {
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
                let buffer = JsBuffer::new(&mut cx, texture_data.len() as u32)?;
                let guard = cx.lock();
                let contents = buffer.borrow(&guard);
                let slice = contents.as_mut_slice();
                slice.copy_from_slice(&texture_data);
                buffer
            };

            Ok(tex_buffer.upcast())
        }

        method get_sound(mut cx) {
            let this = cx.this();
            let package = {
                let guard = cx.lock();
                let data = this.borrow(&guard);
                data.package.replace(assets::Package::empty())
            };

            let sound_data = match read_sound(package) {
                Ok(data) => data,
                Err(err) => return cx.throw_error(parse_err(err)),
            };

            let s_buffer = {
                let buffer = JsBuffer::new(&mut cx, sound_data.len() as u32)?;
                let guard = cx.lock();
                let contents = buffer.borrow(&guard);
                let slice = contents.as_mut_slice();
                slice.copy_from_slice(&sound_data);
                buffer
            };

            Ok(s_buffer.upcast())
        }
    }
}

register_module!(mut cx, {
    cx.export_function("read_texture_to_file", read_texture_to_file)?;
    cx.export_function("read_pak_key", read_pak_key)?;
    cx.export_function("read_locale", read_locale)?;
    cx.export_class::<JsExtractor>("Extractor")?;
    cx.export_class::<JsPackage>("Package")?;
    Ok(())
});
