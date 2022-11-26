use std::fmt;
use std::convert::TryInto;

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub enum TokenType {
    Add,
    Subtract,
    OpenLoop,
    EndLoop,
    Number,
    Output,
    Input,
    LoopCounter,

    Push,
    Pop,

    SkipNext,

    StartNumber,
    Extra,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenType::Add              => write!(f, "Add"),
            TokenType::Subtract         => write!(f, "Subtract"),
            TokenType::OpenLoop         => write!(f, "OpenLoop"),
            TokenType::EndLoop          => write!(f, "EndLoop"),
            TokenType::Number           => write!(f, "Number"),
            TokenType::Output           => write!(f, "Output"),
            TokenType::Input            => write!(f, "Input"),
            TokenType::LoopCounter      => write!(f, "LoopCounter"),

            TokenType::Push             => write!(f, "Push"),
            TokenType::Pop              => write!(f, "Pop"),

            TokenType::SkipNext         => write!(f, "SkipNext"),

            TokenType::StartNumber      => write!(f, "StartNumber"),
            TokenType::Extra            => write!(f, "Extra"),
        }
    }
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub struct Token {
    token_type  : TokenType,
    loop_id     : i64,
    loop_layer  : i64,
}


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("Type: {}, LoopID: {}, LoopLayer: {}", (*self).token_type, (*self).loop_id, (*self).loop_layer))
    }
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub struct OptiToken {
    og_token    : Token,
    repeated    : u8,
}


impl fmt::Display for OptiToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("OG Token: {}, Repeated: {}", (*self).og_token, (*self).repeated))
    }
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub enum TokenHandler {
    Normal,
    QuestionMark,
    ParseString,
    FromString,
}

impl fmt::Display for TokenHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenHandler::Normal            => write!(f, "Normal"), 
            TokenHandler::QuestionMark      => write!(f, "Question Mark"),
            TokenHandler::ParseString       => write!(f, "Waiting String"),
            TokenHandler::FromString        => write!(f, "String just ended"),
        }
    }
}

fn pop_first<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.is_empty() {
        return None;
    }
    Some(vec.remove(0))
}

pub fn compile(mut input: Vec<u8>, no_hlt_enabled: bool) -> String {
    // To token
    let mut token_list      : Vec<Token>        = Vec::new();
    let mut loop_stack      : Vec<i64>          = Vec::new();
    let mut loop_time_ref   : Vec<u16>          = Vec::new();   // Use numbers larger than 255 to signal special stuff
                                                                // Special number list:
                                                                //      0x0100: Inf loop
                                                                //      0x0101: N loop
    let mut number_ref      : Vec<u8>      = Vec::new();
    let mut port_name_ref   : Vec<String>  = Vec::new();        // Use String for ports

    let mut char_buf  : u8      = 0;
    let mut be_number : bool    = false;
    let mut loop_layer: i64     = 0;
    let mut loop_id   : i64     = 0;
    let mut prev_tok  : Token   = Token{token_type: TokenType::Extra, loop_id: 0, loop_layer: 0};

    let mut next_token_stuff: TokenHandler      = TokenHandler::Normal;
    let mut string_buffer   : Vec<u8>           = Vec::new();

    for el in input.iter_mut() {
        let curr_tok_type:TokenType = match next_token_stuff {
            TokenHandler::QuestionMark          => {
                next_token_stuff = TokenHandler::Normal;

                match *el {
                    60                  => TokenType::Pop,      // <
                    62                  => TokenType::Push,     // >
                    91                  => {
                        loop_time_ref.push(257);
                        loop_id += 1;
                        loop_layer += 1;
                        loop_stack.push(loop_id);
                        TokenType::OpenLoop
                    }   // ?[

                    _                   => continue,
                }
            },

            TokenHandler::ParseString           => {
                if *el == 37 {
                    next_token_stuff = TokenHandler::FromString;
                } else {
                    string_buffer.push(*el);
                }

                continue;
            },

            TokenHandler::FromString            => {
                next_token_stuff = TokenHandler::Normal;
                match *el {
                    60                  => {
                        port_name_ref.push(format!("%{}", std::str::from_utf8(&string_buffer).unwrap()));
                        TokenType::Input
                    },  // <

                    62                  => {
                        port_name_ref.push(format!("%{}", std::str::from_utf8(&string_buffer).unwrap()));
                        TokenType::Output
                    },  // >

                    _                   => continue,
                }
            },

            _ | TokenHandler::Normal            => {
                match *el {
                    43                  => TokenType::Add,      // +
                    45                  => TokenType::Subtract, // -
        
                    91                  => {
                        if prev_tok.token_type == TokenType::Number {
                            loop_time_ref.push(*number_ref.last().unwrap() as u16);
                        } else {
                            loop_time_ref.push(256);
                        }
                        
                        loop_id += 1;
                        loop_layer += 1;
                        loop_stack.push(loop_id);
                        
                        TokenType::OpenLoop
                    },  // [
                    93                  => {
                        loop_layer -= 1;

                        let curr_tok:Token = Token {
                            token_type      : TokenType::EndLoop,
                            loop_id         : loop_stack.pop().unwrap(),
                            loop_layer      : loop_layer
                        };
                
                        prev_tok = curr_tok.clone();
                        token_list.push(curr_tok);

                        continue;
                    },  // ]
        
                    48                  => {
                        if be_number {
                            let mut buf: Vec<u8> = Vec::new();
                            buf.push(char_buf);
                            buf.push(*el);
        
                            number_ref.push(u8::from_str_radix(std::str::from_utf8(&buf).unwrap(), 16).unwrap());
                            be_number = false;
        
                            TokenType::Number
                        } else if prev_tok.token_type == TokenType::StartNumber {
                            char_buf = *el;
                            be_number = true;
                            continue;
                        } else {
                            prev_tok = Token {
                                token_type      : TokenType::StartNumber,
                                loop_id         : loop_id,
                                loop_layer      : loop_layer
                            };
                            continue;
                        }
                    },  // 0
                    49..=57 | 65..=70 | 97..=102  => {
                        if be_number {
                            let mut buf: Vec<u8> = Vec::new();
                            buf.push(char_buf);
                            buf.push(*el);
                            be_number = false;
        
                            number_ref.push(u8::from_str_radix(std::str::from_utf8(&buf).unwrap(), 16).unwrap());
                        } else if prev_tok.token_type == TokenType::StartNumber {
                            char_buf = *el;
                            be_number = true;
                            continue;
                        }
                        TokenType::Number
                    },  // 1 - 9, A - F, a - f
        
                    60                  => {
                        if prev_tok.token_type == TokenType::Number {
                            port_name_ref.push(format!("{}", *number_ref.last().unwrap()));
                        } else {
                            port_name_ref.push("%TEXT".to_string());
                        }
                        TokenType::Input
                    },  // <
                    62                  => {
                        if prev_tok.token_type == TokenType::Number {
                            port_name_ref.push(format!("{}", *number_ref.last().unwrap()));
                        } else {
                            port_name_ref.push("%TEXT".to_string());
                        }
                        TokenType::Output
                    },  // >
        
                    63                  => {
                        next_token_stuff = TokenHandler::QuestionMark;
                        continue;
                    },  // ?
        
                    37                  => {
                        string_buffer.clear();
                        next_token_stuff = TokenHandler::ParseString;
                        continue;
                    },  // %

                    36                  => TokenType::LoopCounter,  // $
        
                    _                   => continue,
                }
            },
        };

        let curr_tok:Token = Token {
            token_type      : curr_tok_type,
            loop_id         : loop_id,
            loop_layer      : loop_layer
        };

        prev_tok = curr_tok.clone();
        token_list.push(curr_tok);
    }

    // Optimize LV.1: Unroll
    let mut token_list_2    : Vec<Token>        = Vec::new();
    let mut ref_loop_time   : Vec<u16>          = loop_time_ref.clone();
    let mut i               : usize             = 0;

    while i < token_list.len().try_into().unwrap() {
        if token_list[i].token_type == TokenType::OpenLoop && token_list[i+1].token_type == TokenType::Add && token_list[i+2].token_type == TokenType::EndLoop {
            token_list_2.push(Token{token_type: TokenType::SkipNext, loop_id: 0, loop_layer: 0});
            for _ in 0..pop_first(&mut ref_loop_time).unwrap() {
                token_list_2.push(token_list[i+1]);
            }
            i += 3;
        } else if token_list[i].token_type == TokenType::OpenLoop && token_list[i+1].token_type == TokenType::Subtract && token_list[i+2].token_type == TokenType::EndLoop {
            token_list_2.push(Token{token_type: TokenType::SkipNext, loop_id: 0, loop_layer: 0});
            for _ in 0..pop_first(&mut ref_loop_time).unwrap() {
                token_list_2.push(token_list[i+1]);
            }
            i += 3;
        } else {
            token_list_2.push(token_list[i]);
            i += 1;
        }
    }

    // Optimize LV.2: Counting
    let mut repeated            : i64               = 0;
    let mut temp_token          : Token             = Token{token_type: TokenType::Extra, loop_id: 0, loop_layer: 0};
    let mut otoken_list         : Vec<OptiToken>    = Vec::new();

    for el in token_list_2.iter_mut() {
        if (*el).token_type == TokenType::Add {
            temp_token = *el;
            repeated += 1;
            continue;
        } else if (*el).token_type == TokenType::Subtract {
            temp_token = *el;
            repeated -= 1;
            continue;
        }

        if repeated < 0 && (*el).token_type != TokenType::Add {
            otoken_list.push(OptiToken{og_token: temp_token, repeated: repeated.abs() as u8});
            repeated = 0;
        } else if repeated > 0 && (*el).token_type != TokenType::Subtract {
            otoken_list.push(OptiToken{og_token: temp_token, repeated: repeated.abs() as u8});
            repeated = 0;
        }

        otoken_list.push(OptiToken{og_token: *el, repeated: 1});
    }

    token_to_urcl(&mut otoken_list, &mut loop_time_ref, &mut number_ref, &mut port_name_ref, no_hlt_enabled)
}

pub fn token_to_urcl(token_list: &mut Vec<OptiToken>, loop_time_ref: &mut Vec<u16>, number_ref: &mut Vec<u8>, port_name_ref: &mut Vec<String>, no_hlt_enabled: bool) -> String {
    let mut urcl_tmp: Vec<String>   = Vec::new();
    
    let mut tmp_loop_ref        : Vec<u16>      = Vec::new();

    for el in token_list.iter_mut() {
        urcl_tmp.push(match (*el).og_token.token_type {
            TokenType::SkipNext     => "BRZ ~+2 R1".to_string(),

            TokenType::Add          => format!("ADD R1 R1 {}", (*el).repeated),
            TokenType::Subtract     => format!("SUB R1 R1 {}", (*el).repeated),

            TokenType::OpenLoop     => {
                let loop_time = pop_first(loop_time_ref).unwrap();
                tmp_loop_ref.push(loop_time);
                if loop_time == 256 {
                    format!(".loop{}", (*el).og_token.loop_id)
                } else if loop_time == 257 {
                    format!("BRZ .loop{}_e R1\nMOV R{} R1\n.loop{}", (*el).og_token.loop_id, (*el).og_token.loop_layer + 2, (*el).og_token.loop_id)
                } else {
                    format!("BRZ .loop{}_e R1\nMOV R{} {}\n.loop{}", (*el).og_token.loop_id, (*el).og_token.loop_layer + 2, loop_time, (*el).og_token.loop_id)
                }
            }
            TokenType::EndLoop      => {
                let loop_time = pop_first(&mut tmp_loop_ref).unwrap();
                
                if loop_time == 256 {
                    format!("JMP .loop{}", (*el).og_token.loop_id)
                } else {
                    format!("DEC R{} R{}\nBNZ .loop{} R{}\n.loop{}_e", (*el).og_token.loop_layer + 2, (*el).og_token.loop_layer + 2, (*el).og_token.loop_id, (*el).og_token.loop_layer + 2, (*el).og_token.loop_id)
                }
            }

            TokenType::Input        => format!("IN R1 {}", pop_first(port_name_ref).unwrap()),
            TokenType::Output       => format!("OUT {} R1", pop_first(port_name_ref).unwrap()),

            TokenType::LoopCounter  => format!("MOV R1 R{}", (*el).og_token.loop_layer + 2),

            TokenType::Push         => format!("PSH R1"),
            TokenType::Pop          => format!("POP R1"),

            TokenType::Number       => continue,
            _                       => format!(""),
        })
    }

    if !no_hlt_enabled {
        urcl_tmp.push(format!("HLT"));
    }

    // println!("{}", urcl_tmp.join("\n"))
    urcl_tmp.join("\n")
}