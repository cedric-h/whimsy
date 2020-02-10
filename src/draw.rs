
use quicksilver::geom::{Rectangle, Transform};
use quicksilver::graphics::{Color, Graphics};

#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    Rect(Rectangle),
    Fill(Color),
    Transform(Transform),
    PopTransform,
    PushTransform,
}

pub fn render_commands(gfx: &mut Graphics, draw_cmds: &std::sync::mpsc::Receiver<DrawCommand>) {
    // Remove any lingering artifacts from the previous frame
    gfx.clear(Color::BLACK);

    let mut color = Color::WHITE;
    let mut transforms = vec![quicksilver::geom::Transform::IDENTITY];

    while let Ok(cmd) = draw_cmds.try_recv() {
        use DrawCommand::*;
        match cmd {
            Rect(rect) => gfx.fill_rect(&rect, color),
            Fill(new_color) => color = new_color,
            Transform(t) => {
                let now = transforms.last_mut().unwrap();
                *now = *now * t;
                gfx.set_transform(*now);
            }
            PushTransform => {
                let now = transforms.last().unwrap().clone();
                transforms.push(now);
                gfx.set_transform(now);
            }
            PopTransform => {
                if transforms.len() >= 2 {
                    transforms.pop();
                }
                gfx.set_transform(*transforms.last().unwrap());
            }
        }
    }
}
