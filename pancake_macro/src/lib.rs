extern crate proc_macro;

use proc_macro2::{Ident, Literal, Punct, Spacing, TokenStream, TokenTree};
use proc_macro::TokenStream as TS;
use quote::quote;

type TT = TokenTree;
type TS2 = TokenStream;

/// binop!("+" Num) == atomify!(("+" ((a:Num, b:Num) -> Num) { a + b } ))
#[proc_macro]
pub fn binop(input: TS) -> TS {
    let input = TS2::from(input);
    let mut iter = input.into_iter();
    let lit = iter.next().unwrap();
    let ty = iter.next().unwrap();

    if let (TT::Literal(lit), TT::Ident(ty)) = (lit, ty) {
        let lit_str = lit.to_string();
        let lit_str = lit_str[1..lit_str.len()-1].to_string();
        let spacing: Spacing = if lit_str.len() > 1 { Spacing::Joint } else { Spacing::Alone };
        let p: Vec<Punct> = lit_str.chars().map(|c| Punct::new(c, spacing)).collect();
        let tokens = quote! {
            (#lit ((a: #ty, b: #ty) -> #ty) { a #(#p)* b })
        };
        atomify(TS::from(tokens))
    } else {
        panic!("Expected literal & type ident");
    }
}

/// Takes something similar to the form `"+" ((a: Num, b: Num) -> Num) { a + b }`
/// and constructs a closure representing this for Pancake internally. This is
/// the core macro for constructing closure-based Atoms.
#[proc_macro]
pub fn atomify(input: TS) -> TS {
    let input = TS2::from(input);

    let (name, arg_name, arg_type, return_type, expr) = extract(input);

    let name: TT = name.into();
    let arg_name: Vec<TT> = arg_name.into_iter().map(|x| x.into()).collect();
    let arg_name_rev = arg_name.clone().into_iter().rev();
    let arg_type: Vec<TT> = arg_type.into_iter().map(|x| x.into()).collect();
    let return_type: TT = return_type.into();

    let tokens = quote! {
        |env: &mut Env| {
            #(let #arg_name_rev = env.pop_atom();)*
            if let (#(#arg_type(#arg_name)),*) = (#(#arg_name),*) {
                env.push_atom(#return_type(#expr));
            } else {
                panic!("Invalid arguments for {}", #name);
            }
        }
    };

    TS::from(tokens)
}

fn extract(input: TS2) -> (Literal, Vec<Ident>, Vec<Ident>, Ident, TS2) {
    let mut iter = input.into_iter();
    let tt = iter.next().unwrap();

    if let TT::Group(all) = tt {
        let ts = all.stream();
        let mut iter = ts.into_iter();
        let lit = iter.next().unwrap();
        let type_sig = iter.next().unwrap();
        let expr = iter.next().unwrap();

        if let (TT::Literal(lit), TT::Group(type_sig), TT::Group(expr)) = (lit, type_sig, expr) {
            let name = lit;
            let mut iter = type_sig.stream().into_iter();
            let args = iter.next().unwrap();
            let _ = iter.next().unwrap(); // ->
            let _ = iter.next().unwrap(); // ->
            let return_type = iter.next().unwrap();

            if let TT::Group(args) = args {
                // These should be of the form Ident(name), Punct(:),
                // Ident(type), Punct(,), ...
                let args_vec: Vec<TT> = args.stream().into_iter().map(|x| x).collect();

                let args_vec = args_vec.split(
                    |tt| {
                        if let TT::Punct(p) = tt {
                            p.as_char() == ','
                        } else {
                            false
                        }
                    }
                ).map(
                    |v_tt| {
                        let mut iter = v_tt.into_iter();
                        let arg_name = iter.next().unwrap();
                        let _ = iter.next().unwrap();
                        let arg_type = iter.next().unwrap();
                        if let (TT::Ident(arg_name), TT::Ident(arg_type)) = (arg_name, arg_type) {
                            (arg_name.clone(), arg_type.clone())
                        } else {
                            panic!();
                        }
                    }
                ).unzip();

                if let TT::Ident(return_type) = return_type {
                    return (name, args_vec.0, args_vec.1, return_type, expr.stream());
                } else {
                    panic!("Return type not Ident")
                }
            } else {
                panic!("args not Group")
            }
        } else {
            panic!("lit, type_sig, expr mismatch")
        }
    } else {
        panic!("Outer not enclosed in group");
    }
}
