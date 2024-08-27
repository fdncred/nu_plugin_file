// Attribution: spacedrive
// https://github.com/spacedriveapp/spacedrive/tree/main/crates/file-ext
pub mod extensions;
pub mod kind;
pub mod magic;
#[cfg(feature = "executables")]
pub mod executable;

use crate::{extensions::Extension, magic::MagicBytes, magic::MagicBytesMeta};
use home::home_dir;
use nu_plugin::{
    serve_plugin, EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand,
    SimplePluginCommand,
};
use nu_protocol::{
    record, Category, Example, LabeledError, Signature, Span, Spanned, SyntaxShape, Value,
};
use std::path::Path;

struct FilePlugin;

impl Plugin for FilePlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(Implementation)]
    }
}

fn is_windows_absolute_path(path: &str) -> bool {
    path.starts_with(r"\\") || path.chars().skip_while(|c| c.is_alphabetic()).take(1).eq(":".chars())
}

struct Implementation;

impl SimplePluginCommand for Implementation {
    type Plugin = FilePlugin;

    fn name(&self) -> &str {
        "file"
    }

    fn description(&self) -> &str {
        "View file format information"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .required(
                "filename",
                SyntaxShape::String,
                "full path to file name to inspect",
            )
            .category(Category::Experimental)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Get format information from file",
            example: "file some.jpg",
            result: Some(Value::test_record(record!(
                        "description" => Value::test_string("Image"),
                        "format" => Value::test_string("jpg"),
                        "magic_offset" => Value::test_string("0"),
                        "magic_length" => Value::test_string("2"),
                        "magic_bytes" => Value::test_string("[FF, D8]")))),
        }]
    }
    fn run(
        &self,
        _plugin: &FilePlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let param: Option<Spanned<String>> = call.opt(0)?;
        let Some(filename) = param else {
            return Ok(Value::nothing(call.head));
        };
        let span = filename.span;
        
        let filename = if filename.item.starts_with('~') {
            let home_dir = match home_dir() {
                Some(path) => path,
                None => {
                    return Err(LabeledError::new("Cannot find home directory")
                        .with_label("Cannot find home directory", call.head))
                }
            };
            let Some(home_dir) = home_dir.to_str() else {
                return Err(
                    LabeledError::new("Cannot convert home directory to valid UTF-8")
                        .with_label("Cannot convert home directory to valid UTF-8", span),
                );
            };
            filename.item.replace('~', home_dir)
        } else if (cfg!(target_family = "unix") && filename.item.starts_with('/'))
                  || (cfg!(target_family = "windows") && is_windows_absolute_path(&filename.item)) {
            filename.item
        } else {
            match engine.get_current_dir() {
                Ok(dir) => dir.to_string() + std::path::MAIN_SEPARATOR_STR + &filename.item,
                Err(e) => {
                    return Err(LabeledError::new(e.to_string()).with_label(e.to_string(), span))
                }
            }
        };

        let canon_path = match Path::new(&filename).canonicalize() {
            Ok(path) => path,
            Err(e) => return Err(LabeledError::new(e.to_string()).with_label(e.to_string(), span)),
        };
        let file_format = extensions::Extension::resolve_conflicting(canon_path, true);

        match file_format {
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
            None => Ok(Value::nothing(call.head)),
        }
    }
}

fn get_magic_details(
    magic: Vec<MagicBytesMeta>,
    format: &str,
    data_format: String,
    span: Span,
) -> Value {
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

    Value::record(
        record!(
        "description" => Value::string(format, span),
        "format" => Value::string(data_format, span),
        "magic_offset" => Value::string(offsets.join(", "), span),
        "magic_length" => Value::string(lengths.join(", "), span),
        "magic_bytes" => Value::string(mbytes.join(", "), span),
        ),
        span,
    )
}

fn get_text_format_details(format: &str, text_format: String, span: Span) -> Value {
    Value::record(
        record!(
        "description" => Value::string(format, span),
        "format" => Value::string(text_format, span),
        "magic_offset" => Value::nothing(span),
        "magic_length" => Value::nothing(span),
        "magic_bytes" => Value::nothing(span),
        ),
        span,
    )
}

fn main() {
    serve_plugin(&FilePlugin, MsgPackSerializer);
}
