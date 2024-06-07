#![feature(test)]

use std::{error::Error, path::PathBuf, process::exit};

use clap::{Parser, ValueEnum};
use py_ast::semantic::Generator;
use py_codegen::Backend;
use py_ir::Item;
use py_lex::Token;
use terl::{Buffer, ResultMapperExt, Source};

#[cfg(all(test, feature = "backend-llvm"))]
mod tests;

#[derive(ValueEnum, Clone, Copy)]
enum CodeGenBackend {
    #[cfg(feature = "backend-llvm")]
    #[allow(clippy::upper_case_acronyms)]
    LLVM,
    #[cfg(feature = "backend-c")]
    C,
}

#[cfg(feature = "backend-llvm")]
#[derive(ValueEnum, Clone, Copy)]
enum LLVMOutputMode {
    Text,
    Bitcode,
}

// #[cfg(feature = "backend-llvm")]
// #[derive(ValueEnum, Clone, Copy)]
// enum LLVMOptimizeLevel {
//     O0,
//     O1,
//     O2,
//     O3,
// }

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    src: PathBuf,
    #[arg(short, long, help = "path for output file, default to be a.out")]
    output: Option<PathBuf>,
    #[arg(long, help = "path for ast output file")]
    output_ast: Option<PathBuf>,
    #[arg(long, help = "path for py-ir output file")]
    output_ir: Option<PathBuf>,
    #[cfg(feature = "backend-llvm")]
    #[arg(short = 'm', long, value_enum, default_value_t = LLVMOutputMode::Bitcode, help = "llvm ir output mode",)]
    output_mode: LLVMOutputMode,
    // #[cfg(feature = "backend-llvm")]
    // #[arg(short = 'O', long = "opt", value_enum, default_value_t = LLVMOptimizeLevel::O1, help = "llvm ir optimize level",)]
    // optimize_level: LLVMOptimizeLevel,
    #[arg(short = 'b', long, value_enum, default_value_t = CodeGenBackend::C, help = "code generation backend")]
    backend: CodeGenBackend,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // generate ast
    let path = &cli.src;
    let src = std::fs::read_to_string(path)?;
    let path = path.to_string_lossy().to_string();
    let (error_handler, ast) = generate_ast(path.clone(), src);
    let error_handler = (&error_handler.0, &error_handler.1);

    if let Some(ast_path) = cli.output_ast {
        std::fs::write(ast_path, format!("{ast:#?}"))?;
    }

    // generate ir
    let ir = generate_ir(error_handler, &ast);
    if let Some(ast_path) = cli.output_ir {
        let mut file = std::fs::File::create(ast_path)?;
        serde_json::to_writer(&mut file, &ir)?;
    }

    let output = cli.output.unwrap_or_else(|| PathBuf::from("a.out"));

    match cli.backend {
        #[cfg(feature = "backend-llvm")]
        CodeGenBackend::LLVM => {
            use py_codegen_llvm::LLVMBackend;
            let backend = LLVMBackend::init(());
            let module = backend.module(&path, &ir)?;

            match cli.output_mode {
                LLVMOutputMode::Text => {
                    use std::io::Write;
                    let mut file = std::fs::File::create(output)?;
                    write!(&mut file, "{}", module.print_to_string().to_string())?;
                }
                LLVMOutputMode::Bitcode => {
                    module.write_bitcode_to_path(&output);
                }
            }
        }
        #[cfg(feature = "backend-c")]
        CodeGenBackend::C => {
            use py_codegen_c::CBackend;
            let backend = CBackend::init(());
            let module = backend.module(&path, &ir)?;

            std::fs::write(output, module.text())?;
        }
    };

    Ok(())
}

fn generate_ir(error_handler: (&Buffer, &Buffer<Token>), ast: &[py_ast::parse::Item]) -> Vec<Item> {
    let mut scope: py_ast::semantic::Defines = Default::default();

    match scope.generate(ast) {
        Ok(mir) => return mir,
        Err(err) => match err {
            either::Either::Left(errs) => errs
                .into_iter()
                .for_each(|e| eprintln!("{}", py_lex::Token::handle_error(&error_handler, e))),
            either::Either::Right(errss) => errss
                .into_iter()
                .flatten()
                .for_each(|e| eprintln!("{}", py_lex::Token::handle_error(&error_handler, e))),
        },
    }
    exit(-1);
}

type GenAstResult = ((Buffer, Buffer<Token>), Vec<py_ast::parse::Item>);

fn generate_ast(path: String, src: String) -> GenAstResult {
    let source = Buffer::new(path, src.chars().collect());
    let parser = terl::Parser::<char>::new(source);
    let (char_buffer, mut parser) = parser
        .process(|p| {
            let mut tokens = vec![];

            while let Some(token) = p.parse::<Token>().apply(terl::mapper::Try)? {
                tokens.push(token);
            }
            Ok(tokens)
        })
        .unwrap_or_else(|_| unreachable!());
    let parse_result = (|| -> terl::Result<_, terl::ParseError> {
        let mut ast = vec![];
        while parser.peek().is_some() {
            ast.push(
                parser
                    .parse::<py_ast::parse::Item>()
                    .apply(terl::mapper::MustMatch)?,
            )
        }
        Ok(ast)
    })();
    let error_handler = (&char_buffer, parser.buffer());
    let ast = match parse_result {
        Ok(ast) => ast,
        Err(error) => {
            eprintln!("{}", parser.calling_tree());
            eprintln!(
                "{}",
                py_lex::Token::handle_error(&error_handler, error.error())
            );

            exit(-1);
        }
    };
    ((char_buffer, parser.take_buffer()), ast)
}
