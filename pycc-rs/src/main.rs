#![feature(test)]

use std::path::PathBuf;

use clap::Parser;
use compile::CodeGen;
use inkwell::context::Context;
use py_ast::semantic::Generator;
use py_lex::Token;
use terl::{Buffer, ResultMapperExt, Source};

pub mod compile;
pub mod primitive;
pub mod scope;
#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    src: PathBuf,
    #[arg(long, default_value_t = true)]
    output_llvm: bool,
    #[arg(long)]
    output_ast: bool,
    #[arg(long)]
    output_ir: bool,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    // generate ast
    let path = &cli.src;
    let (error_handler, ast) = generate_ast_from_fs(path)?;
    let error_handler = (&error_handler.0, &error_handler.1);

    if cli.output_ast {
        println!("{ast:#?}")
    }

    // generate mir
    let mir = generate_ir(error_handler, &ast);
    if cli.output_ir {
        println!("{mir:#?}")
    }

    let context = Context::create();
    let compiler = CodeGen::new(&context, "test", &mir).unwrap();
    if cli.output_llvm {
        println!("{}", compiler.llvm_ir())
    }
    Ok(())
}

fn generate_ir(
    error_handler: (&Buffer, &Buffer<Token>),
    ast: &[py_ast::parse::Item],
) -> Vec<py_ir::Item<py_ir::Variable>> {
    let mut scope: py_ast::semantic::Defines = Default::default();
    let mut mir = vec![];
    for item in ast {
        let item = scope
            .generate(item)
            .map_err(|e| match e {
                either::Either::Left(e) => {
                    eprintln!("{}", py_lex::Token::handle_error(&error_handler, e))
                }
                either::Either::Right(es) => es
                    .into_iter()
                    .for_each(|e| eprintln!("{}", py_lex::Token::handle_error(&error_handler, e))),
            })
            .unwrap();
        if let Some(item) = item {
            mir.push(item)
        }
    }
    mir
}

type GenAstResult = ((Buffer, Buffer<Token>), Vec<py_ast::parse::Item>);

fn generate_ast_from_fs(path: &PathBuf) -> std::io::Result<GenAstResult> {
    let src = std::fs::read_to_string(path)?;
    Ok(generate_ast(path.to_string_lossy().to_string(), src))
}

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
    let parse_result = (|| -> Result<_, terl::ParseError> {
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
            panic!()
        }
    };
    ((char_buffer, parser.take_buffer()), ast)
}
