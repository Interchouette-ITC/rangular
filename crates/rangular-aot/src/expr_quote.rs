use proc_macro2::TokenStream;
use quote::quote;
use rangular_expr::{BinOp, Expr, Literal, UnOp};

pub fn expr_tokens(expr: &Expr) -> TokenStream {
    match expr {
        Expr::Lit(lit) => lit_tokens(lit),
        Expr::Ident(name) => quote! { rangular_expr::Expr::Ident(#name.to_string()) },
        Expr::Unary { op, expr } => {
            let inner = expr_tokens(expr);
            let op = unop_tokens(*op);
            quote! { rangular_expr::Expr::Unary { op: #op, expr: Box::new(#inner) } }
        }
        Expr::Binary { op, left, right } => {
            let l = expr_tokens(left);
            let r = expr_tokens(right);
            let op = binop_tokens(*op);
            quote! {
                rangular_expr::Expr::Binary {
                    op: #op,
                    left: Box::new(#l),
                    right: Box::new(#r),
                }
            }
        }
        Expr::Call { callee, args } => {
            let c = expr_tokens(callee);
            let a = args.iter().map(expr_tokens);
            quote! {
                rangular_expr::Expr::Call {
                    callee: Box::new(#c),
                    args: vec![#(#a),*],
                }
            }
        }
        Expr::Pipe { expr, name, args } => {
            let inner = expr_tokens(expr);
            let a = args.iter().map(expr_tokens);
            quote! {
                rangular_expr::Expr::Pipe {
                    expr: Box::new(#inner),
                    name: #name.to_string(),
                    args: vec![#(#a),*],
                }
            }
        }
        Expr::Ternary {
            cond,
            then_branch,
            else_branch,
        } => {
            let c = expr_tokens(cond);
            let t = expr_tokens(then_branch);
            let e = expr_tokens(else_branch);
            quote! {
                rangular_expr::Expr::Ternary {
                    cond: Box::new(#c),
                    then_branch: Box::new(#t),
                    else_branch: Box::new(#e),
                }
            }
        }
    }
}

fn lit_tokens(lit: &Literal) -> TokenStream {
    match lit {
        Literal::Str(s) => {
            quote! { rangular_expr::Expr::Lit(rangular_expr::Literal::Str(#s.to_string())) }
        }
        Literal::Num(n) => quote! { rangular_expr::Expr::Lit(rangular_expr::Literal::Num(#n)) },
        Literal::Bool(b) => quote! { rangular_expr::Expr::Lit(rangular_expr::Literal::Bool(#b)) },
    }
}

fn unop_tokens(op: UnOp) -> TokenStream {
    match op {
        UnOp::Not => quote! { rangular_expr::UnOp::Not },
    }
}

fn binop_tokens(op: BinOp) -> TokenStream {
    match op {
        BinOp::Or => quote! { rangular_expr::BinOp::Or },
        BinOp::And => quote! { rangular_expr::BinOp::And },
        BinOp::Eq => quote! { rangular_expr::BinOp::Eq },
        BinOp::Ne => quote! { rangular_expr::BinOp::Ne },
        BinOp::Add => quote! { rangular_expr::BinOp::Add },
    }
}
