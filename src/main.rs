// Attribution: spacedrive
// https://github.com/spacedriveapp/spacedrive/tree/main/crates/file-ext
pub mod extensions;
pub mod kind;
pub mod magic;

use crate::{extensions::Extension, magic::MagicBytes, magic::MagicBytesMeta};
use home::home_dir;
use nu_plugin::{serve_plugin, EvaluatedCall, LabeledError, MsgPackSerializer, Plugin};
use nu_protocol::{Category, PluginExample, PluginSignature, Span, Spanned, SyntaxShape, Value};
use std::path::Path;

struct Implementation;

impl Implementation {
    fn new() -> Self {
        Self {}
    }
}

impl Plugin for Implementation {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("file")
            .usage("View file format information")
            .required(
                "filename",
                SyntaxShape::String,
                "full path to file name to inspect",
            )
            .category(Category::Experimental)
            .plugin_examples(vec![PluginExample {
                description: "Get format information from file".into(),
                example: "file some.jpg".into(),
                result: Some(Value::Record {
                    cols: vec![
                        "description".into(),
                        "format".into(),
                        "magic_offset".into(),
                        "magic_length".into(),
                        "magic_bytes".into(),
                    ],
                    vals: vec![
                        Value::string("Image", Span::test_data()),
                        Value::string("jpg", Span::test_data()),
                        Value::string("0", Span::test_data()),
                        Value::string("2", Span::test_data()),
                        Value::string("[FF, D8]", Span::test_data()),
                    ],
                    span: Span::test_data(),
                }),
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
            let canon_path = Path::new(&filename).canonicalize().unwrap();
            let file_format = extensions::Extension::resolve_conflicting(canon_path, true);
            return match file_format {
                Some(file_format) => match file_format {
                    Extension::Document(document_format) => {
                        let magic = document_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Document",
                            document_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Video(video_format) => {
                        let magic = video_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Video",
                            video_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Image(image_format) => {
                        let magic = image_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Image",
                            image_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Audio(audio_format) => {
                        let magic = audio_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Audio",
                            audio_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Archive(archive_format) => {
                        let magic = archive_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Archive",
                            archive_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Executable(executable_format) => {
                        let magic = executable_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Executable",
                            executable_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Text(text_format) => {
                        return Ok(get_text_format_details(
                            "Text",
                            text_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Encrypted(encrypted_format) => {
                        let magic = encrypted_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Encrypted",
                            encrypted_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Key(key_format) => {
                        return Ok(get_text_format_details("Key", key_format.to_string(), span));
                    }
                    Extension::Font(font_format) => {
                        let magic = font_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Font",
                            font_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Mesh(mesh_format) => {
                        let magic = mesh_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Mesh",
                            mesh_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Code(code_format) => {
                        return Ok(get_text_format_details(
                            "Code",
                            code_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Database(database_format) => {
                        let magic = database_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Database",
                            database_format.to_string(),
                            span,
                        ));
                    }
                    Extension::Book(book_format) => {
                        let magic = book_format.magic_bytes_meta();
                        return Ok(get_magic_details(
                            magic,
                            "Book",
                            book_format.to_string(),
                            span,
                        ));
                    }
                },
                None => Ok(Value::Nothing { span: call.head }),
            };
        }

        Ok(Value::Nothing { span: call.head })
    }
}

fn get_magic_details(
    magic: Vec<MagicBytesMeta>,
    format: &str,
    data_format: String,
    span: Span,
) -> Value {
    // let mut details = vec![];
    // details.push(Value::string(format, magic.span));
    // details.push(Value::string(magic.format, magic.span));
    // details.push(Value::string(magic.offset.to_string(), magic.span));
    // details.push(Value::string(magic.length.to_string(), magic.span));
    // details.push(Value::string(format!("{:X?}", magic.bytes), magic.span));
    // details
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
        Value::string(format, span),
        Value::string(data_format, span),
        Value::string(offsets.join(", "), span),
        Value::string(lengths.join(", "), span),
        Value::string(format!("{}", mbytes.join(", ")), span),
    ];

    Value::Record { cols, vals, span }
}

fn get_text_format_details(format: &str, text_format: String, span: Span) -> Value {
    let cols = vec![
        "description".into(),
        "format".into(),
        "magic_offset".into(),
        "magic_length".into(),
        "magic_bytes".into(),
    ];

    Value::Record {
        cols,
        vals: vec![
            Value::string(format, span),
            Value::string(text_format, span),
            Value::nothing(span),
            Value::nothing(span),
            Value::nothing(span),
        ],
        span,
    }
}

fn main() {
    serve_plugin(&mut Implementation::new(), MsgPackSerializer);
}
