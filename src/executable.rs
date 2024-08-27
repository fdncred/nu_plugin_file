use std::path::Path;
use goblin::{
    mach::{Mach, SingleArch}, 
    Object
};
use nu_protocol::{record, Span, Value};

pub struct Binary {
    pub arches: Vec<BinaryArch>,
}
pub struct BinaryArch {
    pub format: &'static str,
    pub arch: String,
    pub dependencies: Vec<String>,
}

impl Binary {
    pub fn into_value(&self, span: Span) -> Value {
        match self.arches.len() {
            0 => Value::nothing(span),
            1 => self.arches[0].into_value(span),
            _ => Value::record(
                record!(
                    "arches" => Value::list(
                        self
                            .arches
                            .iter()
                            .map(|x| x.into_value(span))
                            .collect(),
                        span
                    )
                ), 
                span
            )
        }
    }
}

impl BinaryArch {
    pub fn into_value(&self, span: Span) -> Value {
        Value::record(
            record!(
                "arch" => Value::string(&self.arch, span),
                "format" => Value::string(self.format, span),
                "dependencies" => Value::list(
                    self
                        .dependencies
                        .iter()
                        .map(|x| Value::string(x, span))
                        .collect(),
                    span
                )
            ),
            span
        )
    }
}
impl Binary {
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, String> {
        let buffer = std::fs::read(path).map_err(|e| e.to_string())?;
        let object = Object::parse(&buffer).map_err(|e| e.to_string())?;
        match object {
            Object::Mach(Mach::Binary(prg)) => {
                Ok(Binary{
                    arches: vec![
                        BinaryArch {
                            format: "mach-o",
                            arch: goblin::mach::cputype::get_arch_name_from_types(prg.header.cputype, prg.header.cpusubtype).map_or(String::new(), |x| x.to_lowercase()),
                            dependencies: prg.libs.iter().map(|x| x.to_string()).skip(1).collect(),
                        }
                    ]
                })
            }
            Object::Mach(Mach::Fat(arches)) => {
                Ok(Binary{
                    arches: std::iter::repeat(()).take(arches.narches).enumerate().map(|(index, _)| {
                        match arches.get(index).map_err(|e| e.to_string())? {
                            SingleArch::MachO(prg) => Ok(BinaryArch {
                                format: "mach-o",
                                arch: goblin::mach::cputype::get_arch_name_from_types(prg.header.cputype, prg.header.cpusubtype).map_or(String::new(), |x| x.to_lowercase()),
                                dependencies: prg.libs.iter().map(|x| x.to_string()).skip(1).collect(),
                            }),
                            SingleArch::Archive(_) => todo!(),
                        }
                    }).collect::<Result<Vec<_>, String>>()?
                })
            }
            Object::PE(prg) => {
                Ok(Binary{
                    arches: vec![
                        BinaryArch {
                            format:  if prg.is_64 {"pe32+"} else {"pe32"},
                            arch: goblin::pe::header::machine_to_str(prg.header.coff_header.machine).to_lowercase(),
                            dependencies: prg.libraries.iter().map(|x| x.to_string()).collect(),
                        }
                    ]
                })
            }
            Object::Elf(prg) => {
                Ok(Binary{
                    arches: vec![
                        BinaryArch {
                            format:  if prg.is_64 {"elf64"} else {"elf32"},
                            arch: goblin::elf::header::machine_to_str(prg.header.e_machine).to_lowercase(),
                            dependencies: prg.libraries.iter().map(|x| x.to_string()).collect(),
                        }
                    ]
                })
            },
            _ => todo!(),
        }
    }
}