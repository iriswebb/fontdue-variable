# FemtoFont - Fast, featured, embedded friendly TTF renderer

FemtoFont is a `no_std` rasterizer for scalable fonts (`.ttf`, `.otf`, `.ttc`), it includes
variable fonts, configurable caching, pixel and subpixlel anti-aliasing, and a basic layout engine.
It attempts to limit stack and heap usage to a point where it is usable in embedded projects while
retaining speed.

Although its layout engine doesn't shape glyphs, [`rustybuzz`](https://docs.rs/rustybuzz/) (or
[`harfrust`](https://docs.rs/harfrust/latest/harfrust/)) could be used to do so in conjunction with it.

FemtoFont can run in extremely low amounts of memory, rendering small fonts in as little as 4KB of stack space and less
than 64KB of heap space.

FemtoFont is a fork of [`fontdue`](https://docs.rs/fontdue/latest/fontdue/), which itself is a fork of [`font-rs`](https://docs.rs/font-rs/latest/font_rs/). And is based off the [`ttf_parser`](https://docs.rs/ttf-parser/latest/ttf_parser/) API

## Quickstart

Using `eg-femtofont` on `embedded-graphics`:

```rust
// load the font from raw data
let font = include_bytes!("assets/path_to_font.ttf") as &[u8];
let font = femtofont::Font::from_bytes_with_weight(font, 600.0, fontdue::FontSettings::default()).unwrap();

// Red text anti-aliased as if it were on a blue background
let style = eg_femtofont::FemtoFontTextStyle::with_aa_color(&font, Rgb888::RED, Rgb888::BLUE, 20);
let rendered_text = Text::new("FemtoFont", Point::new(100, 100), style);

// Draw
rendered_text.draw(&mut display).unwrap();
```

## Goals

1. Keeping the stack and heap intact (<4K stack, <64k heap)
2. Ease of use
3. Portability
4. Features
5. Speed

## Changes from fontdue

- Using the portable implementations of `simd` and floating point math inside `core` instead of manually optimizing them
- Removing most pre-render caching and replacing it with optional render-time caching
- Adding support for variable fonts by coupling the API closer to `ttf_parser`'s
- `Arc`-ing font faces to preserve stack space and to allow multiple `Font`'s to use the same `Face`

## Other font rendering crates

Non-shaping rasterizers:

- [`rusttype`](https://gitlab.redox-os.org/redox-os/rusttype)
- [`ab_glyph`](https://github.com/alexheretic/ab-glyph)
- [`glyph_brush`](https://github.com/alexheretic/glyph-brush/tree/master/glyph-brush)
- [`glyph_brush_layout`](https://github.com/alexheretic/glyph-brush/tree/master/layout)

## License

Licensed under either [Apache](LICENSE-APACHE), [MIT](LICENSE-MIT), or [zlib](LICENSE-ZLIB)
