use rsmt2::{Solver, SmtRes};
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Term, Let, Literal};
use crate::vm2::{Context, Identifier};
use crate::inference_data::{TypeScheme, SimpleType, TypeVariable, TVarId, TVarRegistry};

type Ctx<'a> = &'a mut Context<TypeScheme>;
type TVars<'a> = &'a mut TVarRegistry;
type SimpleTypeRef = Rc<SimpleType>;

/// Constrains types to enforce subtyping relation `lhs` <: `rhs`
pub fn constrain(lhs: SimpleTypeRef, rhs: SimpleTypeRef, tvars: TVars) {
    use SimpleType::*;
    if Rc::ptr_eq(&lhs, &rhs) { return }
    // TODO: cache

    let mut added_constraints = Vec::new();

    let (lhs_match, rhs_match) = (&*lhs.clone(), &*rhs.clone());
    match (lhs_match, rhs_match) {
        (Function { lhs: l0, rhs: r0 }, Function { lhs: l1, rhs: r1 }) => {
            constrain(l1.clone(), l0.clone(), tvars);
            constrain(r0.clone(), r1.clone(), tvars);
        }
        (Record { fields: fs0 }, Record { fields: fs1}) => {
            for key in fs1.keys() {
                let type1 = fs1.get(key).unwrap().clone();
                let type0 = fs0.get(key).unwrap().clone();
                // constrain(type0, type1, tvars);
                added_constraints.push((type0, type1));
            }
        }
        (Variable(key), _) => {
            if rhs.level(tvars) <= tvars.get(&key).level {
                let lhs = tvars.get_mut(&key);
                lhs.upper_bounds.push(rhs.clone());
                for lower_bound in &lhs.lower_bounds {
                    // constrain(lower_bound.clone(), rhs.clone(), tvars);
                    added_constraints.push((lower_bound.clone(), rhs.clone()));
                }
            } else {
                let rhs = extrude(rhs, false, lhs.level(tvars), tvars,
                                  &mut HashMap::new());
                added_constraints.push((lhs, rhs))
            }
        }
        (_, Variable(key)) => {
            if lhs.level(tvars) <= tvars.get(&key).level {
                let rhs = tvars.get_mut(&key);
                rhs.lower_bounds.push(lhs.clone());
                for upper_bound in &rhs.upper_bounds {
                    added_constraints.push((lhs.clone(), upper_bound.clone()));
                }
            } else {
                let lhs = extrude(lhs, true, rhs.level(tvars), tvars,
                                  &mut HashMap::new());
                added_constraints.push((lhs, rhs))
            }
        }
        _ => { panic!(); }
    }
    for (lhs, rhs) in added_constraints {
        constrain(lhs, rhs, tvars);
    }
}

pub fn extrude(typ: SimpleTypeRef, polarity: bool, lvl: i32, tvars: TVars,
           cache: &mut HashMap<TVarId, TVarId>) -> SimpleTypeRef {
    use SimpleType::*;
    if typ.level(tvars) <= lvl {
        return typ;
    }
    match &*typ {
        Function { lhs, rhs } => Rc::new(Function {
            lhs: extrude(lhs.clone(), !polarity, lvl, tvars, cache),
            rhs: extrude(rhs.clone(), polarity, lvl, tvars, cache)
        }),
        Record { fields } => Rc::new(Record {
            fields: fields.iter().map(
                |(k, v)| (k.clone(), extrude(v.clone(), polarity, lvl, tvars, cache))).collect()
        }),
        Variable(key) => {
            let res = cache.get(key);
            if let Some(res) = res {
                Rc::new(Variable(*res))
            } else {
                let new_var = tvars.fresh_var(lvl);
                let key2 = new_var.tvar_id().unwrap();
                cache.insert(*key, key2);
                if polarity {
                    let t = tvars.get_mut(key);
                    t.upper_bounds.push(new_var.clone());
                    let t: &TypeVariable = t;
                    let new_var_lower = t.lower_bounds.clone().into_iter()
                        .map(|typ| extrude(typ.clone(), polarity, lvl, tvars, cache))
                        .collect();
                    tvars.get_mut(&key2).lower_bounds = new_var_lower;
                } else {
                    let t = tvars.get_mut(key);
                    t.lower_bounds.push(new_var.clone());
                    let t: &TypeVariable = t;
                    let new_var_upper = t.upper_bounds.clone().into_iter()
                        .map(|typ| extrude(typ.clone(), polarity, lvl, tvars, cache))
                        .collect();
                    tvars.get_mut(&key2).upper_bounds = new_var_upper;
                }
                new_var
            }
        }
        Primitive { name: _ } => typ.clone()
    }
}

pub fn infer_term(term: &Term, ctx: Ctx, tvars: TVars, lvl: i32) -> SimpleTypeRef {
    use Literal::*;
    use SimpleType::*;
    use Term::*;
    match term {
        Lit(lit) => match lit {
            Int(_) => Rc::new(Primitive { name: "int".into() }),
        }
        App(f, x) => {
            let res = tvars.fresh_var(lvl);
            let f_typ = infer_term(f, ctx, tvars, lvl);
            let x_typ = infer_term(x, ctx, tvars, lvl);
            constrain(
                f_typ,
                Rc::new(Function {
                    lhs: x_typ,
                    rhs: res.clone()
                }),
                tvars
            );
            res
        }
        Let { binding, body } => {
            let typ = infer_let(binding, ctx, tvars, lvl);
            ctx.with_binding(&binding.name, typ,
                             |ctx2| infer_term(body, ctx2, tvars, lvl))
        }
        Var(name) => match ctx.get(name) {
            Some(t) => t.instantiate(lvl),
            None => panic!("Variable {} not found", name)
        }
        Lam(name, body) => {
            let param = tvars.fresh_var(lvl);
            let body_typ = ctx.with_binding(name, param.into(), |ctx2|
                infer_term(body, ctx2, tvars, lvl));
            Rc::new(Function {
                lhs: param,
                rhs: body_typ
            })
        }
    }
}

pub fn infer_let(binding: &Let, ctx: Ctx, tvars: TVars, lvl: i32) -> TypeScheme {
    use TypeScheme::*;
    let res = if binding.rec {
        let e = tvars.fresh_var(lvl + 1);
        let typ = ctx.with_binding(&binding.name, SimpleType(e.clone()),
            |ctx2| infer_term(&binding.rhs, ctx2, tvars, lvl+1));
        constrain(typ, e.clone(), tvars);
        e
    } else {
        infer_term(&binding.rhs, ctx, tvars, lvl+1)
    };

    TypeScheme::PolymorphicType {
        level: lvl,
        body: res
    }
}

// fn infer_types()

#[test]
pub fn test_smt() -> SmtRes<()> {
    let mut solver = Solver::default(())?;

    solver.declare_const("n", "Int")?;
    solver.declare_const("m", "Int")?;
    solver.define_fun("sq", & [ ("n", "Int") ], "Int", "(* n n)")?;
    solver.assert("(= (+ (sq n) (sq m)) 7)")?;

    let is_sat = solver.check_sat()?;
    assert! (! is_sat);
    Ok(())
}

#[test]
pub fn test_smt2() -> SmtRes<()> {
    let mut solver = Solver::default(())?;

    solver.declare_sort("Type", 0)?;
    solver.declare_const("int", "Type")?;

    // let dummy: [&str; 1] = [""];
    // solver.declare_datatypes( & [
    //     ( "Fn", 0, dummy,
    //        ["((Pair (mk-pair (first Type) (second Type))))" ] ),
    // ] )?;

    // solver.define_fun("can_apply", &[("f", "Fn"), ("t", "Type")], "Bool", "(= t (first f))")?;

    // solver.define_const("inc", "Fn", "(Fn int int)")?;
    // solver.assert("(can_apply inc int)")?;

    let is_sat = solver.check_sat()?;
    assert! (is_sat);
    Ok(())
}

#[test]
pub fn test_inference() {
    use SimpleType::*;
    use Term::*;
    let mut ctx: Context<TypeScheme> = Context::new();
    let bool_type: Rc<_> = Primitive { name: "bool".into() }.into();
    let not_type = Function { lhs: bool_type.clone(),
                              rhs: bool_type.clone() }.into();
    ctx.insert("true", bool_type.clone().into());
    ctx.insert("not", not_type);

    let mut tvars = TVarRegistry::new();

    let not_true_type =
        infer_term(&App(Box::new(Var("not".into())),
                        Box::new(Var("true".into()))), &mut ctx, &mut tvars, 0);

    if let Variable(key) = *not_true_type {
        assert_eq!(tvars.get(&key).lower_bounds.len(), 1);
        assert_eq!(*tvars.get(&key).lower_bounds.get(0).unwrap(),
                   bool_type.clone());
    }
}
