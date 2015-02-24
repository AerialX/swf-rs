use avm1::{ AVM1Data, AVM1Error, AVM1ErrorKind };

pub trait AVM1Stack {
    fn data_push(&mut self, AVM1Data);
    fn data_pop(&mut self) -> Result<AVM1Data, AVM1Error>;
    fn data_peek(&self) -> Result<&AVM1Data, AVM1Error>;

    fn data_pop_string(&mut self) -> Result<String, AVM1Error> {
        Ok(convert_string(try!(self.data_pop())))
    }

    fn data_pop_f32(&mut self) -> Result<f32, AVM1Error> {
        Ok(convert_f32(try!(self.data_pop())))
    }

    fn data_pop_i32(&mut self) -> Result<i32, AVM1Error> {
        Ok(convert_i32(try!(self.data_pop())))
    }

    fn data_pop_bool(&mut self) -> Result<bool, AVM1Error> {
        Ok(convert_bool(try!(self.data_pop())))
    }

    fn data_operator_binary_f32<F : Fn(f32, f32) -> AVM1Data>(&mut self, f: F) -> Result<(), AVM1Error> {
        let op1 = try!(self.data_pop_f32());
        let op2 = try!(self.data_pop_f32());
        self.data_push(f(op1, op2));

        Ok(())
    }

    fn data_operator_unary_f32<F : Fn(f32) -> AVM1Data>(&mut self, f: F) -> Result<(), AVM1Error> {
        let op = try!(self.data_pop_f32());
        self.data_push(f(op));

        Ok(())
    }

    fn data_operator_unary_i32<F : Fn(i32) -> AVM1Data>(&mut self, f: F) -> Result<(), AVM1Error> {
        let op = try!(self.data_pop_i32());
        self.data_push(f(op));

        Ok(())
    }

    fn data_operator_binary_string<F : Fn(String, String) -> AVM1Data>(&mut self, f: F) -> Result<(), AVM1Error> {
        let op1 = try!(self.data_pop_string());
        let op2 = try!(self.data_pop_string());
        self.data_push(f(op1, op2));

        Ok(())
    }

    fn data_operator_unary_string<F : Fn(String) -> AVM1Data>(&mut self, f: F) -> Result<(), AVM1Error> {
        let op = try!(self.data_pop_string());
        self.data_push(f(op));

        Ok(())
    }
}

impl AVM1Stack for Vec<AVM1Data> {
    fn data_push(&mut self, data: AVM1Data) {
        self.push(data);
    }

    fn data_pop(&mut self) -> Result<AVM1Data, AVM1Error> {
        self.pop().ok_or(AVM1Error::new(AVM1ErrorKind::StackEmpty))
    }

    fn data_peek(&self) -> Result<&AVM1Data, AVM1Error> {
        if self.is_empty() { Err(AVM1Error::new(AVM1ErrorKind::StackEmpty)) } else { Ok(&self[self.len() - 1]) }
    }
}

fn convert_f32(d: AVM1Data) -> f32 {
    match d {
        AVM1Data::Boolean(v) => if v { 1.0 } else { 0.0 },
        AVM1Data::Integer(v) => v as f32,
        AVM1Data::Float(v) => v,
        AVM1Data::Double(v) => v as f32,
        AVM1Data::String(v) => v.parse().unwrap_or(0.0),
        _ => 0.0
    }
}

fn convert_i32(d: AVM1Data) -> i32 {
    match d {
        AVM1Data::Boolean(v) => if v { 1 } else { 0 },
        AVM1Data::Integer(v) => v,
        AVM1Data::Float(v) => v as i32,
        AVM1Data::Double(v) => v as i32,
        AVM1Data::String(v) => v.parse().unwrap_or(0),
        _ => 0
    }
}

fn convert_bool(d: AVM1Data) -> bool {
    match d {
        AVM1Data::Boolean(v) => v,
        AVM1Data::Integer(v) => if v == 0 { false } else { true },
        AVM1Data::Float(v) => if v == 0.0 { false } else { true },
        AVM1Data::Double(v) => if v == 0.0 { false } else { true },
        AVM1Data::String(_) => unimplemented!(),
        _ => false
    }
}

fn convert_string(d: AVM1Data) -> String {
    match d {
        AVM1Data::Boolean(v) => v.to_string(),
        AVM1Data::Integer(v) => v.to_string(),
        AVM1Data::Float(v) => v.to_string(),
        AVM1Data::Double(v) => v.to_string(),
        AVM1Data::String(v) => (*v).clone(),
        AVM1Data::Null => String::from_str("null"),
        AVM1Data::Undefined => String::from_str("undefined"),
        _ => String::new()
    }
}
