// Copyright (c) 2018, [Ribose Inc](https://www.ribose.com).
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// ``AS IS'' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use super::{ParseError, Value};
use std::str::FromStr;
use std::u32;

pub fn apply(name: &str, args: &[Value]) -> Result<Value, ParseError> {
    match name {
        "add" => add(args),
        "subtract" => subtract(args),
        "divide" => divide(args),
        "multiply" => multiply(args),
        "power" => power(args),
        "intdiv" => intdiv(args),
        "modulus" => modulus(args),
        "concat" => concat(args),
        "join" => join(args),
        _ => Err(error("No such function")),
    }
}

pub fn add(args: &[Value]) -> Result<Value, ParseError> {
    convert::<i64>(args)
        .map(|(lhs, rhs)| (lhs + rhs).to_string())
        .or_else(|_| convert::<f64>(args).map(|(lhs, rhs)| (lhs + rhs).to_string()))
        .map_err(|_| error("Addition requires two numeric arguments"))
        .map(Value::String)
}

pub fn subtract(args: &[Value]) -> Result<Value, ParseError> {
    convert::<i64>(args)
        .map(|(lhs, rhs)| (lhs - rhs).to_string())
        .or_else(|_| convert::<f64>(args).map(|(lhs, rhs)| (lhs - rhs).to_string()))
        .map_err(|_| error("Subtraction requires two numeric arguments"))
        .map(Value::String)
}

pub fn multiply(args: &[Value]) -> Result<Value, ParseError> {
    convert::<i64>(args)
        .map(|(lhs, rhs)| (lhs * rhs).to_string())
        .or_else(|_| convert::<f64>(args).map(|(lhs, rhs)| (lhs * rhs).to_string()))
        .map_err(|_| error("Multiplication requires two numeric arguments"))
        .map(Value::String)
}

pub fn divide(args: &[Value]) -> Result<Value, ParseError> {
    convert::<f64>(args)
        .map(|(lhs, rhs)| (lhs / rhs).to_string())
        .map_err(|_| error("Division requires two numeric arguments"))
        .map(Value::String)
}

pub fn power(args: &[Value]) -> Result<Value, ParseError> {
    convert::<i64>(args)
        .and_then(|(lhs, rhs)| {
            if rhs > 0 && rhs <= i64::from(u32::MAX) {
                Ok(lhs.pow(rhs as u32).to_string())
            } else {
                Err(())
            }
        }).or_else(|_| convert::<f64>(args).map(|(lhs, rhs)| lhs.powf(rhs).to_string()))
        .map_err(|_| error("Power requires two numeric arguments"))
        .map(Value::String)
}

pub fn intdiv(args: &[Value]) -> Result<Value, ParseError> {
    convert::<i64>(args)
        .map(|(lhs, rhs)| (lhs / rhs).to_string())
        .map_err(|_| error("Integer division requires two integer arguments"))
        .map(Value::String)
}

pub fn modulus(args: &[Value]) -> Result<Value, ParseError> {
    convert::<i64>(args)
        .map(|(lhs, rhs)| (lhs % rhs).to_string())
        .map_err(|_| error("Modulus requires two integer arguments"))
        .map(Value::String)
}

fn concat(args: &[Value]) -> Result<Value, ParseError> {
    args.iter()
        .try_fold(String::new(), |mut s, a| {
            a.as_str().map(|a| {
                s.push_str(a);
                s
            })
        }).map(Value::String)
        .ok_or_else(|| error("Concat only takes string arguments"))
}

fn join(args: &[Value]) -> Result<Value, ParseError> {
    args.iter()
        .try_fold(Vec::new(), |mut v, a| {
            a.as_str().map(|a| {
                v.push(a);
                v
            })
        }).ok_or_else(|| error("Join only takes string arguments"))
        .and_then(|strs| {
            if strs.is_empty() {
                Err(error("Not enough arguments to join"))
            } else {
                Ok(Value::String(strs[1..].join(strs[0])))
            }
        })
}

fn convert<T: FromStr>(args: &[Value]) -> Result<(T, T), ()> {
    args.get(2)
        .map_or_else(|| Ok(()), |_| Err(()))
        .and_then(|_| {
            args.get(0)
                .and_then(|lhs| args.get(1).map(|rhs| (lhs, rhs)))
                .ok_or(())
        }).and_then(|(lhs, rhs)| {
            lhs.as_str()
                .and_then(|lhs| rhs.as_str().map(|rhs| (lhs, rhs)))
                .ok_or(())
        }).and_then(|(lhs, rhs)| {
            lhs.parse::<T>()
                .and_then(|lhs| rhs.parse::<T>().map(|rhs| (lhs, rhs)))
                .map_err(|_| ())
        })
}

fn error(reason: &'static str) -> ParseError {
    ParseError {
        reason,
        positions: Vec::new(),
    }
}
