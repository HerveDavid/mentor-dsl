#[macro_use]
extern crate lalrpop_util;

mod ast;

lalrpop_mod!(pub grammar); // syntaxe qui importe le module généré

fn main() {
    // Exemple d'expression
    let input = "22 * (42 + 33)";

    // Évaluation traditionnelle avec LALRPOP
    let expr_result = grammar::ExprParser::new().parse(input).unwrap();
    println!("Résultat par évaluation directe: {}", expr_result);

    // Construction de l'AST
    let mut ast_parser = ast::AstParser::new();
    let ast = ast_parser.parse(input);

    println!("\nStructure de l'AST:");
    ast::print_ast(&ast, 0);

    // Évaluation de l'AST
    match ast::eval(&ast) {
        Some(result) => println!("\nRésultat par évaluation de l'AST: {}", result),
        None => println!("\nErreur lors de l'évaluation de l'AST"),
    }

    // Mode interactif
    use std::io::{self, BufRead};
    println!("\nEntrez une expression à calculer (ex: 2 + 3 * 4):");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) if line.trim().is_empty() => break,
            Ok(line) => {
                let input = line.trim();

                // Évaluation directe avec LALRPOP
                match grammar::ExprParser::new().parse(input) {
                    Ok(result) => println!("LALRPOP: {}", result),
                    Err(err) => println!("Erreur LALRPOP: {:?}", err),
                }

                // Construction et évaluation de l'AST
                let mut ast_parser = ast::AstParser::new();
                let ast = ast_parser.parse(input);

                println!("\nStructure de l'AST:");
                ast::print_ast(&ast, 0);

                match ast::eval(&ast) {
                    Some(result) => println!("Évaluation AST: {}", result),
                    None => println!("Erreur lors de l'évaluation de l'AST"),
                }
            }
            Err(err) => {
                println!("Erreur de lecture: {:?}", err);
                break;
            }
        }
        println!("\nEntrez une autre expression (ou ligne vide pour quitter):");
    }
}
