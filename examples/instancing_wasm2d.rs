extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::camera::{Camera, FixedView};
use kiss3d::planar_camera::{PlanarCamera, Sidescroll};
use kiss3d::scene::{PlanarInstanceData, PlanarSceneNode};
use kiss3d::window::{State, Window};
use na::{Matrix2, Point2, UnitComplex};

struct AppState {
    rect: PlanarSceneNode,
    rot_rect: UnitComplex<f32>,
    camera2d: Sidescroll,
    camera3d: FixedView,
}

impl State for AppState {
    fn step(&mut self, _window: &mut Window) {
        self.rect.prepend_to_local_rotation(&self.rot_rect);
    }

    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn kiss3d::renderer::Renderer>,
        Option<&mut dyn kiss3d::post_processing::PostProcessingEffect>,
    ) {
        (
            Some(&mut self.camera3d),
            Some(&mut self.camera2d),
            None,
            None,
        )
    }
}

fn main() {
    let mut window = Window::new("Kiss3d: instancing 2D (WASM compatible)");
    let mut rect = window.add_rectangle(50.0, 150.0);

    let rot_rect = UnitComplex::new(0.014);

    let mut instances = vec![];
    let count = 100;

    for i in 0..=count {
        for j in 0..=count {
            let shift = count as f32 / 2.0;
            let ii = i as f32;
            let jj = j as f32;
            instances.push(PlanarInstanceData {
                position: Point2::new((ii - shift) * 150.0, (jj - shift) * 150.0),
                deformation: Matrix2::new(1.0, ii * 0.004, jj * 0.004, 1.0),
                color: [ii / count as f32, jj / count as f32, 1.0, 1.0],
            });
        }
    }

    rect.set_instances(&instances);

    let camera2d = Sidescroll::new();
    let camera3d = FixedView::new();

    let state = AppState {
        rect,
        rot_rect,
        camera2d,
        camera3d,
    };

    window.render_loop(state);
}
