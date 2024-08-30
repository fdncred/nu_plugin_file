use std::{fs::File, io::{Read, Seek, SeekFrom}, path::Path};
use goblin::{
    mach::{Mach, SingleArch}, 
    Object
};
use nu_protocol::{record, Span, Value};
use crate::magic::MagicBytesMeta;

pub struct Binary {
    pub arches: Vec<BinaryArch>,
    pub magic_bytes: Option<MagicBytesMeta>,
}
pub struct BinaryArch {
    pub magic_bytes: MagicBytesMeta,
    pub format: &'static str,
    pub arch: String,
    pub dependencies: Vec<String>,
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
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, String> {
        let buffer = std::fs::read(path).map_err(|e| e.to_string())?;
        let object = Object::parse(&buffer).map_err(|e| e.to_string())?;
        match object {
            Object::Mach(Mach::Binary(prg)) => {
                let magic_bytes = prg.header.magic.to_le_bytes().to_vec();
                Ok(Binary{
                    arches: vec![
                        BinaryArch {
                            magic_bytes: MagicBytesMeta{
                                offset: 0,
                                length: magic_bytes.len(),
                                bytes: magic_bytes
                            },
                            format: "mach-o",
                            arch: goblin::mach::cputype::get_arch_name_from_types(prg.header.cputype, prg.header.cpusubtype).map_or(String::new(), |x| x.to_lowercase()),
                            dependencies: prg.libs.iter().map(|x| x.to_string()).skip(1).collect(),
                        }
                    ],
                    magic_bytes: None
                })
            }
            Object::Mach(Mach::Fat(arches)) => {
                let magic_bytes = buffer[0..4].to_vec();
                Ok(Binary{
                    arches: arches.iter_arches().enumerate().map(|(index, arch)| {
                        let arch = arch.map_err(|e| e.to_string())?;
                        let prg = arches.get(index).map_err(|e| e.to_string())?;
                        match prg {
                            SingleArch::MachO(prg) => {
                                let magic_bytes = prg.header.magic.to_le_bytes().to_vec();
                                Ok(BinaryArch {
                                    magic_bytes: MagicBytesMeta{
                                        offset: arch.offset as _,
                                        length: magic_bytes.len(),
                                        bytes: magic_bytes
                                    },
                                    format: "mach-o",
                                    arch: goblin::mach::cputype::get_arch_name_from_types(prg.header.cputype, prg.header.cpusubtype).map_or(String::new(), |x| x.to_lowercase()),
                                    dependencies: prg.libs.iter().map(|x| x.to_string()).skip(1).collect(),
                                })
                            },
                            SingleArch::Archive(_) => todo!(),
                        }
                    }).collect::<Result<Vec<_>, String>>()?,
                    magic_bytes: Some(MagicBytesMeta{
                        offset: 0,
                        length: magic_bytes.len(),
                        bytes: magic_bytes
                    })
                })
            }
            Object::PE(prg) => {
                let dos_magic_bytes = prg.header.dos_header.signature.to_le_bytes().to_vec();
                let pe_magic_bytes = prg.header.signature.to_le_bytes().to_vec();
                Ok(Binary{
                    arches: vec![
                        BinaryArch {
                            magic_bytes: MagicBytesMeta{
                                offset: prg.header.dos_header.pe_pointer as _,
                                length: pe_magic_bytes.len(),
                                bytes: pe_magic_bytes
                            },
                            format: if prg.is_64 {"pe32+"} else {"pe32"},
                            arch: goblin::pe::header::machine_to_str(prg.header.coff_header.machine).to_lowercase(),
                            dependencies: prg.libraries.iter().map(|x| x.to_string()).collect(),
                        }
                    ],
                    magic_bytes: Some(MagicBytesMeta{
                        offset: 0,
                        length: dos_magic_bytes.len(),
                        bytes: dos_magic_bytes
                    }),
                })
            }
            Object::Elf(prg) => {
                let magic_bytes = prg.header.e_ident[0..4].to_vec();
                Ok(Binary{
                    arches: vec![
                        BinaryArch {
                            magic_bytes: MagicBytesMeta{
                                offset: 0,
                                length: magic_bytes.len(),
                                bytes: magic_bytes
                            },
                            format:  if prg.is_64 {"elf64"} else {"elf32"},
                            arch: goblin::elf::header::machine_to_str(prg.header.e_machine).to_lowercase(),
                            dependencies: prg.libraries.iter().map(|x| x.to_string()).collect(),
                        }
                    ],
                    magic_bytes: None
                })

            },
            _ => Err("Unsupported file format".to_string()),
        }
    }
    pub fn has_magic_bytes(path: impl AsRef<Path>) -> bool {
        let Ok(ref mut file) = File::open(&path) else {
            return false;
        };
        let mut buf: [u8; 4] = [0_u8; 4];

        if file.seek(SeekFrom::Start(0)).is_err() || file.read_exact(&mut buf).is_err() {
            return false
        }
        if buf.eq(goblin::elf::header::ELFMAG) {
            return true
        }
        let magic32 = u32::from_le_bytes(buf);
        if magic32 == goblin::mach::header::MH_MAGIC
            || magic32 == goblin::mach::header::MH_CIGAM
            || magic32 == goblin::mach::header::MH_MAGIC_64
            || magic32 == goblin::mach::header::MH_CIGAM_64 
            || magic32 == goblin::mach::fat::FAT_MAGIC
            || magic32 == goblin::mach::fat::FAT_CIGAM {
            return true
        }
        let magic16 = u16::from_le_bytes([buf[0], buf[1]]);
        if magic16 == goblin::pe::optional_header::MAGIC_32 
            || magic16 == goblin::pe::optional_header::MAGIC_64 {
            return true;
        }
        false
    }
    pub fn description(&self) -> String {
        let desc_arch = |arch: &BinaryArch| format!("{} binary, {}", arch.format, arch.arch);
        match self.arches.len() {
            0 => String::new(),
            1 => desc_arch(&self.arches[0]),
            len => {
                format!("fat binary, with {} arches: {}", len, self.arches.iter().map(|a| format!("[{}]", desc_arch(a))).collect::<Vec<_>>().join(", "))
            }
        }
    }
}