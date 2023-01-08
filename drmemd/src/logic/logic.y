%start Logic
%avoid_insert "INT"
%avoid_insert "FLT"
%avoid_insert "DEVICE"
%avoid_insert "TRUE"
%avoid_insert "FALSE"

%epp EQ "="
%epp GT ">"
%epp GT_EQ ">="
%epp LT "<"
%epp LT_EQ "<="

%%

Logic -> Result<Program, ()>:
    CmpExpr "CONTROL" "DEVICE"
    {
	let v = $3.map_err(|_| ())?;
	let s = $lexer.span_str(v.span());

	Ok(Program::Assign($1?, parse_device(s)))
    }
    ;

CmpExpr -> Result<Expr, ()>:
      Expr "EQ" Expr { Ok(Expr::Eq(Box::new($1?), Box::new($3?))) }
    | Expr "GT" Expr { Ok(Expr::Gt(Box::new($1?), Box::new($3?))) }
    | Expr "GT_EQ" Expr { Ok(Expr::GtEq(Box::new($1?), Box::new($3?))) }
    | Expr "LT" Expr { Ok(Expr::Lt(Box::new($1?), Box::new($3?))) }
    | Expr "LT_EQ" Expr { Ok(Expr::LtEq(Box::new($1?), Box::new($3?))) }
    | Expr { $1 }
    ;

Expr -> Result<Expr, ()>:
    Factor { $1 }
    ;

Factor -> Result<Expr, ()>:
    '(' CmpExpr ')' { $2 }
    | "TRUE" { Ok(Expr::Lit(Value::Bool(true))) }
    | "FALSE" { Ok(Expr::Lit(Value::Bool(false))) }
    | "INT"
      {
          let v = $1.map_err(|_| ())?;

          parse_int($lexer.span_str(v.span()))
      }
    | "FLT"
      {
          let v = $1.map_err(|_| ())?;

	  parse_flt($lexer.span_str(v.span()))
      }
    | Device { $1 }
    ;

Device -> Result<Expr, ()>:
    "DEVICE"
    {
	let v = $1.map_err(|_| ())?;
	let s = $lexer.span_str(v.span());

	Ok(Expr::Var(parse_device(s)))
    }
    ;

Unknown -> ():
    "UNKNOWN" { }
    ;

%%

use drmem_api::types::device::Value;
use super::{Expr, Program};

// Any functions here are in scope for all the grammar actions above.

fn parse_int(s: &str) -> Result<Expr, ()> {
    s.parse::<i32>()
	.map(|v| Expr::Lit(Value::Int(v)))
	.map_err(|_| eprintln!("{} cannot be represented as an i32", s))
}

fn parse_flt(s: &str) -> Result<Expr, ()> {
    s.parse::<f64>()
	.map(|v| Expr::Lit(Value::Flt(v)))
	.map_err(|_| eprintln!("{} cannot be represented as an f64", s))
}

fn parse_device(s: &str) -> String {
    s[1..s.len() - 1].to_string()
}
