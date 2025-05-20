use proc_macro2::{Span, TokenStream};
use syn::Result;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

#[derive(Debug)]
struct GlyphDef {
    pub bit_vec: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub advance: usize,
    pub baseline: isize,
}

fn parse_bitmap_glyph(glyph_def: &str) -> GlyphDef {
    let mut first_line_found = false;
    let mut height = 0;
    let mut width = 0;
    let mut start_line = 0;
    for (i, line) in glyph_def.lines().enumerate() {
        if !first_line_found {
            if line.find("*").is_some() {
                first_line_found = true;
                start_line = i;
            }
        }
        if first_line_found {
            if let Some(line_width) = line.rfind("*").map(|x| x + 1) {
                width = width.max(line_width);
                height = i;
            }
        }
    }
    first_line_found = false;
    let mut parse_line = 0;
    let mut bit_vec = Vec::new();
    let mut baseline = 0;
    let mut advance = 0;
    for (i, line) in glyph_def.lines().enumerate() {
        if !first_line_found {
            if line.find("*").is_some() {
                first_line_found = true;
            }
        }
        if line.contains("<") {
            baseline = i as isize - start_line as isize;
        }
        if let Some(index) = line.find(",") {
            advance = index;
        }
        if first_line_found {
            if parse_line < height {
                let mut chars = line.chars();
                for _ in 0..width {
                    if let Some(c) = chars.next() {
                        match c {
                            '*' => bit_vec.push(true),
                            _ => bit_vec.push(false),
                        }
                    } else {
                        bit_vec.push(false);
                    }
                }
            }
            parse_line += 1;
        }
    }
    let n_bytes = (bit_vec.len() / 8) + if (bit_vec.len() % 8) != 0 { 1 } else { 0 };
    let mut byte_vec = Vec::new();
    for i in 0..n_bytes {
        let base_bit = i * 8;
        let byte = 
            get_bit(&bit_vec, base_bit, 0) |
            get_bit(&bit_vec, base_bit, 1) |
            get_bit(&bit_vec, base_bit, 2) |
            get_bit(&bit_vec, base_bit, 3) |
            get_bit(&bit_vec, base_bit, 4) |
            get_bit(&bit_vec, base_bit, 5) |
            get_bit(&bit_vec, base_bit, 6) |
            get_bit(&bit_vec, base_bit, 7);
        byte_vec.push(byte);
    }
    GlyphDef {
        bit_vec: byte_vec,
        width,
        height,
        baseline,
        advance
    }
}

fn get_bit(bit_vec: &Vec<bool>, base_bit: usize, bit_n: usize) -> u8 {
    let bit = base_bit + bit_n;
    if bit < bit_vec.len() {
        if bit_vec[bit] {
            1 << bit_n
        } else {
            0
        }
    } else {
        0
    }
}

pub fn bitmap_glyph_impl(tokens: TokenStream) -> Result<TokenStream> {
    let mut t = tokens.into_iter();
    let glyph_name = match t.next() {
        Some(proc_macro2::TokenTree::Ident(identifier)) => identifier,
        Some(other) => return Err(syn::Error::new(other.span(), "Expected a name for the glyph")),
        None => return Err(syn::Error::new(Span::call_site(), "Expected a name for the glyph")),
    };
    match t.next() {
        Some(proc_macro2::TokenTree::Punct(punct)) => {
            match punct.as_char() {
                ',' => {},
                _ => return Err(syn::Error::new(punct.span(), "Expected a comma after glyph name")),
            }
        },
        Some(other) => return Err(syn::Error::new(other.span(), "Expected a comma after glyph name")),
        None => return Err(syn::Error::new(Span::call_site(), "Expected a comma after glyph name")),
    };
    let glyph_def = match t.next() {
        Some(proc_macro2::TokenTree::Literal(literal)) => {
            let literal_string = syn::parse::<syn::LitStr>(literal.to_token_stream().into())?;
            parse_bitmap_glyph(&literal_string.value())
        },
        Some(other) => return Err(syn::Error::new(other.span(), "Expected a string depicting the glyph after comma")),
        None => return Err(syn::Error::new(Span::call_site(), "Expected a string depicting the glyph after comma")),
    };
    let glyph_bitmap_name = format_ident!("{}_BITMAP", glyph_name);
    let mut glyph_def_byte_tokens: TokenStream = TokenStream::new();
    for byte in glyph_def.bit_vec.iter() {
        glyph_def_byte_tokens.append(
            proc_macro2::TokenTree::Literal(
                proc_macro2::Literal::u8_unsuffixed(*byte)
            )
        );
        glyph_def_byte_tokens.append(
            proc_macro2::TokenTree::Punct(
                proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone)
            )
        );
    }
    let glyph_width = proc_macro2::TokenTree::Literal(
        proc_macro2::Literal::usize_unsuffixed(glyph_def.width)
    );
    let glyph_height = proc_macro2::TokenTree::Literal(
        proc_macro2::Literal::usize_unsuffixed(glyph_def.height)
    );
    let glyph_baseline = proc_macro2::TokenTree::Literal(
        proc_macro2::Literal::isize_unsuffixed(glyph_def.baseline)
    );
    let glyph_advance = proc_macro2::TokenTree::Literal(
        proc_macro2::Literal::usize_unsuffixed(glyph_def.advance)
    );
    Ok(
        quote! {
            const #glyph_bitmap_name: &'static [u8] = &[
                #glyph_def_byte_tokens
            ];

            const #glyph_name: Glyph = Glyph {
                width: #glyph_width,
                height: #glyph_height,
                bitmap: #glyph_bitmap_name,
                baseline: #glyph_baseline,
                advance: #glyph_advance
            };
        }
    )
}