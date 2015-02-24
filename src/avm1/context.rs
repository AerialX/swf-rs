use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AVM1Data {
    Object(Rc<RefCell<HashMap<String, AVM1Data>>>),
    Array(Rc<RefCell<Vec<AVM1Data>>>),
    Boolean(bool),
    Integer(i32),
    Float(f32),
    Double(f64),
    String(Rc<String>),
    Undefined,
    Null,
}

impl AVM1Data {
    pub fn new_object(v: HashMap<String, Self>) -> Self {
        AVM1Data::Object(Rc::new(RefCell::new(v)))
    }

    pub fn object(&self) -> Option<Rc<RefCell<HashMap<String, Self>>>> {
        match self {
            &AVM1Data::Object(ref v) => Some(v.clone()),
            _ => None
        }
    }

    pub fn array(&self) -> Option<Rc<RefCell<Vec<Self>>>> {
        match self {
            &AVM1Data::Array(ref v) => Some(v.clone()),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct AVM1Context {
    pub version: u8,
    pub registers: [AVM1Data; 4],
    pub variables: HashMap<String, AVM1Data>,
    pub constant_pool: Vec<AVM1Data>,
    pub stack: Vec<AVM1Data>,
    pub target: String,
}

pub trait AVM1Syscalls {
    // SWF 3
    fn goto_frame(&self, frame: u16);
    fn get_url(&self, url: &String, target: &String);
    fn next_frame(&self);
    fn previous_frame(&self);
    fn play(&self);
    fn stop(&self);
    fn toggle_quality(&self);
    fn stop_sounds(&self);
    fn wait_for_frame(&self, frame: u16) -> bool;
    fn goto_label(&self, label: &String);

    // SWF 4
    fn call(&self, target: &String);
    fn get_url2(&self, target: &String, url: &String, method: u8, target_sprite: bool, load_variables: bool);
    fn goto_frame2(&self, frame: &String, scene_bias: u16, play: bool);
    fn get_property(&self, target: &String, index: u32) -> AVM1Data;
    fn set_property(&self, target: &String, index: u32, value: &AVM1Data);
    fn clone_sprite(&self, source: &String, target: &String, depth: f32);
    fn remove_sprite(&self, target: &String);
    fn start_drag(&self, target: &String, lock_centre: f32);
    fn end_drag(&self);
    fn wait_for_frame2(&self, target: &String) -> bool;
    fn trace(&self, value: &String);
    fn get_time(&self) -> u64;
    fn random_number(&self, max: u32) -> u32;
}

#[derive(Debug, Copy, Clone)]
pub enum AVM1ErrorKind {
    StackEmpty,
    InvalidOpcode,
    InvalidOperand,
    TypeMismatch
}

#[derive(Debug, Copy, Clone)]
pub struct AVM1Error {
    pub kind: AVM1ErrorKind,
}

impl AVM1Error {
    pub fn new(kind: AVM1ErrorKind) -> AVM1Error {
        AVM1Error {
            kind: kind
        }
    }
}

pub struct AVM1SyscallsDefault;

#[allow(unused_variables)]
impl AVM1Syscalls for AVM1SyscallsDefault {
    fn goto_frame(&self, frame: u16) { }
    fn get_url(&self, url: &String, target: &String) { }
    fn next_frame(&self) { }
    fn previous_frame(&self) { }
    fn play(&self) { }
    fn stop(&self) { }
    fn toggle_quality(&self) { }
    fn stop_sounds(&self) { }
    fn wait_for_frame(&self, frame: u16) -> bool { true }
    fn goto_label(&self, label: &String) { }

    fn call(&self, target: &String) { }
    fn get_url2(&self, target: &String, url: &String, method: u8, target_sprite: bool, load_variables: bool) { }
    fn goto_frame2(&self, frame: &String, scene_bias: u16, play: bool) { }
    fn get_property(&self, target: &String, index: u32) -> AVM1Data { AVM1Data::Null }
    fn set_property(&self, target: &String, index: u32, value: &AVM1Data) { }
    fn clone_sprite(&self, source: &String, target: &String, depth: f32) { }
    fn remove_sprite(&self, target: &String) { }
    fn start_drag(&self, target: &String, lock_centre: f32) { }
    fn end_drag(&self) { }
    fn wait_for_frame2(&self, target: &String) -> bool { true }
    fn trace(&self, value: &String) { }
    fn get_time(&self) -> u64 { 0 }
    fn random_number(&self, max: u32) -> u32 { 0 }
}

impl AVM1Context {
    pub fn new(version: u8) -> Self {
        AVM1Context {
            version: version,
            registers: [AVM1Data::Null, AVM1Data::Null, AVM1Data::Null, AVM1Data::Null],
            variables: HashMap::new(),
            constant_pool: Vec::new(),
            stack: Vec::new(),
            target: String::new()
        }
    }
}
