use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::iter::Peekable;

pub enum Token {
    Null,
    Printable(char),
    EndSection,
    FileHeader,
    Underline,
    Indent,
    NewLine,
    AlignCenter,
    AlignLeft,
    NewPage,
    AGrave,
    EGrave,
    EAcute,
    IGrave,
    OGrave,
    UGrave
}


fn parse_header(_code: u8) {
    match _code {
        0x1B => print!(" "),
        0x20 ..= 0x7E => print!("{}", _code as char),
        _ => print!(" "),
    }
}

fn match_expected<'a,T: Iterator<Item = &'a u8>>(expected: &Vec<u8>, it: &mut Peekable<T>) -> bool {
    let mut valid = true;
    for ec in expected {
        if let Some(&c) = it.peek() {
            if c != ec {
                valid = false;
            }
            it.next();
        }
    }

    valid
}

fn eat_identical<'a,T: Iterator<Item = &'a u8>>(first: &u8, it: &mut Peekable<T>) {
    loop {
        if let Some(&c) = it.peek() {
            if *c != *first {
                break;
            }
        } else {
            break;
        }
        it.next();
    }
}

fn lex_zero_tag<'a,T: Iterator<Item = &'a u8>>(it: &mut Peekable<T>) -> Token {
    let mut token = Token::Null;

    while let Some(&c) = it.peek() {
        it.next();
        match c {
            0x23 => {
                token = Token::Indent;
            }
            0x28 => {
                token = Token::AlignLeft;
            }
            0x7F => {
                return token;
            }
            _ => {
                return Token::Null;
            }
        }
    }

    return Token::Null;
}

fn lex_0400_tag<'a,T: Iterator<Item = &'a u8>>(it: &mut Peekable<T>) -> Token {
    let mut token = Token::Null;

    if let Some(&c) = it.peek() {
        if *c != 0x00 {
            return Token::Null;
        }
    } else {
        return Token::Null;
    }

    while let Some(&c) = it.peek() {
        it.next();
        match c {
            0x23 => {
                token = Token::Indent;
            }
            0x28 => {
                token = Token::AlignLeft;
            }
            0x7F => {
                return token;
            }
            _ => {
                //TODO: find the real ones
                token = Token::AlignCenter;
                //return Token::Null;
            }
        }
    }

    return Token::Null;
}

fn lex(input: &Vec<u8>) -> Vec<Token> {
    let mut result = Vec::new();

    let mut it = input.iter().peekable();
    
    while let Some(&code) = it.peek() {
        match code {
            0x00 => { // 0x04 0x00 character 0x7F some formatting directives
                it.next();
                let tok = lex_zero_tag(&mut it);
                //FIXME: prettier way?
                if let Token::Null = tok {
                } else {
                    result.push(tok);
                }
            }
            0x04 => { // 0x04 0x00 character 0x7F some formatting directives
                it.next();
                let tok = lex_0400_tag(&mut it);
                //FIXME: prettier way?
                if let Token::Null = tok {
                } else {
                    result.push(tok);
                    let next = it.peek();
                    if next != Some(&&u8::from(0x0D)) {
                        result.push(Token::NewLine);
                    }
                }
            }

            0x1B => { // Starts the section with Oliword version and  date
                it.next();
                
                let expected = vec![u8::from(0x1B)];

                if match_expected(&expected, &mut it) {
                    result.push(Token::FileHeader);
                }
            }

            0xFF => { // ends a section
                it.next();
                eat_identical(code, &mut it);
                result.push(Token::EndSection);
            }

            0x1E => { // 0x1E 0x02 0x1F makes next character underlined
                it.next();

                let expected = vec![u8::from(0x02), u8::from(0x1F)];

                if match_expected(&expected, &mut it) {
                    result.push(Token::Underline);
                }
            }

            0x20 ..= 0x7E => { // Normal printable characters
                it.next();
                // Italian-specific: Check for accents
                let accent = vec!['`' as u8];
                let mut possibleAccent = false;
                let mut foundAccent = false;

                match code {
                    0x68 => { //h: if it's he` will be an eacute
                        possibleAccent = true;
                        result.push(Token::Printable(*code as char));
                        if let Some(&c) = it.peek() {
                            it.next();
                            if *c == 0x65 { // e
                                if let Some(&c) = it.peek() {
                                    it.next();
                                    if *c == 0x60 { // `
                                        foundAccent = true;
                                        result.push(Token::EAcute);
                                    } else {
                                        result.push(Token::Printable('e'));
                                        if *c > 0x20 && *c <= 0x7E {
                                            result.push(Token::Printable(*c as char));
                                        } else if *c >= 0x80 && *c <= 0x8D {
                                            result.push(Token::Printable(' '));
                                        }
                                    }
                                }
                            } else {
                                result.push(Token::Printable(*c as char));
                            }
                        }
                    }
                    0x61 => { //a
                        if let Some(&c) = it.peek() {
                            if *c == 0x60 { // `
                                foundAccent = true;
                                it.next();
                                result.push(Token::AGrave);
                            }
                        }
                    }
                    0x65 => { //e
                        if let Some(&c) = it.peek() {
                            if *c == 0x60 { // `
                                foundAccent = true;
                                it.next();
                                result.push(Token::EGrave);
                            }
                        }
                    }
                    0x69 => { //i
                        if let Some(&c) = it.peek() {
                            if *c == 0x60 { // `
                                foundAccent = true;
                                it.next();
                                result.push(Token::IGrave);
                            }
                        }
                    }
                    0x6F => { //o
                        if let Some(&c) = it.peek() {
                            if *c == 0x60 { // `
                                foundAccent = true;
                                it.next();
                                result.push(Token::OGrave);
                            }
                        }
                    }
                    0x75 => { //u
                        if let Some(&c) = it.peek() {
                            if *c == 0x60 { // `
                                foundAccent = true;
                                it.next();
                                result.push(Token::UGrave);
                            }
                        }
                    }
                    _ => {}
                }

                if foundAccent {
                    if let Some(&c) = it.peek() {
                        if *c > 0x20 && *c <= 0x7E { // Printable Non space
                            // Make sure there is a space after accent
                            result.push(Token::Printable(' '));
                        }
                    }
                } else if !possibleAccent { // put the found character as token
                    result.push(Token::Printable(*code as char));
                }
            }

            0x09 => { // tab
                if let Token::NewLine = result.last().unwrap() {
                    result.push(Token::Indent);
                } else {
                    result.push(Token::Printable(*code as char));
                }
                it.next();
            }
            
            0x0A => { //New Line
                result.push(Token::NewLine);
                it.next();
            }
            
            0x0B => { // Vertical Tab
                result.push(Token::Printable(*code as char));
                it.next();
            }

            0x0C => { //New Page
                result.push(Token::NewPage);
                it.next();
            }

            0x0D => { // Carriage Return
                result.push(Token::NewLine);
                it.next();
            }

            0x80 ..= 0x8D => { // They seem to be randomly used as spaces
                result.push(Token::Printable(' '));
                it.next();
            }
            _ => {  // Ignore everything else
                it.next();
            }
        }
    }

    result
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} inputfile outputfile", args[0]);
        std::process::exit(2)
    }
    let in_file_name = &args[1];
    let out_file_name = &args[2];

    println!("Input file: {}", in_file_name);
    println!("Output file: {}", out_file_name);
    
    
    let mut f = File::open(in_file_name).expect("Input file open failed");
    let mut out_f = File::create(out_file_name).expect("Output file open failed");
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).expect(&format!("Error in reading file {}", in_file_name));

    let tokens = lex(&buffer);

    let mut it = tokens.iter().peekable();
    let mut curr_section = 0;

    let mut paragraph = String::new();
    let mut paragraph_align = "j";
    let mut bold = false;

    let header = r#"{\rtf1\pc {\info{\revtim\mo03\dy02\yr2021}{\creatim\mo03\dy02\yr2021}
{\nofchars31}}\deff0{\fonttbl{\f0\fmodern Times;}{\f1\fmodern Courier;}
{\f2\fmodern elite;}{\f3\fmodern prestige;}{\f4\fmodern lettergothic;}
{\f5\fmodern gothicPS;}{\f6\fmodern cubicPS;}
{\f7\fmodern lineprinter;}{\f8\fswiss Helvetica;}
{\f9\fmodern avantegarde;}{\f10\fmodern spartan;}{\f11\fmodern metro;}
{\f12\fmodern presentation;}{\f13\fmodern APL;}{\f14\fmodern OCRA;}
{\f15\fmodern OCRB;}{\f16\froman boldPS;}{\f17\froman emperorPS;}
{\f18\froman madaleine;}{\f19\froman zapf humanist;}
{\f20\froman classic;}{\f21\froman roman f;}{\f22\froman roman g;}
{\f23\froman roman h;}{\f24\froman timesroman;}{\f25\froman century;}
{\f26\froman palatino;}{\f27\froman souvenir;}{\f28\froman garamond;}
{\f29\froman caledonia;}{\f30\froman bodini;}{\f31\froman university;}
{\f32\fscript script;}{\f33\fscript scriptPS;}{\f34\fscript script c;}
{\f35\fscript script d;}{\f36\fscript commercial script;}
{\f37\fscript park avenue;}{\f38\fscript coronet;}
{\f39\fscript script h;}{\f40\fscript greek;}{\f41\froman kana;}
{\f42\froman hebrew;}{\f43\froman roman s;}{\f44\froman russian;}
{\f45\froman roman u;}{\f46\froman roman v;}{\f47\froman roman w;}
{\f48\fdecor narrator;}{\f49\fdecor emphasis;}
{\f50\fdecor zapf chancery;}{\f51\fdecor decor d;}
{\f52\fdecor old english;}{\f53\fdecor decor f;}{\f54\fdecor decor g;}
{\f55\fdecor cooper black;}{\f56\ftech Symbol;}{\f57\ftech linedraw;}
{\f58\ftech math7;}{\f59\ftech math8;}{\f60\ftech bar3of9;}
{\f61\ftech EAN;}{\f62\ftech pcline;}{\f63\ftech tech h;}}{\colortbl
\red0\green0\blue0;\red255\green0\blue0;
\red0\green255\blue0;\red0\green0\blue255;
\red0\green255\blue255;\red255\green0\blue255;
\red255\green255\blue0;\red255\green255\blue255;}
\paperw11907\paperh16840\ftnbj\ftnrestart\widowctrl \sectd 
\linex576\endnhere "#;
    writeln!(out_f, "{}", header).unwrap();

    while let Some(&token) = it.peek() {
        if let Token::EndSection = token {
            it.next();
            curr_section = curr_section + 1;
            if !paragraph.is_empty() {
                writeln!(out_f, "{{\\pard \\q{} {} \\par}}", paragraph_align, paragraph).unwrap();
                paragraph = String::new();
                paragraph_align = "j";
            }
            continue;
        } else if curr_section != 0 && curr_section != 2 {
            it.next();
            continue;
        }

        match token {
            Token::Printable(c) => {
                it.next();
                paragraph.push(*c);
                if let Some(&next_t) = it.peek() {
                    if let Token::Underline = next_t {
                    } else if bold {
                        paragraph.push_str("}}");
                        bold = false;
                    }
                }
            }
            Token::AGrave => {
                it.next();
                paragraph.push_str("\\u224  ");
            }
            Token::EGrave => {
                it.next();
                paragraph.push_str("\\u232  ");
            }
            Token::EAcute => {
                it.next();
                paragraph.push_str("\\u233  ");
            }
            Token::IGrave => {
                it.next();
                paragraph.push_str("\\u236  ");
            }
            Token::OGrave => {
                it.next();
                paragraph.push_str("\\u242  ");
            }
            Token::UGrave => {
                it.next();
                paragraph.push_str("\\u249  ");
            }
            Token::NewLine => {
                it.next();
                if !paragraph.is_empty() {
                    writeln!(out_f, "{{\\pard \\q{} {} \\par}}", paragraph_align, paragraph).unwrap();
                    paragraph = String::new();
                } else {
                    writeln!(out_f, "{{\\pard \\par}}").unwrap();
                }
                paragraph_align = "j";
            }
            Token::NewPage => {
                it.next();
                if !paragraph.is_empty() {
                    writeln!(out_f, "{{\\pard \\pagebb \\q{} {} \\par}}", paragraph_align, paragraph).unwrap();
                    paragraph = String::new();
                } else {
                    writeln!(out_f, "{{\\pard \\pagebb \\par}}").unwrap();
                }
                paragraph_align = "j";
            }
            Token::AlignCenter => {
                it.next();
                paragraph_align = "c";
            }
            Token::AlignLeft => {
                it.next();
                paragraph_align = "l";
            }
            Token::Indent => {
                it.next();
                paragraph.push_str("  \t");
            }
            Token::Underline => {
                it.next();
                if !bold {
                    paragraph.push_str("{{\\ul ");
                    bold = true;
                }
            }
            _ => {
                it.next();
            }
        }
    }
    writeln!(out_f, "}}").unwrap();

    out_f.sync_all()?;

    Ok(())
}
