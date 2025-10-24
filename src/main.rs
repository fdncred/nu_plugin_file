// Attribution: spacedrive
// https://github.com/spacedriveapp/spacedrive/tree/main/crates/file-ext
#[cfg(feature = "executables")]
pub mod executable;
pub mod extensions;
pub mod kind;
pub mod magic;
#[cfg(feature = "executables")]
use executable::BinaryArch;

use crate::{extensions::Extension, magic::MagicBytes, magic::MagicBytesMeta};

use home::home_dir;
use nu_plugin::{
    EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand, SimplePluginCommand,
    serve_plugin,
};
use nu_protocol::{
    Category, Example, LabeledError, Signature, Span, Spanned, SyntaxShape, Value, record,
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
    path.starts_with(r"\\")
        || path
            .chars()
            .skip_while(|c| c.is_alphabetic())
            .take(1)
            .eq(":".chars())
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
                SyntaxShape::Filepath,
                "full path to file name to inspect",
            )
            .category(Category::Experimental)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            description: "Get format information from file",
            example: "file some.jpg",
            result: Some(Value::test_record(record!(
                        "description" => Value::test_string("Image"),
                        "format" => Value::test_string("jpg"),
                        "mime" => Value::test_string("image/jpeg"),
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
                        .with_label("Cannot find home directory", call.head));
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
            || (cfg!(target_family = "windows") && is_windows_absolute_path(&filename.item))
        {
            filename.item
        } else {
            match engine.get_current_dir() {
                Ok(dir) => dir.to_string() + std::path::MAIN_SEPARATOR_STR + &filename.item,
                Err(e) => {
                    return Err(LabeledError::new(e.to_string()).with_label(e.to_string(), span));
                }
            }
        };

        let canon_path = match Path::new(&filename).canonicalize() {
            Ok(path) => path,
            Err(e) => return Err(LabeledError::new(e.to_string()).with_label(e.to_string(), span)),
        };
        let file_format = extensions::Extension::resolve_conflicting(&canon_path, true);
        let mime = infer_mime(&canon_path);

        match file_format {
            Some(file_format) => match file_format {
                Extension::Document(document_format) => {
                    let magic = document_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Document",
                        document_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                Extension::Video(video_format) => {
                    let magic = video_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Video",
                        video_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                Extension::Image(image_format) => {
                    let magic = image_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Image",
                        image_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                Extension::Audio(audio_format) => {
                    let magic = audio_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Audio",
                        audio_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                Extension::Archive(archive_format) => {
                    let magic = archive_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Archive",
                        archive_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                #[cfg(feature = "executables")]
                Extension::Executable(_) => {
                    let bin = crate::executable::Binary::parse(&canon_path).map_err(|e| {
                        LabeledError::new(e.to_string()).with_label(e.to_string(), span)
                    })?;
                    Ok(get_executable_format_details(bin, span, &mime))
                }
                #[cfg(not(feature = "executables"))]
                Extension::Executable(executable_format) => {
                    let magic = executable_format.magic_bytes_meta();
                    return Ok(get_magic_details(
                        magic,
                        "Encrypted",
                        executable_format.to_string(),
                        span,
                        &mime,
                    ));
                }
                Extension::Text(text_format) => Ok(get_text_format_details(
                    "Text",
                    text_format.to_string(),
                    span,
                )),
                Extension::Encrypted(encrypted_format) => {
                    let magic = encrypted_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Encrypted",
                        encrypted_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                Extension::Key(key_format) => {
                    Ok(get_text_format_details("Key", key_format.to_string(), span))
                }
                Extension::Font(font_format) => {
                    let magic = font_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Font",
                        font_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                Extension::Mesh(mesh_format) => {
                    let magic = mesh_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Mesh",
                        mesh_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                Extension::Code(code_format) => Ok(get_text_format_details(
                    "Code",
                    code_format.to_string(),
                    span,
                )),
                Extension::Database(database_format) => {
                    let magic = database_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Database",
                        database_format.to_string(),
                        span,
                        &mime,
                    ))
                }
                Extension::Book(book_format) => {
                    let magic = book_format.magic_bytes_meta();
                    Ok(get_magic_details(
                        magic,
                        "Book",
                        book_format.to_string(),
                        span,
                        &mime,
                    ))
                }
            },
            None => {
                #[cfg(feature = "executables")]
                if executable::Binary::has_magic_bytes(&canon_path) {
                    let bin = crate::executable::Binary::parse(&canon_path).map_err(|e| {
                        LabeledError::new(e.to_string()).with_label(e.to_string(), span)
                    })?;
                    return Ok(get_executable_format_details(bin, span, &mime));
                }
                Ok(Value::record(
                    record!("mime" => Value::string(&mime, call.head)),
                    call.head,
                ))
            }
        }
    }
}

fn infer_mime(path: &Path) -> String {
    if path.is_dir() {
        "inode/directory".to_string()
    } else {
        infer::get_from_path(path)
            .ok()
            .flatten()
            .map(|t| t.mime_type().to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string())
    }
}

#[cfg(feature = "executables")]
fn get_executable_format_details(bin: executable::Binary, span: Span, mime: &str) -> Value {
    let magics = std::iter::once(bin.magic_bytes.as_ref())
        .flatten()
        .chain(
            bin.arches
                .iter()
                .map(|BinaryArch { magic_bytes, .. }| magic_bytes),
        )
        .map(|magic_bytes| {
            Value::record(
                record!(
                    "offset" => Value::int(magic_bytes.offset as _, span),
                    "length" => Value::int(magic_bytes.length as _, span),
                    "bytes" => Value::binary(&magic_bytes.bytes[..], span),
                ),
                span,
            )
        })
        .collect();
    Value::record(
        record!(
        "description" => Value::string(bin.description(), span),
        "format" => Value::string("Executable", span),
        "mime" => Value::string(mime, span),
        "magics" => Value::list(magics, span),
        "details" => bin.into_value(span),
        ),
        span,
    )
}

fn get_magic_details(
    magic: Vec<MagicBytesMeta>,
    format: &str,
    data_format: String,
    span: Span,
    mime: &str,
) -> Value {
    let magics = magic
        .into_iter()
        .map(|b| {
            Value::record(
                record!(
                    "offset" => Value::int(b.offset as _, span),
                    "length" => Value::int(b.length as _, span),
                    "bytes" => Value::binary(b.bytes, span),
                ),
                span,
            )
        })
        .collect();
    Value::record(
        record!(
        "description" => Value::string(format, span),
        "format" => Value::string(data_format, span),
        "mime" => Value::string(mime, span),
        "magics" => Value::list(magics, span)
        ),
        span,
    )
}

fn get_text_format_details(format: &str, text_format: String, span: Span) -> Value {
    let mime = format!(
        "text/{}",
        if text_format == "txt" {
            "plain"
        } else {
            &text_format
        }
    );
    Value::record(
        record!(
        "description" => Value::string(format, span),
        "format" => Value::string(text_format, span),
        "mime" => Value::string(mime, span),
        "magics" => Value::nothing(span),
        ),
        span,
    )
}

fn main() {
    serve_plugin(&FilePlugin, MsgPackSerializer);
}
