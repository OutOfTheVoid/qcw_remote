use crate::gfx::bitmap_font::{BitmapFont, Glyph, GlyphRange};
use proc_bitmap_font::bitmap_glyph;

bitmap_glyph!(A_UPPER, r#"
 * 
* *
***
* *
* *<,
"#);

bitmap_glyph!(B_UPPER, r#"
** 
* *
**
* *
** <,
"#);

bitmap_glyph!(C_UPPER, r#"
 **
*
*
*
 **<,
"#);

bitmap_glyph!(D_UPPER, r#"
** 
* *
* *
* *
** <,
"#);

bitmap_glyph!(E_UPPER, r#"
***
*
**
*
***<,
"#);

bitmap_glyph!(F_UPPER, r#"
***
*
**
*
*  <,
"#);

bitmap_glyph!(G_UPPER, r#"
 **
*  
* **
*  *
 ** <,
"#);

bitmap_glyph!(H_UPPER, r#"
* *
* *
***
* *
* *<,
"#);

bitmap_glyph!(I_UPPER, r#"
***
 * 
 * 
 * 
***<,
"#);

bitmap_glyph!(J_UPPER, r#"
***
  * 
  * 
* * 
 * <,
"#);

bitmap_glyph!(K_UPPER, r#"
* *
** 
*  
** 
* *<,
"#);

bitmap_glyph!(L_UPPER, r#"
*
*
*
*
***<,
"#);

bitmap_glyph!(M_UPPER, r#"
*   *
** **
* * *
*   *
*   *<,
"#);

bitmap_glyph!(N_UPPER, r#"
*   *
**  *
* * *
*  **
*   *<,
"#);

bitmap_glyph!(O_UPPER, r#"
 **
*  *
*  *
*  *
 ** <,
"#);

bitmap_glyph!(P_UPPER, r#"
**
* *
* *
**
*  <,
"#);

bitmap_glyph!(Q_UPPER, r#"
 **
*  *
* **
*  *
 ** *<,
"#);

bitmap_glyph!(R_UPPER, r#"
**
* *
** 
** 
* *<,
"#);

bitmap_glyph!(S_UPPER, r#"
 **
*  
 * 
  * 
** <,
"#);

bitmap_glyph!(T_UPPER, r#"
***
 * 
 * 
 * 
 * <,
"#);

bitmap_glyph!(U_UPPER, r#"
* *
* *
* *
* *
***<,
"#);

bitmap_glyph!(V_UPPER, r#"
* *
* *
* * 
* *
 * <,
"#);

bitmap_glyph!(W_UPPER, r#"
*   *
*   *
* * *
* * *
 * * <,
"#);

bitmap_glyph!(X_UPPER, r#"
* *
* *
 *  
* *
* *<,
"#);

bitmap_glyph!(Y_UPPER, r#"
* *
* * 
 * 
 *  
 * <,
"#);

bitmap_glyph!(Z_UPPER, r#"
***
  *
 *  
*  
***<,
"#);

const UPPER_CASE: GlyphRange = GlyphRange {
    start_char: 'A',
    glyphs: &[
        &A_UPPER,
        &B_UPPER,
        &C_UPPER,
        &D_UPPER,
        &E_UPPER,
        &F_UPPER,
        &G_UPPER,
        &H_UPPER,
        &I_UPPER,
        &J_UPPER,
        &K_UPPER,
        &L_UPPER,
        &M_UPPER,
        &N_UPPER,
        &O_UPPER,
        &P_UPPER,
        &Q_UPPER,
        &R_UPPER,
        &S_UPPER,
        &T_UPPER,
        &U_UPPER,
        &V_UPPER,
        &W_UPPER,
        &X_UPPER,
        &Y_UPPER,
        &Z_UPPER,
    ]
};

bitmap_glyph!(NUM_0, r#"
 ** 
* **
*  *
** *
 ** <,
"#);

bitmap_glyph!(NUM_1, r#"
 * 
**
 *
 *
***<,
"#);

bitmap_glyph!(NUM_2, r#"
 **
*  *
  *
 *
****<,
"#);

bitmap_glyph!(NUM_3, r#"
 **
*  *
  *
*  *
 ** <,
"#);

bitmap_glyph!(NUM_4, r#"
* *
* *
****
  *
  * <,
"#);

bitmap_glyph!(NUM_5, r#"
****
*   
***
   *
*** <,
"#);

bitmap_glyph!(NUM_6, r#"
  *
 *  
***
*  *
 ** <,
"#);

bitmap_glyph!(NUM_7, r#"
****
   *
  *
 *
 *  <,
"#);

bitmap_glyph!(NUM_8, r#"
 **
*  *
 **
*  *
 ** <,
"#);

bitmap_glyph!(NUM_9, r#"
 **
*  *
 ***
  *
 *  <,
"#);

const NUMBERS: GlyphRange = GlyphRange {
    start_char: '0',
    glyphs: &[
        &NUM_0,
        &NUM_1,
        &NUM_2,
        &NUM_3,
        &NUM_4,
        &NUM_5,
        &NUM_6,
        &NUM_7,
        &NUM_8,
        &NUM_9,
    ]
};

bitmap_glyph!(SPACE, r#"
<  ,
"#);

bitmap_glyph!(BANG, r#"
*
*
*

*<,
"#);

bitmap_glyph!(DQUOTE, r#"
* *
* *


   <,
"#);

bitmap_glyph!(POUND, r#"
 * *
*****
 * *
*****
 * * <,
"#);

bitmap_glyph!(DOLLAR, r#"
 ***
* *
 ***
  * *
 *** <,
"#);

bitmap_glyph!(PERCENT, r#"
**  *
** *
  *
 * **
*  **<,
"#);

bitmap_glyph!(AMPERSAND, r#"
 *
* *
 ** *
*  *
 ** *<,
"#);

bitmap_glyph!(SQUOTE, r#"
*
*


 <,
"#);

bitmap_glyph!(LPAREN, r#"
 *
*
*
*
 *<,
"#);

bitmap_glyph!(RPAREN, r#"
*
 *
 *
 *
* <,
"#);

bitmap_glyph!(STAR, r#"
* *
 *
* *
  
   <,
"#);

bitmap_glyph!(PLUS, r#"
 *
***
 *
   <,
"#);

bitmap_glyph!(COMMA, r#"
 * <,
* 
"#);

bitmap_glyph!(MINUS, r#"
***

   <,
"#);

bitmap_glyph!(DOT, r#"
*<,
"#);

bitmap_glyph!(SLASH, r#"
  *
  *
 *
*
*  <,
"#);

const PUNCT_1: GlyphRange = GlyphRange {
    start_char: ' ',
    glyphs: &[
        &SPACE,
        &BANG,
        &DQUOTE,
        &POUND,
        &DOLLAR,
        &PERCENT,
        &AMPERSAND,
        &SQUOTE,
        &LPAREN,
        &RPAREN,
        &STAR,
        &PLUS,
        &COMMA,
        &MINUS,
        &DOT,
        &SLASH
    ]
};

bitmap_glyph!(COLON, r#"
*

*
 <,
"#);

bitmap_glyph!(SEMICOLON, r#"
 *

 *
* <,
"#);

bitmap_glyph!(LESS_THAN, r#"
  *
 *
*
 *
  *<,
"#);

bitmap_glyph!(MORE_THAN, r#"
*
 *
  *
 *
*  <,
"#);

bitmap_glyph!(QMARK, r#"
 **
*  *
  *

  * <,
"#);

bitmap_glyph!(AT, r#"
 ***
*  **
* * *
*  **
 *   <,
"#);

const PUNCT_2: GlyphRange = GlyphRange {
    start_char: ':',
    glyphs: &[
        &COLON,
        &SEMICOLON,
        &LESS_THAN,
        &MORE_THAN,
        &QMARK,
        &AT,
    ]
};

bitmap_glyph!(LBRACKET, r#"
***
*
*
*
***<,
"#);

bitmap_glyph!(BACKSLASH, r#"
*
*
 *
  *
  *<,
"#);

bitmap_glyph!(RBRACKET, r#"
***
  *
  *
  *
***<,
"#);

bitmap_glyph!(CARET, r#"
 * 
* *
   
   
   <,
"#);

bitmap_glyph!(UNDERSCORE, r#"
   <,
***
"#);

bitmap_glyph!(ACCENT_GRAVE, r#"
*
 *


  <,
"#);

const PUNCT_3: GlyphRange = GlyphRange {
    start_char: '[',
    glyphs: &[
        &LBRACKET,
        &BACKSLASH,
        &RBRACKET,
        &CARET,
        &UNDERSCORE,
        &ACCENT_GRAVE,
    ]
};

bitmap_glyph!(LOWER_A, r#"
 **
* *
* *
 * *<,
"#);

bitmap_glyph!(LOWER_B, r#"
*
**
* *
* *
** <,
"#);

bitmap_glyph!(LOWER_C, r#"
 ** 
*
*  
 **<,
"#);

bitmap_glyph!(LOWER_D, r#"
  *
 **
* *
* *
 **<,
"#);

bitmap_glyph!(LOWER_E, r#"
 *
* * 
**
 **<,
"#);

bitmap_glyph!(LOWER_F, r#"
  *
 *  
***
 *
 * <,
"#);

bitmap_glyph!(LOWER_G, r#"
 **
* *
 **
  *<,
 *
"#);

bitmap_glyph!(LOWER_H, r#"
*
**
* *
* *
* *<,
"#);

bitmap_glyph!(LOWER_I, r#"
*

*
*
*<,
"#);

bitmap_glyph!(LOWER_J, r#"
 *

 *
 *
 *<,
*
"#);

bitmap_glyph!(LOWER_K, r#"
*
* *
**
* *
* *<,
"#);

bitmap_glyph!(LOWER_L, r#"
*
*
*
*
 *<,
"#);

bitmap_glyph!(LOWER_M, r#"
** *
* * *
* * *
* * *<,
"#);

bitmap_glyph!(LOWER_N, r#"
**
* *
* *
* *<,
"#);

bitmap_glyph!(LOWER_O, r#"
 *
* *
* *
 * <,
"#);

bitmap_glyph!(LOWER_P, r#"
 *
* *
* *
** <,
*
"#);

bitmap_glyph!(LOWER_Q, r#"
 **
* *
* *
 **<,
   *
"#);

bitmap_glyph!(LOWER_R, r#"
 **
*
*
*  <,
"#);

bitmap_glyph!(LOWER_S, r#"
 **
*
  *
** <,
"#);

bitmap_glyph!(LOWER_T, r#"
 *
 *
***
 * 
 * <,
"#);

bitmap_glyph!(LOWER_U, r#"
* *
* *
* *
 **<,
"#);

bitmap_glyph!(LOWER_V, r#"
* *
* *
* *
 * <,
"#);

bitmap_glyph!(LOWER_W, r#"
* * *
* * *
* * *
 * * <,
"#);

bitmap_glyph!(LOWER_X, r#"
* *
 *
 *
* *<,
"#);

bitmap_glyph!(LOWER_Y, r#"
* *
* *
* *
 **<,
  *
"#);

bitmap_glyph!(LOWER_Z, r#"
***
  *
*
***<,
"#);

const LOWERCASE: GlyphRange = GlyphRange {
    start_char: 'a',
    glyphs: &[
        &LOWER_A,
        &LOWER_B,
        &LOWER_C,
        &LOWER_D,
        &LOWER_E,
        &LOWER_F,
        &LOWER_G,
        &LOWER_H,
        &LOWER_I,
        &LOWER_J,
        &LOWER_K,
        &LOWER_L,
        &LOWER_M,
        &LOWER_N,
        &LOWER_O,
        &LOWER_P,
        &LOWER_Q,
        &LOWER_R,
        &LOWER_S,
        &LOWER_T,
        &LOWER_U,
        &LOWER_V,
        &LOWER_W,
        &LOWER_X,
        &LOWER_Y,
        &LOWER_Z,
    ]
};

bitmap_glyph!(LBRACE, r#"
 **
 *
**
 *
 **<,
"#);

bitmap_glyph!(BAR, r#"
*
*
*
*
*<,
"#);

bitmap_glyph!(RBRACE, r#"
**
 *
 **
 *
** <,
"#);

bitmap_glyph!(TILDE, r#"
 *
* * *
   *
    <,
"#);

const PUNCT_4: GlyphRange = GlyphRange {
    start_char: '{',
    glyphs: &[
        &LBRACE,
        &BAR,
        &RBRACE,
        &TILDE,
    ]
};

pub const FONT: BitmapFont = BitmapFont {
    line_height: 8,
    ranges: &[
        &PUNCT_1,
        &NUMBERS,
        &PUNCT_2,
        &UPPER_CASE,
        &PUNCT_3,
        &LOWERCASE,
        &PUNCT_4,
    ]
};
