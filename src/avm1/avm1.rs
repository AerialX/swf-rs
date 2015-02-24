use std::num;
use std::char;
use std::rc::Rc;
use std::cell::RefCell;
use std::num::Float;

use avm1::AVM1ActionKind as AVM1Action;
use avm1::AVM1Data as Data;
use avm1::{ AVM1ProgramCounter, AVM1Syscalls, AVM1Context, AVM1Error, AVM1ErrorKind };
use avm1::stack::AVM1Stack;

pub fn avm1_execute<S : AVM1Syscalls>(ctx: &mut AVM1Context, pc: &mut AVM1ProgramCounter, syscalls: S, step: bool) -> Result<(), AVM1Error> {
    let version = ctx.version;

    loop {
        if let Some(op) = num::from_u8(pc.opcode()) { match op {
            // SWF 3
            AVM1Action::End => break,
            AVM1Action::GotoFrame => syscalls.goto_frame(try!(pc.operand_u16())),
            AVM1Action::GetURL => {
                let url = try!(pc.operand_string());
                let target = try!(pc.operand_string());
                syscalls.get_url(&url, &target);
            },
            AVM1Action::NextFrame => syscalls.next_frame(),
            AVM1Action::PreviousFrame => syscalls.previous_frame(),
            AVM1Action::Play => syscalls.play(),
            AVM1Action::Stop => syscalls.stop(),
            AVM1Action::ToggleQuality => syscalls.toggle_quality(),
            AVM1Action::StopSounds => syscalls.stop_sounds(),
            AVM1Action::WaitForFrame => {
                let frame = try!(pc.operand_u16());
                let skip = try!(pc.operand_u8());
                if !syscalls.wait_for_frame(frame) {
                    for _ in 0..skip {
                        pc.increment();
                    }
                }
            },
            AVM1Action::SetTarget => ctx.target = try!(pc.operand_string()),
            AVM1Action::GoToLabel => syscalls.goto_label(&try!(pc.operand_string())),

            // SWF 4
            AVM1Action::Push => {
                while pc.data_left() > 0 {
                    let kind = try!(pc.operand_u8());
                    ctx.stack.push(match num::from_u8(kind) {
                        Some(AVM1StackDataType::StringLiteral) => Data::String(Rc::new(try!(pc.operand_string()))),
                        Some(AVM1StackDataType::FloatLiteral) => Data::Float(try!(pc.operand_f32())),
                        Some(AVM1StackDataType::Null) => Data::Null,
                        Some(AVM1StackDataType::Undefined) => Data::Undefined,
                        Some(AVM1StackDataType::Register) => ctx.registers[try!(pc.operand_u8()) as usize].clone(),
                        Some(AVM1StackDataType::Boolean) => Data::Boolean(try!(pc.operand_u8()) != 0),
                        Some(AVM1StackDataType::Double) => Data::Double(try!(pc.operand_f64())),
                        Some(AVM1StackDataType::Integer) => Data::Integer(try!(pc.operand_u32()) as i32),
                        Some(AVM1StackDataType::Constant8) => ctx.constant_pool[try!(pc.operand_u8()) as usize].clone(),
                        Some(AVM1StackDataType::Constant16) => ctx.constant_pool[try!(pc.operand_u16()) as usize].clone(),
                        None => return Err(AVM1Error::new(AVM1ErrorKind::InvalidOperand))
                    });
                }
            },
            AVM1Action::Pop => { ctx.stack.pop(); },
            AVM1Action::Add => try!(ctx.stack.data_operator_binary_f32(|op1, op2| Data::Float(op1 + op2))),
            AVM1Action::Subtract => try!(ctx.stack.data_operator_binary_f32(|op1, op2| Data::Float(op2 - op1))),
            AVM1Action::Multiply => try!(ctx.stack.data_operator_binary_f32(|op1, op2| Data::Float(op1 * op2))),
            AVM1Action::Divide => try!(ctx.stack.data_operator_binary_f32(|op1, op2| swf_f32(version, op2 / op1))),
            AVM1Action::Equals => try!(ctx.stack.data_operator_binary_f32(|op1, op2| swf_boolean(version, op1 == op2))),
            AVM1Action::Less => try!(ctx.stack.data_operator_binary_f32(|op1, op2| swf_boolean(version, op2 < op1))),
            AVM1Action::And => try!(ctx.stack.data_operator_binary_f32(|op1, op2| swf_boolean(version, op1 != 0.0 && op2 != 0.0))),
            AVM1Action::Or => try!(ctx.stack.data_operator_binary_f32(|op1, op2| swf_boolean(version, op1 != 0.0 || op2 != 0.0))),
            AVM1Action::Not => try!(ctx.stack.data_operator_unary_f32(|op| swf_boolean(version, op == 0.0))),
            AVM1Action::StringEquals => try!(ctx.stack.data_operator_binary_string(|op1, op2| swf_boolean(version, op1 == op2))),
            AVM1Action::StringLength => try!(ctx.stack.data_operator_unary_string(|op| Data::Integer(op.len() as i32))),
            AVM1Action::StringAdd => try!(ctx.stack.data_operator_binary_string(|op1, op2| Data::String(Rc::new(op1 + &op2)))),
            AVM1Action::StringExtract => {
                let count = try!(ctx.stack.data_pop_i32());
                let index = try!(ctx.stack.data_pop_i32());
                let value = try!(ctx.stack.data_pop_string());
                ctx.stack.push(Data::String(Rc::new(if count <= 0 || index < 0 || index as usize >= value.len() || (index + count) as usize > value.len() {
                    String::new()
                } else {
                    let mut vec = Vec::<u8>::with_capacity(count as usize);
                    vec.push_all(&value.as_bytes()[index as usize .. (index + count) as usize]);
                    String::from_utf8(vec).unwrap_or(String::new())
                })));
            },
            AVM1Action::StringLess => try!(ctx.stack.data_operator_binary_string(|op1, op2| swf_boolean(version, op2 < op1))),
            AVM1Action::MBStringLength => try!(ctx.stack.data_operator_unary_string(|op| Data::Integer(op[..].chars().count() as i32))),
            AVM1Action::MBStringExtract => {
                let count = try!(ctx.stack.data_pop_i32());
                let index = try!(ctx.stack.data_pop_i32());
                let _value = try!(ctx.stack.data_pop_string());
                let value = &_value[..];
                ctx.stack.push(Data::String(Rc::new(if count <= 0 || index < 0 || index as usize >= value.len() || (index + count) as usize > value.len() {
                    String::new()
                } else {
                    String::from_str(&value.slice_chars(index as usize, (index + count) as usize))
                })));
            },
            AVM1Action::ToInteger => try!(ctx.stack.data_operator_unary_i32(|op| Data::Integer(op))),
            AVM1Action::CharToAscii => try!(ctx.stack.data_operator_unary_string(|op| {
                let op = &op[..];
                Data::Integer(if op.len() > 0 { op.char_at(0) as i32 & 0xff } else { 0 })
            })),
            AVM1Action::AsciiToChar => try!(ctx.stack.data_operator_unary_i32(|op| {
                Data::String(Rc::new(String::from_utf8(vec![op as u8]).unwrap()))
            })),
            AVM1Action::MBCharToAscii => try!(ctx.stack.data_operator_unary_string(|op| {
                let op = &op[..];
                Data::Integer(if op.len() > 0 { op.char_at(0) as i32 } else { 0 })
            })),
            AVM1Action::MBAsciiToChar => try!(ctx.stack.data_operator_unary_i32(|op| {
                let mut s = String::new();
                if let Some(c) = char::from_u32(op as u32) {
                    s.push(c);
                }
                Data::String(Rc::new(s))
            })),
            AVM1Action::Jump => {
                let offset = try!(pc.operand_u16());
                pc.increment();
                pc.offset(offset);
            },
            AVM1Action::If => {
                let offset = try!(pc.operand_u16());
                if try!(ctx.stack.data_pop_bool()) {
                    pc.increment();
                    pc.offset(offset);
                }
            },
            AVM1Action::Call => syscalls.call(&try!(ctx.stack.data_pop_string())),
            AVM1Action::GetVariable => {
                let name = try!(ctx.stack.data_pop_string());
                // TODO: A variable in another execution context can be referenced by prefixing the variable name with the target path and a colon
                //       For example: /A/B:FOO references variable FOO in a movie clip with a target path of /A/B.
                ctx.stack.push(ctx.variables.get(&name).unwrap_or(&Data::Undefined).clone());
            }
            AVM1Action::SetVariable => {
                let value = try!(ctx.stack.data_pop());
                let name = try!(ctx.stack.data_pop_string());
                ctx.variables.insert(name, value);
            },
            AVM1Action::GetURL2 => {
                let field = try!(pc.operand_u8());
                let method = (field & 0xc0) >> 6; // 0 = none, 1 = GET, 2 = POST
                // let reserved = (field & 0x3c) >> 2; // always 0
                let target_sprite = field & 0x02 != 0; // false = target is browser window, true = target is path to sprite
                let load_variables = field & 0x01 != 0; // 0 = no variables to load, 1 = load variables
                let target = try!(ctx.stack.data_pop_string());
                let url = try!(ctx.stack.data_pop_string());
                syscalls.get_url2(&target, &url, method, target_sprite, load_variables);
            },
            AVM1Action::GotoFrame2 => {
                let field = try!(pc.operand_u8());
                // let reserved = (field & 0x8c) >> 2; // always 0
                let play = field & 0x01 != 0;
                let scene_bias = if field & 0x02 != 0 { try!(pc.operand_u16()) } else { 0 };
                let frame = try!(ctx.stack.data_pop_string());
                syscalls.goto_frame2(&frame, scene_bias, play);
            },
            AVM1Action::SetTarget2 => ctx.target = try!(ctx.stack.data_pop_string()),
            AVM1Action::GetProperty => {
                let index = try!(ctx.stack.data_pop_i32());
                let target = try!(ctx.stack.data_pop_string());
                ctx.stack.push(syscalls.get_property(&target, index as u32));
            },
            AVM1Action::SetProperty => {
                let value = try!(ctx.stack.data_pop());
                let index = try!(ctx.stack.data_pop_i32());
                let target = try!(ctx.stack.data_pop_string());
                syscalls.set_property(&target, index as u32, &value);
            },
            AVM1Action::CloneSprite => {
                let depth = try!(ctx.stack.data_pop_f32());
                let target = try!(ctx.stack.data_pop_string());
                let source = try!(ctx.stack.data_pop_string());
                syscalls.clone_sprite(&source, &target, depth);
            },
            AVM1Action::RemoveSprite => syscalls.remove_sprite(&try!(ctx.stack.data_pop_string())),
            AVM1Action::StartDrag => {
                let target = try!(ctx.stack.data_pop_string());
                let look_centre = try!(ctx.stack.data_pop_f32());
                let constrain = try!(ctx.stack.data_pop_bool());
                if constrain {
                    try!(ctx.stack.data_pop()); // y2
                    try!(ctx.stack.data_pop()); // x2
                    try!(ctx.stack.data_pop()); // y1
                    try!(ctx.stack.data_pop()); // x1
                }
                syscalls.start_drag(&target, look_centre);
            }
            AVM1Action::EndDrag => syscalls.end_drag(),
            AVM1Action::WaitForFrame2 => {
                let frame = try!(ctx.stack.data_pop_string());
                let skip = try!(pc.operand_u8());
                if !syscalls.wait_for_frame2(&frame) {
                    for _ in 0..skip {
                        pc.increment();
                    }
                }
            },
            AVM1Action::Trace => syscalls.trace(&try!(ctx.stack.data_pop_string())),
            AVM1Action::GetTime => ctx.stack.push(Data::Integer(syscalls.get_time() as i32)),
            AVM1Action::RandomNumber => {
                let v = syscalls.random_number(try!(ctx.stack.data_pop_i32()) as u32);
                ctx.stack.push(Data::Integer(v as i32));
            },

            // SWF 5
            AVM1Action::ConstantPool => {
                let count = try!(pc.operand_u16());
                ctx.constant_pool.clear();
                for _ in 0..count {
                    ctx.constant_pool.push(Data::String(Rc::new(try!(pc.operand_string()))));
                }
            },
            AVM1Action::InitArray => {
                let count = try!(ctx.stack.data_pop_i32()) as usize;
                let mut array = Vec::<Data>::with_capacity(count);
                for _ in 0..count {
                    array.push(try!(ctx.stack.data_pop()));
                }
                ctx.stack.push(Data::Array(Rc::new(RefCell::new(array))));
            },
            AVM1Action::GetMember => {
                let name = try!(ctx.stack.data_pop_string());
                let obj = try!(ctx.stack.data_pop());
                ctx.stack.push(match obj {
                    Data::Object(v) => v.borrow_mut().get(&name).unwrap_or(&Data::Undefined).clone(),
                    _ => Data::Null
                });
            },
            AVM1Action::SetMember => {
                let value = try!(ctx.stack.data_pop());
                let name = try!(ctx.stack.data_pop_string());
                let obj = try!(ctx.stack.data_pop());
                match obj {
                    Data::Object(v) => { v.borrow_mut().insert(name, value); },
                    _ => return Err(AVM1Error::new(AVM1ErrorKind::TypeMismatch))
                }
            },
            AVM1Action::PushDuplicate => {
                let value = try!(ctx.stack.data_peek()).clone();
                ctx.stack.push(value);
            },
            AVM1Action::StoreRegister => {
                let index = try!(pc.operand_u8());
                ctx.registers[index as usize] = try!(ctx.stack.data_peek()).clone();
            },

            _ => unimplemented!()
        } } else {
            return Err(AVM1Error::new(AVM1ErrorKind::InvalidOpcode))
        }

        pc.increment();
        if step {
            break
        }
    }

    Ok(())
}

fn swf_boolean(version: u8, v: bool) -> Data {
    match version {
        0...4 => Data::Integer(if v == true { 1 } else { 0 }),
        _ => Data::Boolean(v)
    }
}

fn swf_f32(version: u8, v: f32) -> Data {
    match version {
        0...4 if !v.is_finite() => Data::String(Rc::new("#ERROR#".to_string())),
        _ => Data::Float(v)
    }
}

#[derive(FromPrimitive)]
enum AVM1StackDataType {
    StringLiteral = 0,
    FloatLiteral  = 1,
    Null          = 2,
    Undefined     = 3,
    Register      = 4,
    Boolean       = 5,
    Double        = 6,
    Integer       = 7,
    Constant8     = 8,
    Constant16    = 9
}
