extern crate proc_macro;

use proc_macro::TokenStream as TS;
use proc_macro2::*;
use quote::quote;

type TT = TokenTree;
type TS2 = TokenStream;

fn binop_general(op: Literal, in_ty: Ident, out_ty: Ident) -> TS {
    let lit_str = op.to_string();
    let lit_str = lit_str[1..lit_str.len() - 1].to_string();
    let spacing: Spacing = if lit_str.len() > 1 {
        Spacing::Joint
    } else {
        Spacing::Alone
    };
    let p: Vec<Punct> = lit_str.chars().map(|c| Punct::new(c, spacing)).collect();
    let tokens = quote! {
        #op ((a: #in_ty, b: #in_ty) -> #out_ty) { a #(#p)* b }
    };
    atomify(TS::from(tokens))
}

#[proc_macro]
pub fn binops(input: TS) -> TS {
    let iter = TS2::from(input).into_iter();

    enum Type {
        Arithmetic,
        Comparison,
    }

    let mut arms: Vec<TS2> = Vec::new();
    let mut second = false;
    let mut ty: Option<Type> = None;
    for i in iter {
        if second {
            if let TT::Literal(lit) = i {
                let f = if let Some(Type::Arithmetic) = ty {
                    arith_op
                } else if let Some(Type::Comparison) = ty {
                    cmp_op
                } else {
                    unreachable!()
                };
                let tt: TT = lit.clone().into();
                let ts: TS2 = tt.into();
                let ts: TS = ts.into();
                let closure: TS2 = f(ts).into();
                let arm = quote! {
                    #lit => #closure
                };
                arms.push(arm);
            } else {
                panic!("Expected a literal following type.")
            }
        } else if let TT::Ident(ident) = i {
            let ident = ident.to_string();
            ty = Some(match ident.as_str() {
                "a" => Type::Arithmetic,
                "c" => Type::Comparison,
                _ => {
                    panic!("Unrecognized type.");
                }
            });
        } else {
            panic!("Expected an identifier specifying type.")
        }
        second = !second;
    }

    let tokens = quote! {
        Some(match op {
            #(#arms,)*
            _ => {
                return None;
            }
        })
    };

    tokens.into()
}

/// arith_op!("+" Num) == atomify!(("+" ((a:Num, b:Num) -> Num) { a + b } ))
#[proc_macro]
pub fn arith_op(input: TS) -> TS {
    let input = TS2::from(input);
    let mut iter = input.into_iter();
    let lit = iter.next().unwrap();

    if let TT::Literal(lit) = lit {
        let ty = Ident::new("Num", Span::call_site());
        let ty2 = ty.clone();
        binop_general(lit, ty, ty2)
    } else {
        panic!("Expected literal.");
    }
}

#[proc_macro]
pub fn cmp_op(input: TS) -> TS {
    let input = TS2::from(input);
    let mut iter = input.into_iter();
    let lit = iter.next().unwrap();

    if let TT::Literal(lit) = lit {
        let ty1 = Ident::new("Num", Span::call_site());
        let ty2 = Ident::new("Bool", Span::call_site());
        binop_general(lit, ty1, ty2)
    } else {
        panic!("Expected literal.");
    }
}

#[proc_macro]
pub fn shuffle(input: TS) -> TS {
    let input: TS2 = input.into();
    let mut args: Vec<Ident> = Vec::new();
    let mut out: Vec<Ident> = Vec::new();
    let mut iter = input.into_iter();
    loop {
        let tt: TT = iter.next().unwrap();
        if let TT::Ident(i) = tt {
            args.push(i);
        } else {
            break;
        }
    }

    iter.next().unwrap();

    loop {
        let tt: Option<TT> = iter.next();
        if tt.is_none() {
            break;
        }
        if let TT::Ident(i) = tt.unwrap() {
            out.push(i);
        } else {
            panic!("Unexpected non-ident");
        }
    }

    let args = args.iter().rev();

    let num_in = args.len();
    let num_out = out.len();

    let tokens = quote! {
        O::new(|env: &mut Env| {
            #(let #args = env.pop_atom();)*
            #(env.push_atom(#out);)*
        }, Some((#num_in as u8, #num_out as u8)))
    };

    tokens.into()
}

/// Takes something similar to the form `"+" ((a: Num, b: Num) -> Num) { a + b }`
/// and constructs a closure representing this for Pancake internally. This is
/// the core macro for constructing closure-based Atoms.
#[proc_macro]
pub fn atomify(input: TS) -> TS {
    let input = TS2::from(input);
    impl_atomify(input.into_iter())
}

// TODO: Unhandled cases:
// - Precondition, postcondition (check if necessary?)
// - Ignoring type (polymorphism)
// - Returning more than one element
fn impl_atomify(iter: impl Iterator<Item = TT>) -> TS {
    let (name, arg_name, arg_type, return_type, expr) = extract(iter);

    let name: TT = name.into();
    let arg_name: Vec<TT> = arg_name.into_iter().map(|x| x.into()).collect();
    let arg_name_rev = arg_name.clone().into_iter().rev();
    let arg_type: Vec<TT> = arg_type.into_iter().map(|x| x.into()).collect();

    let num_in = arg_name.len();
    let arity = if return_type.is_some() {
        quote! {
            Some((#num_in as u8, 1 as u8))
        }
    } else {
        quote! {
            None
        }
    };

    let expr: TS2 = if let Some(return_type) = return_type {
        if return_type.to_string() == "Any" {
            quote! {
                env.push_atom(#expr);
            }
        } else {
            quote! {
                let output = #return_type(#expr);
                env.push_atom(output);
            }
        }
    } else {
        quote! {#expr}
    };

    let tokens = quote! {
        O::new(|env: &mut Env| {
            #(let #arg_name_rev = env.pop_atom();)*
            if let (#(#arg_type(mut #arg_name)),*) = (#(#arg_name),*) {
                #expr
            } else {
                panic!("Invalid arguments for {}", #name);
            }
        }, #arity)
    };

    TS::from(tokens)
}

fn extract(
    mut iter: impl Iterator<Item = TT>,
) -> (Literal, Vec<Ident>, Vec<Ident>, Option<Ident>, TS2) {
    let lit = iter.next().unwrap();
    let type_sig = iter.next().unwrap();
    let expr = iter.next().unwrap();

    if let (TT::Literal(lit), TT::Group(type_sig), TT::Group(expr)) = (lit, type_sig, expr) {
        let name = lit;
        let mut iter = type_sig.stream().into_iter();
        let args = iter.next().unwrap();

        let mut return_type: Option<Ident> = None;
        if iter.next().is_some() {
            // ->
            let _ = iter.next().unwrap(); // ->
            if let TT::Ident(r) = iter.next().unwrap() {
                return_type = Some(r);
            } else {
                panic!("Return type not Ident");
            }
        }

        if let TT::Group(args) = args {
            // These should be of the form Ident(name), Punct(:),
            // Ident(type), Punct(,), ...
            let args_vec: Vec<TT> = args.stream().into_iter().map(|x| x).collect();

            let args_vec = args_vec
                .split(|tt| {
                    if let TT::Punct(p) = tt {
                        p.as_char() == ','
                    } else {
                        false
                    }
                })
                .map(|v_tt| {
                    let mut iter = v_tt.iter();
                    let arg_name = iter.next().unwrap();
                    let _ = iter.next().unwrap();
                    let arg_type = iter.next().unwrap();
                    if let (TT::Ident(arg_name), TT::Ident(arg_type)) = (arg_name, arg_type) {
                        (arg_name.clone(), arg_type.clone())
                    } else {
                        panic!();
                    }
                })
                .unzip();

            (name, args_vec.0, args_vec.1, return_type, expr.stream())
        } else {
            panic!("args not Group")
        }
    } else {
        panic!("lit, type_sig, expr mismatch")
    }
}
