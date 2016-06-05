use std::rc::{ Rc };

#[derive(Debug, Clone)]
enum Value
{
    Error,
    Atom(i32),
    Cell(Rc<Value>, Rc<Value>)
}

impl Value {
    fn atom_value(&self) -> Option<i32> {
        match self {
            &Value::Atom(a) => Some(a),
            _ => None
        }
    }

    fn cell_content(&self) -> Option<(Rc<Value>, Rc<Value>)> {
        match self {
            &Value::Cell(ref a, ref b) => Some((a.clone(), b.clone())),
            _ => None
        }
    }
}

fn kind(v: &Value) -> Value {
    match v {
        &Value::Error => Value::Error,
        &Value::Atom(_) => Value::Atom(0),
        &Value::Cell(_, _) => Value::Atom(1),
    }
}

fn sub(v: &Value) -> Value {
    match v {
        &Value::Error => Value::Error,

        // math- a
        &Value::Atom(v) => Value::Atom(-v),

        // math- [a, b]
        // math- [[a, b], c]
        // math- [a, [b, c]]
        // math- [[a, b], [c, d]]
        &Value::Cell(ref a, ref b) => sub_cell(&a, &b)
    }
}

// Equivalent to calling sub(Value::Cell(a, b))
fn sub_cell(a: &Value, b: &Value) -> Value {
    match (a, b) {
        // math- [a, b]
        (&Value::Atom(a), &Value::Atom(b)) => Value::Atom(a - b),

        // math- [[a, b], c] => [math- [a, c], math- [b, c]]
        (&Value::Cell(ref a, ref b), &Value::Atom(c)) => Value::Cell(
            Rc::new(sub_cell(&a, &Value::Atom(c))),
            Rc::new(sub_cell(&b, &Value::Atom(c)))
        ),

        // math- [a, [b, c]] => [math- [a, b], math- [a, c]]
        (&Value::Atom(a), &Value::Cell(ref b, ref c)) => Value::Cell(
            Rc::new(sub_cell(&Value::Atom(a), &b)),
            Rc::new(sub_cell(&Value::Atom(a), &c))
        ),

        // math- [[a, b], [c, d]] => [math- [a, c], math- [b, d]]
        (&Value::Cell(ref a, ref b), &Value::Cell(ref c, ref d)) => Value::Cell(
            Rc::new(sub_cell(&a, &c)),
            Rc::new(sub_cell(&b, &d)),
        ),

        _ => Value::Error
    }
}

fn eq(v: &Value) -> Value {
    match v {
        &Value::Error => Value::Error,

        // math= a => Error
        &Value::Atom(v) => Value::Error,

        // math= [a, a] => 1
        // math= [a, b] => 0
        // math= [[a, b], [c, d]] => math= [math= [a, c], math= [b, d]]
        &Value::Cell(ref a, ref b) => eq_cell(&a, &b)
    }
}

fn eq_cell(a: &Value, b: &Value) -> Value {
    match (a, b) {

        // math= [a, a] => 1
        // math= [a, b] => 0
        (&Value::Atom(a), &Value::Atom(b)) => Value::Atom(if a == b { 1 } else { 0 }),

        // math= [[a, b], [c, d]] => math= [math= [a, c], math= [b, d]]
        (&Value::Cell(ref a, ref b), &Value::Cell(ref c, ref d)) => eq_cell(
            &eq_cell(a, c),
            &eq_cell(b, d)
        ),

        //If we didn't find a structure for subtree equality this is just a case of `math=[a, b] => 0`
        _ => Value::Atom(0)
    }
}

fn swap(v: &Value) -> Value {
    match v {
        &Value::Error => Value::Error,
        &Value::Atom(a) => Value::Atom(a),
        &Value::Cell(ref a, ref b) => Value::Cell(b.clone(), a.clone())
    }
}

fn eval(v: &Value) -> Value {
    match v {
        &Value::Error => Value::Error,
        &Value::Atom(a) => Value::Error,

        &Value::Cell(ref a, ref b) => {
            match a.atom_value() {
                Some(a) => {
                    match (a, b) {
                        (0, _) => kind(&b),
                        (1, _) => sub(&b),
                        (2, _) => eq(&b),
                        (3, _) => swap(&b),

                        _ => Value::Error
                    }
                },
                _ => Value::Error
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{ Value, kind, sub, eq, swap, eval };
    use std::rc::{ Rc };

    #[test]
    fn kind_of_error_is_error() {
        // eval! [0, Error] => kind? Error => Error
        let r = eval(&Value::Cell(
            Rc::new(Value::Atom(0)),
            Rc::new(Value::Error)
        ));

        match r {
            Value::Error => assert!(true),
            _ => assert!(false)
        };
    }

    #[test]
    fn kind_of_cell_is_cell() {
        // eval! [0, Error] => kind? [1, 2] => 1
        let r = eval(&Value::Cell(
            Rc::new(Value::Atom(0)),
            Rc::new(Value::Cell(
                Rc::new(Value::Atom(1)),
                Rc::new(Value::Atom(2)),
            ))
        ));

        match r {
            Value::Atom(1) => assert!(true),
            _ => assert!(false)
        };
    }

    #[test]
    fn kind_of_atom_is_atom() {
        // eval! [0, 1] => kind? 1 => 0
        let r = eval(&Value::Cell(
            Rc::new(Value::Atom(0)),
            Rc::new(Value::Atom(1))
        ));

        match r {
            Value::Atom(0) => assert!(true),
            _ => assert!(false)
        };
    }

    #[test]
    fn math_sub_atom_is_negate() {
        let r = sub(&Value::Atom(-1));

        match r {
            Value::Atom(1) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn math_sub_error_is_error() {
        let r = sub(&Value::Error);

        match r {
            Value::Error => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn math_sub_cell_is_sub() {
        let r = sub(&Value::Cell(
            Rc::new(Value::Atom(1)),
            Rc::new(Value::Atom(2)),
        ));

        match r {
            Value::Atom(-1) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn math_sub_atom_from_cell() {
        let r = sub(&Value::Cell(
            Rc::new(Value::Cell(
                Rc::new(Value::Atom(1)),
                Rc::new(Value::Atom(2))
            )),
            Rc::new(Value::Atom(3))
        ));

        match r {
            Value::Cell(a, b) => {
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
        let r = sub(&Value::Cell(
            Rc::new(Value::Atom(3)),
            Rc::new(Value::Cell(
                Rc::new(Value::Atom(1)),
                Rc::new(Value::Atom(2))
            )),
        ));

        match r {
            Value::Cell(a, b) => {
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
        let r = sub(&Value::Cell(
            Rc::new(Value::Cell(
                Rc::new(Value::Atom(1)),
                Rc::new(Value::Atom(2))
            )),
            Rc::new(Value::Cell(
                Rc::new(Value::Atom(3)),
                Rc::new(Value::Atom(4))
            )),
        ));

        println!("{:?}", r);

        match r {
            Value::Cell(a, b) => {
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
        match eq(&Value::Error) {
            Value::Error => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_atom_is_error() {
        match eq(&Value::Atom(1)) {
            Value::Error => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_cell_is_equal_with_equal_atoms() {
        match eq(&Value::Cell(Rc::new(Value::Atom(1)), Rc::new(Value::Atom(1)))) {
            Value::Atom(1) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_cell_is_unequal_with_unequal_atoms() {
        match eq(&Value::Cell(Rc::new(Value::Atom(1)), Rc::new(Value::Atom(2)))) {
            Value::Atom(0) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn eq_cell_is_equal_with_equal_subtrees() {
        match eq(&Value::Cell(
            Rc::new(Value::Cell(
                Rc::new(Value::Atom(1)),
                Rc::new(Value::Atom(2)),
            )),
            Rc::new(Value::Cell(
                Rc::new(Value::Atom(1)),
                Rc::new(Value::Atom(2)),
            ))
        )) {
            Value::Atom(1) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn swap_cell_swaps_sides() {
        let v = swap(&Value::Cell(Rc::new(Value::Atom(1)), Rc::new(Value::Atom(2))));
        match v {
            Value::Cell(ref a, ref b) => {
                match (a.atom_value(), b.atom_value()) {
                    (Some(2), Some(1)) => assert!(true),
                    _ => assert!(false)
                }
            },
            _ => assert!(false)
        }
    }
}
