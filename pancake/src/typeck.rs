use rsmt2::{Solver, SmtRes};

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
