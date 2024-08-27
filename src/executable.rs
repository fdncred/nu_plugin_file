use std::path::Path;
use goblin::{
    mach::{cputype, Mach, SingleArch}, 
    Object
};
use nu_protocol::{record, Span, Value};

pub struct Binary {
    pub arches: Vec<BinaryArch>,
}
pub struct BinaryArch {
    pub format: &'static str,
    pub arch: &'static str,
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
                "arch" => Value::string(self.arch, span),
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
                            arch: Self::mach_get_arch(prg.header.cputype),
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
                                arch: Self::mach_get_arch(prg.header.cputype),
                                dependencies: prg.libs.iter().map(|x| x.to_string()).skip(1).collect(),
                            }),
                            SingleArch::Archive(_) => todo!(),
                        }
                    }).collect::<Result<Vec<_>, String>>()?
                })
            }
            _ => todo!(),
        }
    }
    fn mach_get_arch(cpu_type: u32) -> &'static str {
        match cpu_type {
            cputype::CPU_TYPE_ANY => "any",
            cputype::CPU_TYPE_VAX => "vax",
            cputype::CPU_TYPE_MC680X0 => "mc680x0",
            cputype::CPU_TYPE_X86 => "x86",
            // cputype::CPU_TYPE_I386 => "i386",
            cputype::CPU_TYPE_X86_64 => "x86_64",
            cputype::CPU_TYPE_MIPS => "mips",
            cputype::CPU_TYPE_MC98000 => "mc98000",
            cputype::CPU_TYPE_HPPA => "hppa",
            cputype::CPU_TYPE_ARM => "arm",
            cputype::CPU_TYPE_ARM64 => "arm64",
            cputype::CPU_TYPE_ARM64_32 => "arm64_32",
            cputype::CPU_TYPE_MC88000 => "mc88000",
            cputype::CPU_TYPE_SPARC => "sparc",
            cputype::CPU_TYPE_I860 => "i860",
            cputype::CPU_TYPE_ALPHA => "alpha",
            cputype::CPU_TYPE_POWERPC => "powerpc",
            cputype::CPU_TYPE_POWERPC64 => "powerpc64",
            _ => "unknown",
        }
    }
}