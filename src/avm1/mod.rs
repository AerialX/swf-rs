mod context;
mod stack;
mod pc;
mod avm1;

pub use self::context::{ AVM1Data, AVM1Context, AVM1Syscalls, AVM1SyscallsDefault, AVM1Error, AVM1ErrorKind };
pub use self::pc::AVM1ProgramCounter;
pub use self::avm1::avm1_execute;

// AVM1 (AS1/AS2)
#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq, Eq)]
pub enum AVM1ActionKind {
    // SWF 3
    End             = 0x00,
    GotoFrame       = 0x01 | 0x80,
    GetURL          = 0x03 | 0x80,
    NextFrame       = 0x04,
    PreviousFrame   = 0x05,
    Play            = 0x06,
    Stop            = 0x07,
    ToggleQuality   = 0x08,
    StopSounds      = 0x09,
    WaitForFrame    = 0x0a | 0x80,
    SetTarget       = 0x0b | 0x80,
    GoToLabel       = 0x0c | 0x80,

    // SWF 4
    Push            = 0x16 | 0x80,
    Pop             = 0x17,

    Add             = 0x0a,
    Subtract        = 0x0b,
    Multiply        = 0x0c,
    Divide          = 0x0d,

    Equals          = 0x0e,
    Less            = 0x0f,
    And             = 0x10,
    Or              = 0x11,
    Not             = 0x12,

    StringEquals    = 0x13,
    StringLength    = 0x14,
    StringAdd       = 0x21,
    StringExtract   = 0x15,
    StringLess      = 0x29,
    MBStringLength  = 0x31,
    MBStringExtract = 0x35,

    ToInteger       = 0x18,
    CharToAscii     = 0x32,
    AsciiToChar     = 0x33,
    MBCharToAscii   = 0x36,
    MBAsciiToChar   = 0x37,

    Jump            = 0x19 | 0x80,
    If              = 0x1d | 0x80,
    Call            = 0x1e | 0x80,

    GetVariable     = 0x1c,
    SetVariable     = 0x1d,

    GetURL2         = 0x1a | 0x80,
    GotoFrame2      = 0x1f | 0x80,
    SetTarget2      = 0x20,
    GetProperty     = 0x22,
    SetProperty     = 0x23,
    CloneSprite     = 0x24,
    RemoveSprite    = 0x25,
    StartDrag       = 0x27,
    EndDrag         = 0x28,
    WaitForFrame2   = 0x0d | 0x80,

    Trace           = 0x26,
    GetTime         = 0x34,
    RandomNumber    = 0x30,

    // SWF 5

    CallFunction    = 0x3d,
    CallMethod      = 0x52,
    ConstantPool    = 0x08 | 0x80,
    DefineFunction  = 0x1b | 0x80,
    DefineLocal     = 0x3c,
    DefineLocal2    = 0x41,
    Delete          = 0x3a,
    Delete2         = 0x3b,
    Enumerate       = 0x46,
    Equals2         = 0x49,
    GetMember       = 0x4e,
    InitArray       = 0x42,
    InitObject      = 0x43,
    NewMethod       = 0x53,
    NewObject       = 0x40,
    SetMember       = 0x4f,
    TargetPath      = 0x45,
    With            = 0x14 | 0x80,
    ToNumber        = 0x4a,
    ToString        = 0x4b,
    TypeOf          = 0x44,
    Add2            = 0x47,
    Less2           = 0x48,
    Modulo          = 0x3f,
    BitAnd          = 0x60,
    BitLShift       = 0x63,
    BitOr           = 0x61,
    BitRShift       = 0x64,
    BitURShift      = 0x65,
    BitXor          = 0x62,
    Decrement       = 0x51,
    Increment       = 0x50,
    PushDuplicate   = 0x4c,
    Return          = 0x3e,
    StackSwap       = 0x4d,
    StoreRegister   = 0x07 | 0x80,

    // SWF 6
    
    InstanceOf      = 0x54,
    Enumerate2      = 0x55,
    StrictEquals    = 0x66,
    Greater         = 0x67,
    StringGreater   = 0x68,

    // SWF 7

    DefineFunction2 = 0x0e | 0x80,
    Extends         = 0x69,
    CastOp          = 0x2b,
    ImplementsOp    = 0x2c,
    Try             = 0x0f | 0x80,
    Throw           = 0x2a
}

impl AVM1ActionKind {
    pub fn has_data(&self) -> bool {
        avm1_action_has_data(*self as u8)
    }
}

pub fn avm1_action_has_data(v: u8) -> bool {
    v & 0x80 != 0
}
