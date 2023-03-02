use std::fmt;

#[derive(Debug)]
enum Channel {
    RGB = 3,
    RGBA = 4,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct RGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl RGBA {
    fn new() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0xFF,
        }
    }

    fn hash(&self) -> u8 {
        let index = self.r as u32 * 3 + self.g as u32 * 5 + self.b as u32 * 7 + self.a as u32 * 11;
        return (index % 64) as u8;
    }
}

impl fmt::Display for RGBA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "r: {}, g: {}, b: {}, a: {}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl From<u32> for RGBA {
    fn from(value: u32) -> Self {
        const BYTE_SIZE: u8 = 8;
        let r = ((value >> 0 * BYTE_SIZE) & 0xFF) as u8;
        let g = ((value >> 1 * BYTE_SIZE) & 0xFF) as u8;
        let b = ((value >> 2 * BYTE_SIZE) & 0xFF) as u8;
        let a = ((value >> 3 * BYTE_SIZE) & 0xFF) as u8;
        Self { r, g, b, a }
    }
}

impl From<&u32> for RGBA {
    fn from(value: &u32) -> Self {
        const BYTE_SIZE: u8 = 8;
        let r = ((value >> 0 * BYTE_SIZE) & 0xFF) as u8;
        let g = ((value >> 1 * BYTE_SIZE) & 0xFF) as u8;
        let b = ((value >> 2 * BYTE_SIZE) & 0xFF) as u8;
        let a = ((value >> 3 * BYTE_SIZE) & 0xFF) as u8;
        Self { r, g, b, a }
    }
}
#[derive(Debug)]
enum Colorspace {
    SRGB = 0,
    Linear = 1,
}

#[derive(Debug)]
struct QoiHeader {
    magic: [char; 4],
    width: u32,
    height: u32,
    channels: Channel,
    colorspace: Colorspace,
}

#[derive(Debug, Clone, Copy)]
enum Tag {
    B11,
    B00,
    B01,
    B10,
    B11111110,
    B11111111,
}

#[derive(Debug)]
struct QoiOpRun {
    tag: Tag, // 2-bit tag b11
    run: u8,  // 6-bit run-length repeating the previous pixel: 1..62
}

impl QoiOpRun {
    fn new() -> Self {
        Self {
            tag: Tag::B11,
            run: 1,
        }
    }

    fn add_run(&mut self) {
        self.run += 1;
    }
}

#[derive(Debug)]
struct QoiOpIndex {
    tag: Tag,  // 2-bit tag b00
    index: u8, // 6-bit index into the color index array: 0..63
}

impl QoiOpIndex {
    fn new() -> Self {
        Self {
            tag: Tag::B00,
            index: 0,
        }
    }

    fn from_rgba(color: &RGBA) -> Self {
        Self {
            tag: Tag::B00,
            index: color.hash(),
        }
    }
}

#[derive(Debug)]
struct QoiOpDiff {
    tag: Tag, // 2-bit tag b01
    dr: u8,   // 2-bit   red channel difference from the previous pixel between -2..1
    dg: u8,   // 2-bit green channel difference from the previous pixel between -2..1
    db: u8,   // 2-bit  blue channel difference from the previous pixel between -2..1
}

#[derive(Debug)]
struct QoiOpLuma {
    tag: Tag,  // 2-bit tag b10
    dg: u8,    // 6-bit green channel difference from the previous pixel -32..31
    dr_dg: u8, // 4-bit   red channel difference minus green channel difference -8..7
    dr_db: u8, // 4-bit  blue channel difference minus green channel difference -8..7
}

#[derive(Debug)]
struct QoiOpRGB {
    tag: Tag,  // 8-bit tag b11111110
    red: u8,   // 8-bit   red channel value
    green: u8, // 8-bit green channel value
    blue: u8,  // 8-bit  blue channel value
}

#[derive(Debug)]
struct QoiOpRGBA {
    tag: Tag,  // 8-bit tag b11111111
    red: u8,   // 8-bit   red channel value
    green: u8, // 8-bit green channel value
    blue: u8,  // 8-bit  blue channel value
    alpha: u8, // 8-bit alpha channel value
}

impl QoiOpRGBA {
    fn from_rgba(color: &RGBA) -> Self {
        Self {
            tag: Tag::B11111111,
            red: color.r,
            green: color.g,
            blue: color.b,
            alpha: color.a,
        }
    }
}

#[derive(Debug)]
enum QoiOps {
    Run(QoiOpRun),
    Index(QoiOpIndex),
    Diff(QoiOpDiff),
    Luma(QoiOpLuma),
    RGB(QoiOpRGB),
    RGBA(QoiOpRGBA),
}

#[derive(Debug)]
struct Encountered([RGBA; 64]);

impl Encountered {
    fn new() -> Self {
        Self([RGBA::new(); 64])
    }

    fn contains(&self, color: &RGBA) -> bool {
        self.0.contains(color)
    }

    fn set(&mut self, color: &RGBA) {
        let index = color.hash() as usize;
        self.0[index] = *color;
    }
}

impl fmt::Display for Encountered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut content = String::new();
        content += "Encountered colors:";
        for (idx, c) in self.0.iter().enumerate() {
            if !(c == &RGBA::new()) {
                content += &format!("\n{idx}: [{c}]")
            }
        }
        write!(f, "{}", content)
    }
}

struct Chunks(Vec<QoiOps>);

impl Chunks {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn last_mut(&mut self) -> Option<&mut QoiOps> {
        self.0.last_mut()
    }

    fn push(&mut self, op: QoiOps) {
        self.0.push(op)
    }
}

impl fmt::Display for Chunks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let mut content = String::new();
        content += "Chunks:";
        for v in self.0.iter() {
            let display = match v {
                QoiOps::Run(run) => format!("{run:?}"),
                QoiOps::Index(index) => format!("{index:?}"),
                QoiOps::RGBA(rgba) => format!("{rgba:?}"),
                _ => todo!(),
            };
            content += "\n";
            content += &display;
        }

        write!(f, "{}", content)
    }
}

fn main() {
    let pixels: [u32; 12] = [
        0xFFAFAFAF, 0xFFAFAFAF, 0xFFAFAFAF, 0xFFAAAFAF, 0xFFAFAFAF, 0xFFAFAFAF, 0xFFAFAFAF,
        0xFFAFAFAF, 0xFFAFAFAF, 0xFFAFAFAF, 0xFFAFAFAF, 0xFFAFAFAF,
    ];

    let mut chunks = Chunks::new();
    let mut encountered = Encountered::new();

    let mut previous = RGBA::new();
    for pixel in pixels.iter() {
        let rgba = RGBA::from(pixel);
        // dbg!(&rgba);
        if rgba == previous {
            if let Some(QOI) = chunks.last_mut() {
                match QOI {
                    QoiOps::Run(chunk) => chunk.add_run(),
                    _ => chunks.push(QoiOps::Run(QoiOpRun::new())),
                }
            }
        } else if encountered.contains(&rgba) {
            chunks.push(QoiOps::Index(QoiOpIndex::from_rgba(&rgba)));
            previous = rgba;
        } else {
            let index = rgba.hash() as usize;
            encountered.set(&rgba);
            chunks.push(QoiOps::RGBA(QoiOpRGBA::from_rgba(&rgba)));
            previous = rgba;
        }
    }

    println!("{}", chunks);
    println!("{}", encountered)
}
