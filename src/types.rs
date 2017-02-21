#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bool { True, False }

pub type Name = String;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr<'a> {
  Var(Name),
  App(&'a Expr<'a>, &'a Expr<'a>),
  Lam(Name, &'a Expr<'a>),
  Let(Name, &'a Expr<'a>, &'a Expr<'a>),
  Lit(Lit),
  If(&'a Expr<'a>, &'a Expr<'a>, &'a Expr<'a>),
  Fix(&'a Expr<'a>),
  Op(Binop, &'a Expr<'a>, &'a Expr<'a>)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lit {
  LInt(i64),
  LBool(Bool),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Binop { Add, Sub, Mul, Eql }

#[derive(PartialEq, Eq)]
pub enum Program<'a> { Program(&'a [Decl<'a>], &'a Expr<'a>) }

pub type Decl<'a> = (String, Expr<'a>);
