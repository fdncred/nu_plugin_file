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
        eprintln!("starting run");

        // fn test_path(subpath: &str) -> Option<extensions::Extension> {
        //     eprintln!("testing {}...", subpath);
        //     let canon_path = Path::new(subpath).canonicalize().ok()?;
        //     // .extension()
        //     // .and_then(|ext| ext.to_str())
        //     // .and_then(|ext| {
        //     //     eprintln!("ext: {:?}", ext);
        //     //     let file_format = extensions::Extension::resolve_conflicting(ext, true);
        //     //     eprintln!("file_format: {:?}", file_format);
        //     //     file_format
        //     // })
        //     // .and_then(|_| {
        //     eprintln!("canon_path: {:?}", &canon_path);
        //     eprintln!("no ext1");
        //     let file_format = extensions::Extension::resolve_conflicting(canon_path, true);
        //     eprintln!("file_format: {:?}", file_format);
        //     file_format
        //     // })
        //     // .or_else(|| {
        //     //     eprintln!("no ext2");
        //     //     extensions::Extension::resolve_conflicting(
        //     //         subpath.split('.').last().unwrap(),
        //     //         true,
        //     //     )
        //     // })
        //     //
        //     // extensions::Extension::resolve_conflicting(subpath.split('.').last().unwrap(), true)
        // }

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
            let filename = if filename.item.starts_with("~") {
                filename.item.replace("~", home_dir.to_str().unwrap())
            } else {
                filename.item
            };
            eprintln!("filename: {:?}", &filename);
            let canon_path = Path::new(&filename).canonicalize().unwrap();
            eprintln!("canon_path: {:?}", &canon_path);
            let file_format = extensions::Extension::resolve_conflicting(canon_path, true);
            eprintln!("file_format: {:?}", file_format);

            // let ext = test_path(&filename.item);
            // eprintln!("ext: {:?}", ext);
        }

        // Handle::current().block_on(async {
        // let ext = test_path("test.txt");
        // println!("ext: {:?}", ext);
        // });

        // let ret_val = match input {
        //     Value::String { val, span } => crate::file::file_do_something(param, val, *span)?,
        //     v => {
        //         return Err(LabeledError {
        //             label: "Expected something from pipeline".into(),
        //             msg: format!("requires some input, got {}", v.get_type()),
        //             span: Some(call.head),
        //         });
        //     }
        // };

        // Ok(ret_val)
        Ok(Value::Nothing { span: call.head })
    }
}

fn main() {
    serve_plugin(&mut Implementation::new(), MsgPackSerializer);
}
