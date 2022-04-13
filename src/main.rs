use anyhow::Result;
use wast::parser::{self, ParseBuffer};
use wast::ModuleField;
use wast::ModuleKind;
use wast::ValType;

use std::fs;
use std::env;
use std::io::Read;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} in.wasm", args[0]);
        return Ok(());
    }

    let mut f = fs::File::open(&args[1])?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let buf = ParseBuffer::new(&contents)?;
    let ast = parser::parse::<wast::Wast>(&buf)?;

    for directive in ast.directives {
        match directive {
            wast::WastDirective::Module(mut _mod) => {
                if let ModuleKind::Text(txt) = &_mod.kind {
                    for field in txt {
                        if let ModuleField::Func(_func) = field {
                            let return_type = _func.ty.inline.clone().unwrap().results[0];
                            let c_return_type = match return_type {
                                ValType::I32 => "int",
                                ValType::I64 => "long int",
                                ValType::F32 => "float",
                                ValType::F64 => "double",
                                _ => panic!("AHHHH!"),
                            };
                            let mut c_param: Vec<String> = Vec::new();
                            for (_, _, p) in _func.ty.inline.clone().unwrap().params.iter() {
                                match p {
                                    ValType::I32 => c_param.push("int".to_string()),
                                    ValType::I64 => c_param.push("long int".to_string()),
                                    ValType::F32 => c_param.push("float".to_string()),
                                    ValType::F64 => c_param.push("double".to_string()),
                                    _ => panic!("AHHHH!"),
                                };
                            }
    
                            print!(
                                "IMPORT {} wasmf_{}({}",
                                c_return_type, _func.exports.names[0], c_param[0]
                            );
                            if c_param.len() > 1 {
                                for n in 1..c_param.len() {
                                    print!(
                                        ", {}",
                                        c_param[n]
                                    );
                                }
                            }
                            println!(");");
                        }
                    }
                    // And generate wasm module
                    let path: &Path = Path::new("test.wasm");
                    fs::write(path, _mod.encode().unwrap()).unwrap();
                }
            },

            wast::WastDirective::AssertReturn{span: _, exec, results} => {
                println!("\nint main(int argc, char* argv[]) {{");
                match exec {
                    wast::WastExecute::Invoke(invoke) =>{
                        print!("\tawsm_assert(wasmf_{}(", invoke.name);
                        let mut ct = 0;
                        // obtain arguments
                        for p in invoke.args.iter() {
                            match p.instrs[0] {
                                wast::Instruction::I32Const(val) => {
                                    if ct > 0 {print!(", ");}
                                    print!("{:?}", val);
                                    ct = ct+1;
                                },
                                _ => {print!("OTHER");}
                            }
                        }
                        // obtain results
                        print!(") == ");
                        match results[0] {
                            wast::AssertExpression::I32(val) => {
                                println!("{:?});", val);
                            }
                            _ => {print!("OTHER");}
                        }
                    },
                    _ => {}
                }

                println!("}}");
            },

            _ => {}

        }
    }

    Ok(())
}