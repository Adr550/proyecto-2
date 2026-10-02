// Fuente bitmap propia de 5 x 7; no necesita fuentes ni recursos externos.
fn glyph(c: char) -> [u8; 7] {
    match c.to_ascii_uppercase() {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'É' => [2, 4, 31, 16, 30, 16, 31],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 2, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 25, 21, 19, 19, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 27, 17],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        ':' => [0, 12, 12, 0, 12, 12, 0],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        '>' => [16, 8, 4, 2, 4, 8, 16],
        '<' => [1, 2, 4, 8, 4, 2, 1],
        _ => [0; 7],
    }
}
pub fn rect(p: &mut [u8], w: usize, x: usize, y: usize, rw: usize, rh: usize, c: [u8; 3]) {
    let h = p.len() / 4 / w;
    for yy in y..(y + rh).min(h) {
        for xx in x..(x + rw).min(w) {
            let i = (yy * w + xx) * 4;
            p[i..i + 3].copy_from_slice(&c);
        }
    }
}
pub fn text(p: &mut [u8], w: usize, x: usize, y: usize, s: &str, scale: usize, c: [u8; 3]) {
    for (j, ch) in s.chars().enumerate() {
        for (row, bits) in glyph(ch).iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    rect(
                        p,
                        w,
                        x + (j * 6 + col) * scale,
                        y + row * scale,
                        scale,
                        scale,
                        c,
                    );
                }
            }
        }
    }
}
pub fn overlay(p: &mut [u8], w: usize, _h: usize, _audio: bool, info: &str, _view: usize) {
    if info.is_empty() {
        return;
    }
    let width = 430usize.min(w - 36);
    let x = w - width - 18;
    let y = 22;
    let sections: Vec<_> = info.split('|').collect();
    let columns = (width - 36) / 12;
    let mut lines: Vec<String> = Vec::new();
    for section in &sections {
        let mut line = String::new();
        for word in section.split_whitespace() {
            if !line.is_empty() && line.len() + 1 + word.len() > columns {
                lines.push(line);
                line = String::new();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines.push(String::new());
    }
    let height = 56 + lines.len() * 20;
    rect(p, w, x, y, width, height, [245, 244, 232]);
    rect(p, w, x, y, 4, height, [70, 105, 82]);
    text(p, w, x + width - 27, y + 12, "X", 2, [55, 70, 61]);
    for (i, line) in lines.iter().enumerate() {
        text(
            p,
            w,
            x + 18,
            y + 34 + i * 20,
            line,
            2,
            if i == 0 { [35, 63, 48] } else { [70, 78, 66] },
        );
    }
    text(
        p,
        w,
        x + 18,
        y + height - 18,
        "X: VOLVER A LA VISTA ANTERIOR",
        1,
        [89, 100, 85],
    );
}
pub fn card_contains(x: f64, y: f64, w: usize, info: &str) -> bool {
    !info.is_empty() && x > w as f64 - 448. && y >= 22. && y < 420.
}
pub fn close_hit(x: f64, y: f64, w: usize) -> bool {
    x > w as f64 - 60. && x < w as f64 - 18. && (22.0..58.0).contains(&y)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    ZoomIn,
    ZoomOut,
    Meteor,
    Survivors,
    Coast,
}
pub fn action_at(
    x: f64,
    y: f64,
    w: usize,
    h: usize,
    disaster: crate::disaster::Disaster,
) -> Option<Action> {
    if y < h as f64 - 64. || y > h as f64 - 16. {
        return None;
    }
    if (18.0..66.0).contains(&x) {
        Some(Action::ZoomOut)
    } else if (76.0..124.0).contains(&x) {
        Some(Action::ZoomIn)
    } else if x >= w as f64 - 280. && x <= w as f64 - 18. {
        Some(Action::Meteor)
    } else if (446.0..696.0).contains(&x) {
        Some(Action::Coast)
    } else if disaster.aftermath() && (138.0..430.0).contains(&x) {
        Some(Action::Survivors)
    } else {
        None
    }
}
pub fn controls(
    p: &mut [u8],
    w: usize,
    h: usize,
    disaster: crate::disaster::Disaster,
    focused: bool,
) {
    let y = h - 64;
    for (x, label) in [(18, "-"), (76, "+")] {
        rect(p, w, x, y, 48, 48, [36, 43, 42]);
        text(p, w, x + 15, y + 13, label, 3, [242, 235, 216]);
    }
    rect(
        p,
        w,
        w - 280,
        y,
        262,
        48,
        if disaster.active() {
            [78, 52, 43]
        } else {
            [131, 59, 34]
        },
    );
    text(p, w, w - 266, y + 17, disaster.label(), 2, [255, 236, 205]);
    if disaster.aftermath() {
        rect(p, w, 138, y, 292, 48, [51, 59, 56]);
        text(p, w, 152, y + 17, "VER SUPERVIVIENTES", 2, [241, 237, 220]);
    }
    rect(p, w, 446, y, 250, 48, [30, 72, 84]);
    text(p, w, 462, y + 17, "GOLFO DE MÉXICO", 2, [235, 244, 240]);
    if focused {
        text(p, w, 20, 22, "X: VOLVER", 2, [245, 242, 222]);
    }
    if disaster.elapsed.is_some() {
        text(
            p,
            w,
            20,
            h - 83,
            "SECUENCIA ARTISTICA - TIEMPO COMPRIMIDO",
            1,
            [241, 231, 210],
        );
    }
}
