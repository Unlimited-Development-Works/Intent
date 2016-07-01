use std::rc::{ Rc };

#[derive(Debug, Clone)]
enum Value
{
    Error,
    Atom(i32),
    Cell(Rc<Value>, Rc<Value>)
}

use self::Value::{ Error, Atom, Cell };

impl Value {
    fn atom_value(&self) -> Option<i32> {
        match self {
            &Atom(a) => Some(a),
            _ => None
        }
    }

    fn cell_content(&self) -> Option<(Rc<Value>, Rc<Value>)> {
        match self {
            &Cell(ref a, ref b) => Some((a.clone(), b.clone())),
            _ => None
        }
    }
}

fn kind(v: &Value) -> Value {
    match v {
        &Error => Error,
        &Atom(_) => Atom(0),
        &Cell(_, _) => Atom(1),
    }
}

fn sub(v: &Value) -> Value {
    match v {
        &Error => Error,

        // math- a
        &Atom(v) => Atom(-v),

        // math- [a, b]
        // math- [[a, b], c]
        // math- [a, [b, c]]
        // math- [[a, b], [c, d]]
        &Cell(ref a, ref b) => sub_cell(&a, &b)
    }
}

// Equivalent to calling sub(Cell(a, b))
fn sub_cell(a: &Value, b: &Value) -> Value {
    match (a, b) {
        // math- [a, b]
        (&Atom(a), &Atom(b)) => Atom(a - b),

        // math- [[a, b], c] => [math- [a, c], math- [b, c]]
        (&Cell(ref a, ref b), &Atom(c)) => Cell(
            Rc::new(sub_cell(&a, &Atom(c))),
            Rc::new(sub_cell(&b, &Atom(c)))
        ),

        // math- [a, [b, c]] => [math- [a, b], math- [a, c]]
        (&Atom(a), &Cell(ref b, ref c)) => Cell(
            Rc::new(sub_cell(&Atom(a), &b)),
            Rc::new(sub_cell(&Atom(a), &c))
        ),

        // math- [[a, b], [c, d]] => [math- [a, c], math- [b, d]]
        (&Cell(ref a, ref b), &Cell(ref c, ref d)) => Cell(
            Rc::new(sub_cell(&a, &c)),
            Rc::new(sub_cell(&b, &d)),
        ),

        _ => Error
    }
}

fn eq(v: &Value) -> Value {
    match v {
        &Error => Error,

        // math= a => Error
        &Atom(v) => Error,

        // math= [a, a] => 1
        // math= [a, b] => 0
        // math= [[a, b], [c, d]] => math= [math= [a, c], math= [b, d]]
        &Cell(ref a, ref b) => eq_cell(&a, &b)
    }
}

fn eq_cell(a: &Value, b: &Value) -> Value {
    match (a, b) {

        // math= [a, a] => 1
        // math= [a, b] => 0
        (&Atom(a), &Atom(b)) => Atom(if a == b { 1 } else { 0 }),

        // math= [[a, b], [c, d]] => math= [math= [a, c], math= [b, d]]
        (&Cell(ref a, ref b), &Cell(ref c, ref d)) => eq_cell(
            &eq_cell(a, c),
            &eq_cell(b, d)
        ),

        //If we didn't find a structure for subtree equality this is just a case of `math=[a, b] => 0`
        _ => Atom(0)
    }
}

fn swap(v: &Value) -> Value {
    match v {
        &Error => Error,
        &Atom(a) => Atom(a),
        &Cell(ref a, ref b) => Cell(b.clone(), a.clone())
    }
}

fn eval(v: &Value) -> Rc<Value> {
    match v {
        &Error => Rc::new(Error),
        &Atom(a) => Rc::new(Error),
        &Cell(ref a, ref b) => eval_cell(&a, &b)
    }
}

fn eval_cell(a: &Value, b: &Value) -> Rc<Value> {
    match (a.atom_value(), b) {
        (None, _) => Rc::new(Error),
        (Some(0), _) => Rc::new(kind(&b)),
        (Some(1), _) => Rc::new(sub(&b)),
        (Some(2), _) => Rc::new(eq(&b)),
        (Some(3), _) => Rc::new(swap(&b)),

        // eval! [4, [a, [b, c]] => eval! [ eval! [a, b], eval! [a, c] ]
        (Some(4), &Cell(ref a, ref bc)) => {
            match **bc {
                Error => Rc::new(Error),
                Atom(_) => Rc::new(Error),
                Cell(ref b, ref c) => Rc::new(eval_cell(
                    &eval_cell(&a, &b),
                    &eval_cell(&a, &c)
                ))
            }
        },

        // eval! [5, [0, [b, c]]] => b
        // eval! [5, [1, [b, c]]] => c
        (Some(5), &Cell(ref a, ref bc)) => {
            match (a.atom_value(), bc.cell_content()) {
                (Some(0), Some((ref b, ref c))) => Rc::new(b),
                (Some(1), Some((ref b, ref c))) => Rc::new(c),
                _ => Rc::new(Error)
            }
        }

        _ => Rc::new(Error)
    }
}

#[cfg(test)]
mod tests {

    use super::{ Value, kind, sub, eq, swap, eval };
    use std::rc::{ Rc };

    #[test]
    fn kind_of_error_is_error() {
        // eval! [0, Error] => kind? Error => Error
        let r = eval(&Cell(
            Rc::new(Atom(0)),
            Rc::new(Error)
        ));

        match r {
            Error => assert!(true),
            _ => assert!(false)
        };
    }

    #[test]
    fn kind_of_cell_is_cell() {
        // eval! [0, Error] => kind? [1, 2] => 1
        let r = eval(&Cell(
            Rc::new(Atom(0)),
            Rc::new(Cell(
                Rc::new(Atom(1)),
                Rc::new(Atom(2)),
            ))
        ));

        match r {
            Atom(1) => assert!(true),
            _ => assert!(false)
        };
    }

    #[test]
    fn kind_of_atom_is_atom() {
        // eval! [0, 1] => kind? 1 => 0
        let r = eval(&Cell(
            Rc::new(Atom(0)),
            Rc::new(Atom(1))
        ));

        match r {
            Atom(0) => assert!(true),
            _ => assert!(false)
        };
    }

    #[test]
    fn math_sub_atom_is_negate() {
        let r = sub(&Atom(-1));

        match r {
            Atom(1) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn math_sub_error_is_error() {
        let r = sub(&Error);

        match r {
            Error => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn math_sub_cell_is_sub() {
        let r = sub(&Cell(
            Rc::new(Atom(1)),
            Rc::new(Atom(2)),
        ));

        match r {
            Atom(-1) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn math_sub_atom_from_cell() {
        let r = sub(&Cell(
            Rc::new(Cell(
                Rc::new(Atom(1)),
                Rc::new(Atom(2))
            )),
            Rc::new(Atom(3))
        ));

        match r {
            Cell(a, b) => {
                match (a.atom_value(), b.atom_value()) {
                    (Some(-2), Some(-1)) => assert!(true),
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn math_sub_cell_from_atom() {
        let r = sub(&Cell(
            Rc::new(Atom(3)),
            Rc::new(Cell(
                Rc::new(Atom(1)),
                Rc::new(Atom(2))
            )),
        ));

        match r {
            Cell(a, b) => {
                match (a.atom_value(), b.atom_value()) {
                    (Some(2), Some(1)) => assert!(true),
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn math_sub_cell_from_cell() {
        let r = sub(&Cell(
            Rc::new(Cell(
                Rc::new(Atom(1)),
                Rc::new(Atom(2))
            )),
            Rc::new(Cell(
                Rc::new(Atom(3)),
                Rc::new(Atom(4))
            )),
        ));

        println!("{:?}", r);

        match r {
            Cell(a, b) => {
                match (a.atom_value(), b.atom_value()) {
                    (Some(-2), Some(-2)) => assert!(true),
                    _ => assert!(false)
                }
            }
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_error_is_error() {
        match eq(&Error) {
            Error => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_atom_is_error() {
        match eq(&Atom(1)) {
            Error => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_cell_is_equal_with_equal_atoms() {
        match eq(&Cell(Rc::new(Atom(1)), Rc::new(Atom(1)))) {
            Atom(1) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_cell_is_unequal_with_unequal_atoms() {
        match eq(&Cell(Rc::new(Atom(1)), Rc::new(Atom(2)))) {
            Atom(0) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_cell_is_equal_with_equal_subtrees() {
        match eq(&Cell(
            Rc::new(Cell(
                Rc::new(Atom(1)),
                Rc::new(Atom(2)),
            )),
            Rc::new(Cell(
                Rc::new(Atom(1)),
                Rc::new(Atom(2)),
            ))
        )) {
            Atom(1) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn swap_cell_swaps_sides() {
        let v = swap(&Cell(Rc::new(Atom(1)), Rc::new(Atom(2))));
        match v {
            Cell(ref a, ref b) => {
                match (a.atom_value(), b.atom_value()) {
                    (Some(2), Some(1)) => assert!(true),
                    _ => assert!(false)
                }
            },
            _ => assert!(false)
        }
    }
}
