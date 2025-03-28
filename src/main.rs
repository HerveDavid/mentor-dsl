#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar); // syntaxe qui importe le module généré

fn main() {
    let expr = grammar::ExprParser::new().parse("22 * (42 + 33)").unwrap();

    println!("Résultat: {}", expr);

    // Vous pouvez aussi tester avec l'entrée utilisateur
    use std::io::{self, BufRead};

    println!("Entrez une expression à calculer (ex: 2 + 3 * 4):");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => match grammar::ExprParser::new().parse(&line) {
                Ok(result) => println!("Résultat: {}", result),
                Err(err) => println!("Erreur de parsing: {:?}", err),
            },
            Err(err) => {
                println!("Erreur de lecture: {:?}", err);
                break;
            }
        }
    }
}
