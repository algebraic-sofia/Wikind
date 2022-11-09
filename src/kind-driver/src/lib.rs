use kind_pass::{expand, desugar, erasure};
use kind_report::report::FileCache;
use kind_span::SyntaxCtxIndex;

use kind_tree::{desugared, backend, concrete};
use session::Session;
use std::{path::PathBuf, collections::HashSet};

use kind_checker as checker;

pub mod errors;
pub mod resolution;
pub mod session;

impl FileCache for Session {
    fn fetch(&self, ctx: SyntaxCtxIndex) -> Option<(PathBuf, &String)> {
        let path = self.loaded_paths[ctx.0].as_ref().to_owned();
        Some((path, &self.loaded_sources[ctx.0]))
    }
}

pub fn type_check_book(session: &mut Session, path: &PathBuf) -> Option<desugared::Book> {
    let concrete_book = to_book(session, path)?;
    let desugared_book = desugar::desugar_book(session.diagnostic_sender.clone(), &concrete_book)?;

    let succeeded = checker::type_check(&desugared_book, session.diagnostic_sender.clone());

    if !succeeded {
        return None;
    }

    let erased = erasure::erase_book(
        &desugared_book,
        session.diagnostic_sender.clone(),
        HashSet::from_iter(vec!["Main".to_string()]),
    )?;

    Some(erased)
}

pub fn to_book(session: &mut Session, path: &PathBuf) -> Option<concrete::Book> { 
    let mut concrete_book = resolution::parse_and_store_book(session, path)?;

    expand::expand_book(&mut concrete_book);

    Some(concrete_book)
}

pub fn erase_book(session: &mut Session, path: &PathBuf) -> Option<desugared::Book> {
    let concrete_book = to_book(session, path)?;
    let desugared_book = desugar::desugar_book(session.diagnostic_sender.clone(), &concrete_book)?;
    erasure::erase_book(
        &desugared_book,
        session.diagnostic_sender.clone(),
        HashSet::from_iter(vec!["Main".to_string()]),
    )
}


pub fn check_erasure_book(session: &mut Session, path: &PathBuf) -> Option<desugared::Book> {
    let concrete_book = to_book(session, path)?;
    let desugared_book = desugar::desugar_book(session.diagnostic_sender.clone(), &concrete_book)?;
    erasure::erase_book(
        &desugared_book,
        session.diagnostic_sender.clone(),
        HashSet::from_iter(vec!["Main".to_string()]),
    )?;
    Some(desugared_book)
}

pub fn compile_book_to_hvm(session: &mut Session, path: &PathBuf) -> Option<backend::File> {
    erase_book(session, path).map(kind_target_hvm::compile_book)
}

pub fn execute_file(file: &backend::File) -> Box<backend::Term> {
    // TODO: Change to from_file when hvm support it
    let mut runtime = hvm::Runtime::from_code(&file.to_string()).unwrap();
    let main = runtime.alloc_code("Main").unwrap();
    runtime.run_io(main);
    runtime.normalize(main);
    let term = runtime.readback(main);
    term
}

pub fn eval_in_checker(book: &desugared::Book) -> Box<backend::Term> {
    // TODO: Change to from_file when hvm support it
    checker::eval_api(book)
}

pub fn generate_checker(book: &desugared::Book) -> String {
    // TODO: Change to from_file when hvm support it
    checker::gen_checker(book)
}