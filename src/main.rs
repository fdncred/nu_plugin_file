// Attribution: spacedrive
// https://github.com/spacedriveapp/spacedrive/tree/main/crates/file-ext
pub mod extensions;
// mod file;
pub mod kind;
pub mod magic;

use home::home_dir;
use nu_plugin::{serve_plugin, EvaluatedCall, LabeledError, MsgPackSerializer, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Spanned, SyntaxShape, Value};
use std::path::Path;

use crate::{extensions::Extension, magic::MagicBytes};
// use tokio::runtime::Handle;

struct Implementation;

impl Implementation {
    fn new() -> Self {
        Self {}
    }
}

impl Plugin for Implementation {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("file")
            .usage("View file results")
            .required("path", SyntaxShape::String, "path to file input file")
            .category(Category::Experimental)
            .plugin_examples(vec![PluginExample {
                description: "This is the example descripion".into(),
                example: "some pipeline involving file".into(),
                result: None,
            }])]
    }

    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        assert_eq!(name, "file");
        let param: Option<Spanned<String>> = call.opt(0)?;
        // eprintln!("starting run");

        if let Some(filename) = param {
            let home_dir = match home_dir() {
                Some(path) => path,
                None => {
                    return Err(LabeledError {
                        label: "Could not find home directory".into(),
                        msg: "Could not find home directory".into(),
                        span: Some(call.head),
                    });
                }
            };
            let span = filename.span;
            let filename = if filename.item.starts_with("~") {
                filename.item.replace("~", home_dir.to_str().unwrap())
            } else {
                filename.item
            };
            // eprintln!("filename: {:?}", &filename);
            let canon_path = Path::new(&filename).canonicalize().unwrap();
            // eprintln!("canon_path: {:?}", &canon_path);
            let file_format = extensions::Extension::resolve_conflicting(canon_path, true);
            // eprintln!("file_format: {:?}", file_format);
            return match file_format {
                Some(file_format) => {
                    match file_format {
                        Extension::Document(document_format) => {
                            return Ok(Value::string(document_format.to_string(), span))
                        }
                        Extension::Video(video_format) => {
                            return Ok(Value::string(video_format.to_string(), span))
                        }
                        Extension::Image(image_format) => {
                            // let f = image_format[jpg];
                            let magic = image_format.magic_bytes_meta();
                            // return Ok(Value::string(
                            //     format!("Image Format: {image_format} Bytes: {bytes:?}"),
                            //     span,
                            // ));
                            let offsets = magic
                                .iter()
                                .map(|b| b.offset.to_string())
                                .collect::<Vec<_>>();
                            let lengths = magic
                                .iter()
                                .map(|b| b.length.to_string())
                                .collect::<Vec<_>>();
                            let mbytes = magic
                                .iter()
                                .map(|b| format!("{:X?}", b.bytes.clone()))
                                .collect::<Vec<_>>();
                            let cols = vec![
                                "description".into(),
                                "format".into(),
                                "magic_offset".into(),
                                "magic_length".into(),
                                "magic_bytes".into(),
                            ];
                            let vals = vec![
                                Value::string("Image", span),
                                Value::string(format!("{image_format}"), span),
                                Value::string(offsets.join(", "), span),
                                Value::string(lengths.join(", "), span),
                                Value::string(format!("{}", mbytes.join(", ")), span),
                            ];
                            return Ok(Value::Record { cols, vals, span });
                        }
                        Extension::Audio(audio_format) => {
                            return Ok(Value::string(audio_format.to_string(), span))
                        }
                        Extension::Archive(archive_format) => {
                            return Ok(Value::string(archive_format.to_string(), span))
                        }
                        Extension::Executable(executable_format) => {
                            return Ok(Value::string(executable_format.to_string(), span))
                        }
                        Extension::Text(text_format) => {
                            return Ok(Value::string(text_format.to_string(), span))
                        }
                        Extension::Encrypted(encrypted_format) => {
                            return Ok(Value::string(encrypted_format.to_string(), span))
                        }
                        Extension::Key(key_format) => {
                            return Ok(Value::string(key_format.to_string(), span))
                        }
                        Extension::Font(font_format) => {
                            return Ok(Value::string(font_format.to_string(), span))
                        }
                        Extension::Mesh(mesh_format) => {
                            return Ok(Value::string(mesh_format.to_string(), span))
                        }
                        Extension::Code(code_format) => {
                            return Ok(Value::string(code_format.to_string(), span))
                        }
                        Extension::Database(database_format) => {
                            return Ok(Value::string(database_format.to_string(), span))
                        }
                        Extension::Book(book_format) => {
                            return Ok(Value::string(book_format.to_string(), span))
                        }
                    }
                    // Ok(Value::string(file_format.to_string(), span))
                }
                None => Ok(Value::Nothing { span: call.head }),
            };
        }

        Ok(Value::Nothing { span: call.head })
    }
}

fn main() {
    serve_plugin(&mut Implementation::new(), MsgPackSerializer);
}
