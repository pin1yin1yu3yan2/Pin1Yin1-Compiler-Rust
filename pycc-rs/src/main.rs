#![feature(test)]

use std::{error::Error, path::PathBuf, process::exit};

use clap::{Parser, ValueEnum};
use py_ast::semantic::Generator;
use py_ir::Item;
use py_lex::Token;
use pyc::Backend;
use terl::{Buffer, ResultMapperExt, Source};

pub mod compile;

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

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    src: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long)]
    output_ast: bool,
    #[arg(long)]
    output_ir: bool,
    #[arg(long, value_enum, default_value_t = CodeGenBackend::LLVM)]
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

    if cli.output_ast {
        println!("{ast:#?}")
    }

    // generate ir
    let ir = generate_ir(error_handler, &ast);
    if cli.output_ir {
        println!("{}", serde_json::to_string(&ir).unwrap());
    }

    let config = ();
    let output = cli.output.unwrap_or_else(|| PathBuf::from("a.out"));

    match cli.backend {
        #[cfg(feature = "backend-llvm")]
        CodeGenBackend::LLVM => {
            use py_codegen_llvm::LLVMBackend;
            let backend = LLVMBackend::init(config);
            let module = backend.module(&path, &ir)?;
            let code = backend.code(&module);
            std::fs::write(output, &*code)?;
        }
        #[cfg(feature = "backend-c")]
        CodeGenBackend::C => {
            use py_codegen_c::CBackend;
            let backend = CBackend::init(config);
            let module = backend.module(&path, &ir)?;
            let code = backend.code(&module);
            std::fs::write(output, &*code)?;
        }
    };

    Ok(())
}

fn generate_ir(error_handler: (&Buffer, &Buffer<Token>), ast: &[py_ast::parse::Item]) -> Vec<Item> {
    let mut scope: py_ast::semantic::Defines = Default::default();
    let mut mir = vec![];
    for item in ast {
        match scope.generate(item) {
            Ok(Some(item)) => mir.push(item),
            Err(e) => {
                match e {
                    either::Either::Left(e) => {
                        eprintln!("{}", py_lex::Token::handle_error(&error_handler, e))
                    }
                    either::Either::Right(es) => es.into_iter().for_each(|e| {
                        eprintln!("{}", py_lex::Token::handle_error(&error_handler, e))
                    }),
                }
                exit(-1);
            }
            _ => {}
        };
    }
    mir
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
