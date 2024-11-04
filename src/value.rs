use std::iter::FromIterator;
use std::{collections::HashMap, ops::Deref};
use rusty_v8::{self as v8, HandleScope};

use godot::meta::{FromGodot, GodotConvert, ToGodot};

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Object(Object),
    Array(Array),
    Undefined,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.coerce_to_string())
    }
}

impl GodotConvert for Value {
    type Via = godot::builtin::Variant;
}

impl ToGodot for Value {
    type ToVia<'v> = godot::builtin::Variant
    where
        Self: 'v;

    fn to_godot(&self) -> Self::ToVia<'_> {
        match self {
            Value::String(str) => godot::builtin::Variant::from(str.deref()),
            Value::Number(num) => godot::builtin::Variant::from(*num),
            Value::Object(_) => godot::builtin::Variant::nil(),
            Value::Array(_) => {
                godot::builtin::Variant::nil()
            }
            Value::Undefined => godot::builtin::Variant::nil(),
        }
    }
}

impl FromGodot for Value {
    fn try_from_godot(_via: Self::Via) -> Result<Self, godot::prelude::ConvertError> {
        todo!("godot to weaktype")
    }
}

impl std::ops::Add for &Value {
    type Output = Value;

    fn add(self, rhs: &Value) -> Self::Output {
        match self {
            Value::String(self_str) => match rhs {
                Value::String(rhs_str) => Value::from(self_str.to_owned() + &rhs_str),
                Value::Undefined => {
                    Value::from(self_str.to_owned() + &rhs.coerce_to_string())
                }
                Value::Object(_) => {
                    Value::from(self_str.to_owned() + &rhs.coerce_to_string())
                }

                Value::Number(rhs_num) => {
                    let self_coerced = self.coerce_to_number();
                    if self_coerced.is_nan() {
                        return Value::from(self_str.to_owned() + &rhs_num.to_string());
                    } else {
                        return Value::from(self_coerced + rhs_num);
                    }
                }
                Value::Array(_) => {
                    Value::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
            },

            Value::Number(self_num) => {
                let rhs_coerced = rhs.coerce_to_number();
                if rhs_coerced.is_nan() {
                    return Value::from(self.coerce_to_string() + &rhs.coerce_to_string());
                } else {
                    return Value::from(self_num + rhs_coerced);
                }
            }
            Value::Object(_) => match rhs {
                Value::String(rhs_str) => Value::from(self.coerce_to_string() + &rhs_str),
                Value::Number(_) => Value::from(f64::NAN),
                Value::Object(_) => {
                    Value::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
                Value::Undefined => {
                    Value::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
                Value::Array(_) => {
                    Value::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
            },
            Value::Undefined => match rhs {
                Value::String(rhs_str) => Value::from(self.coerce_to_string() + &rhs_str),
                Value::Number(_) => Value::from(f64::NAN),
                Value::Undefined => Value::from(f64::NAN),
                Value::Object(_) => {
                    Value::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
                Value::Array(_) => {
                    Value::from(self.coerce_to_string() + &rhs.coerce_to_string())
                }
            },
            Value::Array(_) => Value::from(self.coerce_to_string() + &rhs.coerce_to_string()),
        }
    }
}

impl std::ops::Index<&str> for Value {
    type Output = Value;
    fn index(&self, key: &str) -> &<Self as std::ops::Index<&str>>::Output {
        match self {
            Value::String(_) => &Value::Undefined,
            Value::Number(_) => &Value::Undefined,
            Value::Object(obj) => {
                if obj.0.contains_key(key) {
                    &obj.0[key]
                } else {
                    &Value::Undefined
                }
            }
            Value::Undefined => self,
            Value::Array(_) => &Value::Undefined,
        }
    }
}

impl Value {
    fn from<'a, T>(value: T) -> Value
    where
        T: Into<Value>,
    {
        value.into()
    }

    fn coerce_to_number(&self) -> f64 {
        match self {
            Value::String(str) => {
                let parsing_result = str.parse::<f64>();
                parsing_result.unwrap_or(f64::NAN)
            }
            Value::Number(num) => *num,
            Value::Object(_) => f64::NAN,
            Value::Undefined => f64::NAN,
            Value::Array(_) => f64::NAN,
        }
    }

    fn coerce_to_string(&self) -> String {
        match self {
            Value::String(str) => str.to_string(),
            Value::Number(num) => num.to_string(),
            Value::Object(_) => String::from("[object Object]"),
            Value::Undefined => String::from("undefined"),
            Value::Array(_) => {
                fn reducer(val: &Value) -> String {
                    match val {
                        Value::String(_) => val.coerce_to_string(),
                        Value::Number(_) => val.coerce_to_string(),
                        Value::Object(_) => val.coerce_to_string(),
                        Value::Array(arr) => arr
                            .0
                            .iter()
                            .map(reducer)
                            .reduce(|prev, cur| prev + ", " + &cur)
                            .unwrap(),
                        Value::Undefined => val.coerce_to_string(),
                    }
                }
                reducer(self)
            }
        }
    }

    pub fn as_local<'a>(&self, scope: &mut HandleScope<'a>) -> v8::Local<'a, v8::Value> {
        match self {
            Value::String(v) => v8::String::new(scope, v).unwrap().into(),
            Value::Number(_) => todo!(),
            Value::Object(_) => todo!(),
            Value::Array(_) => v8::String::new(scope, &self.coerce_to_string()).unwrap().into(),
            Value::Undefined => v8::undefined(scope).into(),
        }
    }
}

impl Into<Value> for f64 {
    fn into(self) -> Value {
        Value::Number(self)
    }
}

impl Into<Value> for i32 {
    fn into(self) -> Value {
        Value::Number(f64::from(self))
    }
}

impl Into<Value> for &str {
    fn into(self) -> Value {
        Value::String(self.to_string())
    }
}

impl Into<Value> for String {
    fn into(self) -> Value {
        Value::String(self)
    }
}

impl Into<Value> for HashMap<&'static str, Value> {
    fn into(self) -> Value {
        Value::Object(Object(self))
    }
}

impl Into<Value> for Object {
    fn into(self) -> Value {
        Value::Object(self)
    }
}

impl Into<Value> for Vec<Value> {
    fn into(self) -> Value {
        Value::Array(Array(self))
    }
}

impl Into<Value> for Array {
    fn into(self) -> Value {
        Value::Array(self)
    }
}

#[derive(Clone, Debug)]
pub struct Array(Vec<Value>);

impl Array {
    pub fn from(arr: &[Value]) -> Value {
        Value::from(Array(arr.to_vec()))
    }
    pub fn new(initial:Vec<Value>) -> Self{
        Array(initial)
    }
}

#[derive(Clone, Debug)]
pub struct Object(HashMap<&'static str, Value>);

pub trait FromValues<T> {
    fn from_values(value: T) -> Value;
}

impl<const N: usize> FromValues<[(&'static str, Value); N]> for Object {
    fn from_values(arr: [(&'static str, Value); N]) -> Value {
        Value::from(Object(HashMap::from_iter(arr)))
    }
}

#[allow(non_snake_case)]
pub fn Number(n: impl Into<Value>) -> Value {
    Value::from(n.into().coerce_to_number())
}

#[allow(non_snake_case)]
pub fn String(s: impl Into<Value>) -> Value {
    Value::from(s.into().coerce_to_string())
}
