use crate::delta_debug::delta_debug;
use crate::driver;
use crate::driver::{init_for_testing, Setup};
use crate::transformation;
use log::info;
use sqlparser::ast::Statement;
use std::io::Error;
use std::string::ParseError;
use transformation::transformer;

pub fn reduce(ast: Vec<Statement>) -> Result<String, Error> {
    //info!("{:?}", ast[0].to_string());
    let trans = transformer::transform(ast.clone());
    info!("Transformation is : {:?}", trans);
    Ok("Print".to_string())
}

pub fn reduce_statements(current_ast: Vec<Statement>) -> Vec<Statement> {
    let minimal_stmt = delta_debug(current_ast, 2);

    minimal_stmt.unwrap()
}

#[test]
fn test_delta_debugging() -> Result<(), Box<dyn std::error::Error>> {
    init_for_testing();
    let query = "CREATE TABLE F (p BOOLEAN NOT NULL NULL NOT NULL, i BOOLEAN);
INSERT INTO F SELECT * FROM (VALUES ((NOT false), false), (NULL, (NOT (NOT true)))) AS L WHERE (((+(+(-((+110) / (+((-(-150)) * ((247 * (91 * (-47))) + (-86)))))))) = ((((+(+(24 / (+((+89) * (+58)))))) * (-(-((193 + 223) / (-(222 / 219)))))) * (34 * 70)) * (+(+((((+(+(-202))) / (+52)) - (-(228 + (-104)))) * (-24)))))) = (false <> (66 <> 8)));";

    let current_ast = crate::parser::generate_ast(&query)?;
    let ok_stmt = reduce_statements(current_ast.clone());

    if &ok_stmt.len() < &current_ast.len() {
        println!("minimal statement achieved:");
        for i in ok_stmt {
            println!("{:?}", i);
        }
    } else {
        println!("minimal statement not achieved");
    }

    Ok(())
}
