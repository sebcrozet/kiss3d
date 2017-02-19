extern crate nalgebra as na;
extern crate kiss3d;

use std::path::Path;
use na::{Point2, Point3};
use kiss3d::window::Window;
use kiss3d::text::Font;

fn main() {
    let mut window = Window::new("Kiss3d: text");

    let bigfont   = Font::new(&Path::new("media/font/Inconsolata.otf"), 120);
    let smallfont = Font::new(&Path::new("media/font/Inconsolata.otf"), 60);

    while window.render() {
        window.draw_text("Hello birds!", &Point2::origin(), &bigfont, &Point3::new(0.0, 1.0, 1.0));

        let ascii = " !\"#$%&'`()*+,-_./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^abcdefghijklmnopqrstuvwxyz{|}~";

        window.draw_text(ascii, &Point2::new(0.0, 120.0), &smallfont, &Point3::new(1.0, 1.0, 0.0));
    }
}
