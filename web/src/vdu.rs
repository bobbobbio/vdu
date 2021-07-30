// copyright 2021 Remi Bernotavicius

use vdu_path_tree::{PathTree, PathTreeNode};
use wasm_bindgen::prelude::*;

struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Rectangle {
    fn divide(&self, direction: Direction, left_percent: f64) -> (Self, Self) {
        let right_percent = 1.0 - left_percent;
        match direction {
            Direction::Vertical => (
                Self {
                    x: self.x,
                    y: self.y,
                    width: self.width * left_percent,
                    height: self.height,
                },
                Self {
                    x: self.x + self.width * left_percent,
                    y: self.y,
                    width: self.width * right_percent,
                    height: self.height,
                },
            ),
            Direction::Horizontal => (
                Self {
                    x: self.x,
                    y: self.y,
                    width: self.width,
                    height: self.height * left_percent,
                },
                Self {
                    x: self.x,
                    y: self.y + self.height * left_percent,
                    width: self.width,
                    height: self.height * right_percent,
                },
            ),
        }
    }

    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn colors<'a>() -> impl Iterator<Item = &'a str> {
    vec![
        "AliceBlue",
        "AntiqueWhite",
        "Aqua",
        "Aquamarine",
        "Azure",
        "Beige",
        "Bisque",
        "BlanchedAlmond",
        "Blue",
        "BlueViolet",
        "Brown",
        "BurlyWood",
        "CadetBlue",
        "Chartreuse",
        "Chocolate",
        "Coral",
        "CornflowerBlue",
        "Cornsilk",
        "Crimson",
        "Cyan",
        "DarkBlue",
        "DarkCyan",
        "DarkGoldenRod",
        "DarkGray",
        "DarkGreen",
        "DarkGrey",
        "DarkKhaki",
        "DarkMagenta",
        "DarkOliveGreen",
        "DarkOrange",
        "DarkOrchid",
        "DarkRed",
        "DarkSalmon",
        "DarkSeaGreen",
        "DarkSlateBlue",
        "DarkSlateGray",
        "DarkSlateGrey",
        "DarkTurquoise",
        "DarkViolet",
        "DeepPink",
        "DeepSkyBlue",
        "DimGray",
        "DimGrey",
        "DodgerBlue",
        "FireBrick",
        "FloralWhite",
        "ForestGreen",
        "Fuchsia",
        "Gainsboro",
        "GhostWhite",
        "Gold",
        "GoldenRod",
        "Gray",
        "Green",
        "GreenYellow",
        "Grey",
        "HoneyDew",
        "HotPink",
        "IndianRed",
        "Indigo",
        "Ivory",
        "Khaki",
        "Lavender",
        "LavenderBlush",
        "LawnGreen",
        "LemonChiffon",
        "LightBlue",
        "LightCoral",
        "LightCyan",
        "LightGoldenRodYellow",
        "LightGray",
        "LightGreen",
        "LightGrey",
        "LightPink",
        "LightSalmon",
        "LightSeaGreen",
        "LightSkyBlue",
        "LightSlateGray",
        "LightSlateGrey",
        "LightSteelBlue",
        "LightYellow",
        "Lime",
        "LimeGreen",
        "Linen",
        "Magenta",
        "Maroon",
        "MediumAquaMarine",
        "MediumBlue",
        "MediumOrchid",
        "MediumPurple",
        "MediumSeaGreen",
        "MediumSlateBlue",
        "MediumSpringGreen",
        "MediumTurquoise",
        "MediumVioletRed",
        "MidnightBlue",
        "MintCream",
        "MistyRose",
        "Moccasin",
        "NavajoWhite",
        "Navy",
        "OldLace",
        "Olive",
        "OliveDrab",
        "Orange",
        "OrangeRed",
        "Orchid",
        "PaleGoldenRod",
        "PaleGreen",
        "PaleTurquoise",
        "PaleVioletRed",
        "PapayaWhip",
        "PeachPuff",
        "Peru",
        "Pink",
        "Plum",
        "PowderBlue",
        "Purple",
        "RebeccaPurple",
        "Red",
        "RosyBrown",
        "RoyalBlue",
        "SaddleBrown",
        "Salmon",
        "SandyBrown",
        "SeaGreen",
        "SeaShell",
        "Sienna",
        "Silver",
        "SkyBlue",
        "SlateBlue",
        "SlateGray",
        "SlateGrey",
        "Snow",
        "SpringGreen",
        "SteelBlue",
        "Tan",
        "Teal",
        "Thistle",
        "Tomato",
        "Turquoise",
        "Violet",
        "Wheat",
        "WhiteSmoke",
        "Yellow",
        "YellowGreen",
    ]
    .into_iter()
    .cycle()
}

pub struct Vdu {
    drawing_context: web_sys::CanvasRenderingContext2d,
    canvas: web_sys::HtmlCanvasElement,
    tree: PathTree,
    mouse_pos: (f64, f64),
}

#[derive(Clone, Copy)]
enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    fn next(&self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

fn divide<'a>(
    rect: Rectangle,
    nodes: Vec<(&'a str, &'a PathTreeNode)>,
    direction: Direction,
) -> Vec<(Rectangle, (&'a str, &'a PathTreeNode))> {
    if nodes.len() == 1 {
        return vec![(rect, (&nodes[0].0, &nodes[0].1))];
    }

    let mut left_nodes = nodes;
    let right_nodes = left_nodes.split_off(left_nodes.len() / 2);

    let left_sum: u64 = left_nodes.iter().map(|(_, n)| n.num_bytes()).sum();
    let right_sum: u64 = right_nodes.iter().map(|(_, n)| n.num_bytes()).sum();
    let total = left_sum + right_sum;
    let left_percent = left_sum as f64 / total as f64;

    let (left_rect, right_rect) = rect.divide(direction, left_percent);

    let left = divide(left_rect, left_nodes, direction.next());
    let right = divide(right_rect, right_nodes, direction.next());

    let mut nodes = left;
    nodes.extend(right);
    nodes
}

impl Vdu {
    pub fn new(
        drawing_context: web_sys::CanvasRenderingContext2d,
        canvas: web_sys::HtmlCanvasElement,
        tree: PathTree,
    ) -> Self {
        Self {
            drawing_context,
            canvas,
            tree,
            mouse_pos: (0.0, 0.0),
        }
    }

    fn width(&self) -> u32 {
        self.canvas.width()
    }

    fn height(&self) -> u32 {
        self.canvas.height()
    }

    fn render_helper<'a>(
        &self,
        rect: Rectangle,
        path: &str,
        iter: impl Iterator<Item = (&'a str, &'a PathTreeNode)>,
        color: &mut impl Iterator<Item = &'a str>,
        selected: &mut Option<String>,
    ) {
        let children: Vec<_> = iter.collect();
        if children.is_empty() || rect.area() < 10_000.0 {
            self.drawing_context
                .set_fill_style(&JsValue::from_str(color.next().unwrap()));
            self.drawing_context
                .fill_rect(rect.x, rect.y, rect.width, rect.height);
            if rect.contains(self.mouse_pos.0, self.mouse_pos.1) {
                self.drawing_context
                    .set_fill_style(&JsValue::from_str("black"));
                self.drawing_context
                    .stroke_rect(rect.x, rect.y, rect.width, rect.height);
                *selected = Some(path.into());
            }
        } else {
            for (new_rect, (name, node)) in divide(rect, children, Direction::Vertical) {
                let path = format!("{}/{}", path, name);
                self.render_helper(new_rect, &path, node.children(), color, selected);
            }
        }
    }

    pub fn render(&self) {
        self.drawing_context
            .set_fill_style(&JsValue::from_str("white"));
        self.drawing_context
            .clear_rect(0.0, 0.0, self.width() as f64, self.height() as f64);

        let mut selected = None;
        let starting_rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.width() as f64,
            height: self.height() as f64 - 20.0,
        };
        self.render_helper(
            starting_rect,
            "",
            self.tree.children(),
            &mut colors(),
            &mut selected,
        );

        if let Some(selected) = selected {
            self.drawing_context
                .set_fill_style(&JsValue::from_str("black"));
            self.drawing_context.set_font("30px arial");
            self.drawing_context
                .fill_text(&selected[..], 0.0, self.height() as f64)
                .unwrap();
        }
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.mouse_pos = (x, y);
    }
}
